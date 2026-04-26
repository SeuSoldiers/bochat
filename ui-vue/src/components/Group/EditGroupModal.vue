<template>
  <div class="modal-overlay" @click="$emit('close')">
    <div class="modal-content" @click.stop>
      <div class="modal-header">
        <h2>编辑群聊</h2>
        <button class="close-btn" @click="$emit('close')">✕</button>
      </div>

      <form @submit.prevent="handleSubmit" class="modal-form">
        <div class="form-group">
          <label for="edit-group-name">群名称</label>
          <input id="edit-group-name" v-model="form.name" type="text" maxlength="50" required :disabled="loading" />
        </div>

        <div class="form-group">
          <label for="edit-group-description">群简介（可选）</label>
          <textarea
            id="edit-group-description"
            v-model="form.description"
            rows="3"
            maxlength="200"
            placeholder="请输入群简介"
            :disabled="loading"
          />
        </div>

        <div class="form-group">
          <label for="edit-group-code">群号</label>
          <input
            id="edit-group-code"
            v-model="form.groupCode"
            type="text"
            maxlength="20"
            placeholder="请输入群号"
            :disabled="loading"
          />
        </div>

        <div class="form-group">
          <label for="edit-group-avatar-url">头像链接（可选）</label>
          <input
            id="edit-group-avatar-url"
            v-model="form.avatarUrl"
            type="url"
            placeholder="请输入头像链接"
            :disabled="loading || uploading"
          />
          <div v-if="form.avatarUrl" class="avatar-preview">
            <img :src="form.avatarUrl" alt="群头像预览" />
          </div>
          <div class="upload-row">
            <input
              ref="fileInput"
              type="file"
              accept="image/*"
              class="hidden-input"
              @change="handleFileChange"
            />
            <button
              type="button"
              class="btn-upload"
              :disabled="loading || uploading || !uploadToken"
              @click="triggerUpload"
            >
              {{ uploading ? '上传中...' : '上传头像' }}
            </button>
          </div>
          <p class="help-text">可直接填写链接，或上传后自动回填</p>
        </div>

        <div v-if="error" class="error-message">{{ error }}</div>

        <div class="form-actions">
          <button type="button" class="btn-cancel" @click="$emit('close')" :disabled="loading">取消</button>
          <button type="submit" class="btn-submit" :disabled="loading">{{ loading ? '保存中...' : '保存' }}</button>
        </div>
      </form>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import type { Group } from '@/types'
import { uploadFile } from '@/services/file'
import { getErrorMessage } from '@/utils/error'

const props = defineProps<{
  group: Group
  uploadToken?: string
}>()

const emit = defineEmits<{
  save: [payload: { name: string; description: string; groupCode: string; avatarUrl: string }]
  close: []
}>()

const form = ref({
  name: props.group.name,
  description: props.group.description || '',
  groupCode: props.group.group_code || '',
  avatarUrl: props.group.avatar_url || '',
})
const loading = ref(false)
const uploading = ref(false)
const error = ref<string | null>(null)
const fileInput = ref<HTMLInputElement | null>(null)

const handleSubmit = async () => {
  if (!form.value.name.trim()) {
    error.value = '请输入群名称'
    return
  }

  loading.value = true
  error.value = null

  try {
    emit('save', {
      name: form.value.name,
      description: form.value.description,
      groupCode: form.value.groupCode,
      avatarUrl: form.value.avatarUrl,
    })
  } catch (err: any) {
    error.value = getErrorMessage(err, '保存失败')
  } finally {
    loading.value = false
  }
}

const triggerUpload = () => {
  if (!props.uploadToken) {
    error.value = '当前没有可用机器人令牌，无法上传头像'
    return
  }
  fileInput.value?.click()
}

const handleFileChange = async (event: Event) => {
  const target = event.target as HTMLInputElement
  const file = target.files?.[0]
  if (!file || !props.uploadToken) {
    return
  }

  uploading.value = true
  error.value = null

  try {
    const uploaded = await uploadFile(file, props.uploadToken)
    form.value.avatarUrl = uploaded.url
  } catch (err: any) {
    error.value = getErrorMessage(err, '头像上传失败')
  } finally {
    uploading.value = false
    target.value = ''
  }
}
</script>

<style scoped>
.modal-overlay {
  position: fixed;
  inset: 0;
  background-color: rgba(0, 0, 0, 0.3);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
  padding: 16px;
}

.modal-content {
  width: 100%;
  max-width: 540px;
  background: #f5f5f5;
  border: 1px solid #d0d0d0;
  border-radius: 8px;
  box-shadow: 0 8px 18px rgba(0, 0, 0, 0.12);
}

.modal-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px 18px;
  border-bottom: 1px solid #d0d0d0;
}

.modal-header h2 {
  font-size: 16px;
  font-weight: 600;
  color: #2f2f2f;
  margin: 0;
}

.close-btn {
  background: transparent;
  border: none;
  width: 30px;
  height: 30px;
  border-radius: 6px;
  font-size: 18px;
  color: #6f6f6f;
  cursor: pointer;
  transition: var(--transition-base);
}

.close-btn:hover {
  background: #eaeaea;
  color: #2f2f2f;
}

.modal-form {
  padding: 16px 18px 18px;
}

.form-group {
  margin-bottom: 14px;
}

.form-group label {
  display: block;
  margin-bottom: 6px;
  font-size: 13px;
  color: #4f4f4f;
  font-weight: 600;
}

.form-group input,
.form-group textarea {
  width: 100%;
  padding: 10px 11px;
  border: 1px solid #d0d0d0;
  border-radius: 6px;
  font-size: 14px;
  color: #2f2f2f;
  background: #f7f7f7;
  transition: var(--transition-base);
}

.form-group textarea {
  min-height: 88px;
  resize: vertical;
}

.form-group input:focus,
.form-group textarea:focus {
  outline: none;
  border-color: #8f8f8f;
  box-shadow: 0 0 0 3px rgba(120, 120, 120, 0.12);
}

.form-group input:disabled,
.form-group textarea:disabled {
  background-color: #f0f0f0;
  color: #9a9a9a;
  cursor: not-allowed;
}

.avatar-preview {
  width: 64px;
  height: 64px;
  margin-top: 8px;
  border-radius: 50%;
  overflow: hidden;
  background: #ebebeb;
}

.avatar-preview img {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.upload-row {
  margin-top: 10px;
}

.hidden-input {
  display: none;
}

.btn-upload {
  padding: 8px 12px;
  border: 1px solid #d0d0d0;
  border-radius: 6px;
  background: #efefef;
  color: #2f2f2f;
  cursor: pointer;
  transition: var(--transition-base);
}

.btn-upload:hover:not(:disabled) {
  background: #e7e7e7;
}

.help-text {
  margin-top: 4px;
  font-size: 12px;
  color: #767676;
}

.error-message {
  margin-bottom: 14px;
  padding: 9px 11px;
  border: 1px solid #e0b7b7;
  border-radius: 6px;
  background-color: #f3e7e7;
  color: #9b3c3c;
  font-size: 12px;
}

.form-actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
}

.form-actions button {
  padding: 9px 14px;
  border: 1px solid transparent;
  border-radius: 6px;
  font-size: 13px;
  font-weight: 600;
  cursor: pointer;
  transition: var(--transition-base);
}

.btn-cancel {
  border-color: #d0d0d0;
  background: #efefef;
  color: #2f2f2f;
}

.btn-cancel:hover:not(:disabled) {
  background: #e7e7e7;
}

.btn-submit {
  border-color: #2f2f2f;
  background: #2f2f2f;
  color: #f3f3f3;
}

.btn-submit:hover:not(:disabled) {
  background: #454545;
}

.btn-cancel:disabled,
.btn-submit:disabled,
.btn-upload:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}
</style>
