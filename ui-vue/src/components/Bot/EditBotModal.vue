<template>
  <div class="modal-overlay" @click="$emit('close')">
    <div class="modal-content" @click.stop>
      <div class="modal-header">
        <h2>编辑机器人</h2>
        <button class="close-btn" @click="$emit('close')">✕</button>
      </div>

      <form @submit.prevent="handleSubmit" class="modal-form">
        <div class="form-group">
          <label for="edit-bot-name">机器人名称</label>
          <input id="edit-bot-name" v-model="form.name" type="text" maxlength="50" required :disabled="loading" />
        </div>

        <div class="form-group">
          <label for="edit-bot-description">描述</label>
          <textarea id="edit-bot-description" v-model="form.description" rows="3" maxlength="200" :disabled="loading" />
        </div>

        <div class="form-group">
          <label for="edit-bot-avatar-url">头像链接</label>
          <input id="edit-bot-avatar-url" v-model="form.avatarUrl" type="url" :disabled="loading || uploading" />
          <div v-if="form.avatarUrl" class="avatar-preview">
            <img :src="form.avatarUrl" alt="机器人头像预览" />
          </div>
          <div class="upload-row">
            <input ref="fileInput" type="file" accept="image/*" class="hidden-input" @change="handleFileChange" />
            <button type="button" class="btn-upload" :disabled="loading || uploading" @click="triggerUpload">
              {{ uploading ? '上传中...' : '上传头像' }}
            </button>
          </div>
          <p class="help-text">可以直接填写链接，也可以先上传文件再自动回填链接</p>
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
import type { Bot } from '@/types'
import { uploadFile } from '@/services/file'
import { getErrorMessage } from '@/utils/error'

const props = defineProps<{
  bot: Bot
}>()

const emit = defineEmits<{
  save: [payload: { name: string; description: string; avatarUrl: string }]
  close: []
}>()

const form = ref({
  name: props.bot.name,
  description: props.bot.description || '',
  avatarUrl: props.bot.avatar_url || '',
})
const loading = ref(false)
const uploading = ref(false)
const error = ref<string | null>(null)
const fileInput = ref<HTMLInputElement | null>(null)

const handleSubmit = async () => {
  if (!form.value.name.trim()) {
    error.value = '请输入机器人名称'
    return
  }

  loading.value = true
  error.value = null

  try {
    emit('save', {
      name: form.value.name,
      description: form.value.description,
      avatarUrl: form.value.avatarUrl,
    })
  } catch (err: any) {
    error.value = getErrorMessage(err, '保存失败')
  } finally {
    loading.value = false
  }
}

const triggerUpload = () => {
  fileInput.value?.click()
}

const handleFileChange = async (event: Event) => {
  const target = event.target as HTMLInputElement
  const file = target.files?.[0]
  if (!file) {
    return
  }

  uploading.value = true
  error.value = null

  try {
    const uploaded = await uploadFile(file, props.bot.token)
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
  padding: 20px;
}

.modal-content {
  background: white;
  border-radius: 8px;
  width: 100%;
  max-width: 520px;
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.15);
}

.modal-header,
.modal-form {
  padding: 20px;
}

.modal-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  border-bottom: 1px solid #d4cfc8;
}

.close-btn,
.btn-upload,
.btn-cancel,
.btn-submit {
  border: none;
  border-radius: 6px;
  cursor: pointer;
}

.close-btn {
  background: none;
  font-size: 20px;
}

.form-group {
  margin-bottom: 16px;
}

.form-group input,
.form-group textarea {
  width: 100%;
  padding: 10px 12px;
  border: 1px solid #d4cfc8;
  border-radius: 6px;
}

.avatar-preview {
  width: 64px;
  height: 64px;
  margin-bottom: 10px;
  border-radius: 50%;
  overflow: hidden;
  background: #f5f3f1;
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
  background-color: #d4cfc8;
  color: #4a4a4a;
}

.help-text {
  margin-top: 8px;
  font-size: 12px;
  color: #888888;
}

.error-message {
  padding: 10px 12px;
  background-color: #f5e6e6;
  color: #a88b7f;
  border-radius: 6px;
  margin-bottom: 16px;
}

.form-actions {
  display: flex;
  justify-content: flex-end;
  gap: 10px;
}

.btn-cancel {
  padding: 10px 16px;
  background-color: #d4cfc8;
}

.btn-submit {
  padding: 10px 16px;
  background-color: #8b9d83;
  color: white;
}
</style>
