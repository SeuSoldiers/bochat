<template>
  <div class="modal-overlay" @click="$emit('close')">
    <div class="modal-content" @click.stop>
      <div class="modal-header">
        <h2>创建新群</h2>
        <button class="close-btn" @click="$emit('close')">✕</button>
      </div>

      <form @submit.prevent="handleSubmit" class="modal-form">
        <div class="form-group">
          <label for="create-group-bot">加入群聊的机器人</label>
          <select id="create-group-bot" v-model="form.botId" :disabled="loading">
            <option value="">请选择机器人</option>
            <option v-for="bot in bots" :key="bot.bot_id" :value="bot.bot_id">
              {{ bot.name }} ({{ bot.bot_id.slice(0, 8) }}...)
            </option>
          </select>
        </div>

        <div class="form-group">
          <label for="group-name">群名称</label>
          <input
            id="group-name"
            v-model="form.groupName"
            type="text"
            placeholder="请输入群名称"
            maxlength="50"
            required
            :disabled="loading"
          />
        </div>

        <div class="form-group">
          <label for="group-description">群简介（可选）</label>
          <textarea
            id="group-description"
            v-model="form.description"
            rows="3"
            maxlength="200"
            placeholder="请输入群简介"
            :disabled="loading"
          />
        </div>

        <div class="form-group">
          <label for="group-number">群号</label>
          <input
            id="group-number"
            v-model="form.groupNumber"
            type="text"
            placeholder="请输入群号"
            maxlength="20"
            required
            :disabled="loading"
          />
          <p class="help-text">群号必须唯一</p>
        </div>

        <div class="form-group">
          <label class="switch-row">
            <input v-model="form.isPublic" type="checkbox" :disabled="loading" />
            <span>公开群聊（允许任意 Bot 自由加入）</span>
          </label>
        </div>

        <div class="form-group">
          <label for="group-avatar-url">群头像（可选）</label>
          <input
            id="group-avatar-url"
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
              :disabled="loading || uploading || !selectedBotToken"
              @click="triggerUpload"
            >
              {{ uploading ? '上传中...' : '上传头像' }}
            </button>
          </div>
          <p class="help-text">请选择加入群聊的机器人后再上传头像</p>
        </div>

        <div v-if="error" class="error-message">
          {{ error }}
        </div>

        <div class="form-actions">
          <button type="button" class="btn-cancel" @click="$emit('close')" :disabled="loading">
            取消
          </button>
          <button type="submit" class="btn-submit" :disabled="loading">
            {{ loading ? '创建中...' : '创建' }}
          </button>
        </div>
      </form>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'
import type { Bot } from '@/types'
import { getErrorMessage } from '@/utils/error'
import { uploadFile } from '@/services/file'

const props = defineProps<{
  bots: Bot[]
}>()

const emit = defineEmits<{
  create: [
    groupName: string,
    description: string,
    groupNumber: string,
    botId: string,
    avatarUrl: string,
    isPublic: boolean,
  ]
  close: []
}>()

const form = ref({
  botId: '',
  groupName: '',
  description: '',
  groupNumber: '',
  avatarUrl: '',
  isPublic: false,
})
const loading = ref(false)
const uploading = ref(false)
const error = ref<string | null>(null)
const fileInput = ref<HTMLInputElement | null>(null)
const selectedBotToken = computed(() => {
  if (!form.value.botId) {
    return ''
  }
  return props.bots.find((bot) => bot.bot_id === form.value.botId)?.token || ''
})

const handleSubmit = async () => {
  if (!form.value.groupName.trim()) {
    error.value = '请输入群名称'
    return
  }

  if (!form.value.botId) {
    error.value = '请选择一个机器人'
    return
  }

  if (!form.value.groupNumber.trim()) {
    error.value = '请输入群号'
    return
  }

  loading.value = true
  error.value = null

  try {
    emit(
      'create',
      form.value.groupName,
      form.value.description,
      form.value.groupNumber,
      form.value.botId,
      form.value.avatarUrl,
      form.value.isPublic
    )
  } catch (err: any) {
    error.value = getErrorMessage(err, '创建失败')
  } finally {
    loading.value = false
  }
}

const triggerUpload = () => {
  if (!selectedBotToken.value) {
    error.value = '请先选择加入群聊的机器人'
    return
  }
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
    const uploaded = await uploadFile(file, selectedBotToken.value)
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
  background-color: rgba(0, 0, 0, 0.28);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
  padding: 16px;
}

.modal-content {
  background: #f5f5f5;
  border: 1px solid #d0d0d0;
  border-radius: 8px;
  width: 100%;
  max-width: 540px;
  max-height: 82vh;
  box-shadow: 0 8px 18px rgba(0, 0, 0, 0.12);
  display: flex;
  flex-direction: column;
  overflow: hidden;
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

.modal-form textarea {
  min-height: 88px;
  resize: vertical;
}

.close-btn {
  background: transparent;
  border: none;
  width: 30px;
  height: 30px;
  border-radius: 6px;
  font-size: 18px;
  cursor: pointer;
  color: #6f6f6f;
  transition: var(--transition-base);
}

.close-btn:hover {
  background: #eaeaea;
  color: #2f2f2f;
}

.modal-form {
  padding: 16px 18px 18px;
  overflow-y: auto;
  min-height: 0;
}

.form-group {
  margin-bottom: 14px;
  position: relative;
}

.form-group label {
  display: block;
  margin-bottom: 6px;
  font-size: 13px;
  color: #4f4f4f;
  font-weight: 600;
}

.switch-row {
  display: flex !important;
  align-items: center;
  gap: 8px;
  margin: 0;
}

.switch-row input[type='checkbox'] {
  width: 16px;
  height: 16px;
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

.form-group select {
  width: 100%;
  padding: 10px 11px;
  border: 1px solid #d0d0d0;
  border-radius: 6px;
  font-size: 14px;
  color: #2f2f2f;
  background: #f7f7f7;
  transition: var(--transition-base);
}

.form-group input:focus,
.form-group textarea:focus {
  outline: none;
  border-color: #8f8f8f;
  box-shadow: 0 0 0 3px rgba(120, 120, 120, 0.12);
}

.form-group select:focus {
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

.help-text {
  font-size: 12px;
  color: #767676;
  margin: 4px 0 0 0;
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

.error-message {
  padding: 9px 11px;
  background-color: #f3e7e7;
  color: #9b3c3c;
  border: 1px solid #e0b7b7;
  border-radius: 6px;
  font-size: 12px;
  margin-bottom: 14px;
}

.form-actions {
  display: flex;
  gap: 8px;
  justify-content: flex-end;
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
  background-color: #efefef;
  color: #2f2f2f;
}

.btn-cancel:hover:not(:disabled) {
  background-color: #e7e7e7;
}

.btn-submit {
  border-color: #2f2f2f;
  background-color: #2f2f2f;
  color: #f3f3f3;
}

.btn-submit:hover:not(:disabled) {
  background-color: #454545;
}

.btn-cancel:disabled,
.btn-submit:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.btn-upload:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}
</style>
