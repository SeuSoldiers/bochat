<template>
  <div class="message-input-container">
    <form @submit.prevent="handleSend" class="message-input-form">
      <div class="input-toolbar">
        <select v-model="selectedBotId" class="bot-select" :disabled="sending">
          <option value="">选择发送机器人</option>
          <option v-for="bot in bots" :key="bot.bot_id" :value="bot.bot_id">
            {{ bot.name }} ({{ bot.bot_id.slice(0, 8) }}...)
          </option>
        </select>
      </div>

      <div class="input-wrapper">
        <textarea
          v-model="messageText"
          placeholder="输入消息...（按回车发送，按组合键换行）"
          class="message-input"
          :disabled="sending"
          @keydown.enter="handleKeydown"
        />
      </div>

      <div class="input-footer">
        <input
          ref="fileInputRef"
          type="file"
          class="file-input-hidden"
          :disabled="sending || uploading"
          @change="handleFileChange"
        />
        <button
          type="button"
          class="file-btn"
          :disabled="sending || uploading"
          @click="triggerFilePicker"
        >
          {{ uploading ? '上传中...' : '发文件' }}
        </button>
        <button
          type="submit"
          class="send-btn"
          :disabled="!messageText.trim() || sending || uploading"
        >
          {{ sending ? '发送中...' : '发送' }}
        </button>
      </div>

      <div v-if="pendingFile" class="file-preview-card">
        <div v-if="pendingFileIsImage && pendingFilePreviewUrl" class="file-preview-image-wrap">
          <img :src="pendingFilePreviewUrl" :alt="pendingFile.name" class="file-preview-image" />
        </div>
        <div v-else class="file-preview-icon-wrap" aria-hidden="true">
          <component :is="pendingFileIconComponent" class="file-preview-icon" />
        </div>
        <div class="file-preview-meta">
          <p class="file-preview-name">{{ pendingFile.name }}</p>
          <p class="file-preview-size">{{ formatFileSize(pendingFile.size) }}</p>
        </div>
        <div class="file-preview-actions">
          <button
            type="button"
            class="preview-send-btn"
            :disabled="sending || uploading"
            @click="handleSendPendingFile"
          >
            {{ uploading ? '上传中...' : '发送文件' }}
          </button>
          <button
            type="button"
            class="preview-cancel-btn"
            :disabled="sending || uploading"
            @click="clearPendingFile"
          >
            取消
          </button>
        </div>
      </div>
    </form>

    <div v-if="error" class="error-message">
      {{ error }}
    </div>
  </div>
</template>

<script setup lang="ts">
import { onBeforeUnmount, ref, watch } from 'vue'
import type { Component } from 'vue'
import {
  File as IconFile,
  FileArchive as IconFileArchive,
  FileAudio as IconFileAudio,
  FileCode2 as IconFileCode,
  FileImage as IconFileImage,
  FileSpreadsheet as IconFileSheet,
  FileText as IconFileText,
  FileVideo as IconFileVideo,
} from 'lucide-vue-next'
import type { Bot } from '@/types'

const props = defineProps<{
  groupId: string
  bots: Bot[]
  initialBotId?: string
}>()

const emit = defineEmits<{
  send: [content: string, botId: string]
  'send-file': [file: File, botId: string]
}>()

const messageText = ref('')
const selectedBotId = ref(props.initialBotId || '')
const sending = ref(false)
const uploading = ref(false)
const error = ref<string | null>(null)
const fileInputRef = ref<HTMLInputElement | null>(null)
const pendingFile = ref<File | null>(null)
const pendingFilePreviewUrl = ref<string | null>(null)
const pendingFileIsImage = ref(false)
const pendingFileIconComponent = ref<Component>(IconFile)

watch(
  () => props.initialBotId,
  (botId) => {
    if (botId) {
      selectedBotId.value = botId
    }
  },
  { immediate: true }
)

const handleKeydown = (event: KeyboardEvent) => {
  if (event.key === 'Enter' && !event.shiftKey) {
    event.preventDefault()
    handleSend()
  }
}

const handleSend = async () => {
  const content = messageText.value.trim()

  if (!content) {
    return
  }

  if (!props.groupId || !selectedBotId.value) {
    error.value = '请选择机器人和群'
    return
  }

  sending.value = true
  error.value = null

  try {
    emit('send', content, selectedBotId.value)
    messageText.value = ''
  } catch (err: any) {
    error.value = err.message || '发送失败'
  } finally {
    sending.value = false
  }
}

const triggerFilePicker = () => {
  if (!props.groupId || !selectedBotId.value) {
    error.value = '请选择机器人和群'
    return
  }

  fileInputRef.value?.click()
}

const handleFileChange = async (event: Event) => {
  const input = event.target as HTMLInputElement
  const selectedFile = input.files?.[0]
  if (!selectedFile) {
    return
  }

  if (!props.groupId || !selectedBotId.value) {
    error.value = '请选择机器人和群'
    input.value = ''
    return
  }

  clearPendingFile()
  error.value = null
  pendingFile.value = selectedFile
  pendingFileIsImage.value = selectedFile.type.startsWith('image/')
  pendingFileIconComponent.value = resolveFileIconComponent(selectedFile.name)
  if (pendingFileIsImage.value) {
    pendingFilePreviewUrl.value = URL.createObjectURL(selectedFile)
  }
  input.value = ''
}

const handleSendPendingFile = async () => {
  if (!pendingFile.value || !selectedBotId.value || !props.groupId) {
    return
  }

  uploading.value = true
  error.value = null
  try {
    emit('send-file', pendingFile.value, selectedBotId.value)
    clearPendingFile()
  } catch (err: any) {
    error.value = err.message || '文件发送失败'
  } finally {
    uploading.value = false
  }
}

const clearPendingFile = () => {
  if (pendingFilePreviewUrl.value) {
    URL.revokeObjectURL(pendingFilePreviewUrl.value)
  }
  pendingFile.value = null
  pendingFilePreviewUrl.value = null
  pendingFileIsImage.value = false
  pendingFileIconComponent.value = IconFile
}

const resolveFileIconComponent = (name: string): Component => {
  const lower = name.toLowerCase()
  if (/\.(png|jpe?g|gif|webp|bmp|svg|ico)$/.test(lower)) return IconFileImage
  if (/\.(mp4|mov|mkv|avi|webm)$/.test(lower)) return IconFileVideo
  if (/\.(mp3|wav|flac|aac|ogg)$/.test(lower)) return IconFileAudio
  if (/\.(zip|rar|7z|tar|gz)$/.test(lower)) return IconFileArchive
  if (/\.(xls|xlsx|csv)$/.test(lower)) return IconFileSheet
  if (/\.(pdf|doc|docx|odt|rtf|ppt|pptx|key)$/.test(lower)) return IconFileText
  if (/\.(js|ts|tsx|jsx|py|rs|go|java|c|cpp|h|hpp|json|yaml|yml|toml|md)$/.test(lower)) return IconFileCode
  return IconFile
}

const formatFileSize = (bytes: number) => {
  if (!Number.isFinite(bytes) || bytes < 1024) return `${Math.max(0, Math.floor(bytes))} B`
  const kb = bytes / 1024
  if (kb < 1024) return `${kb.toFixed(1)} KB`
  const mb = kb / 1024
  if (mb < 1024) return `${mb.toFixed(1)} MB`
  const gb = mb / 1024
  return `${gb.toFixed(1)} GB`
}

onBeforeUnmount(() => {
  clearPendingFile()
})
</script>

<style scoped>
.message-input-container {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.message-input-form {
  display: flex;
  flex-direction: column;
  gap: 10px;
  padding: 16px;
  background: #f5f5f5;
  border-radius: 10px;
  border: 1px solid #d0d0d0;
  box-shadow: 0 4px 10px rgba(0, 0, 0, 0.05);
}

.input-toolbar {
  display: flex;
}

.bot-select {
  min-width: 220px;
  padding: 10px 12px;
  border: 1px solid #d0d0d0;
  border-radius: 8px;
  font-size: 13px;
  color: #2f2f2f;
  background: #f5f5f5;
}

.input-wrapper {
  display: flex;
  gap: 10px;
}

.message-input {
  flex: 1;
  padding: 10px 12px;
  border: 1px solid #d0d0d0;
  border-radius: 8px;
  font-size: 13px;
  color: #2f2f2f;
  font-family: inherit;
  resize: vertical;
  min-height: 60px;
  max-height: 150px;
  transition: var(--transition-base);
  background: #f5f5f5;
}

.message-input:focus {
  outline: none;
  border-color: #7a7a7a;
  box-shadow: 0 0 0 4px rgba(120, 120, 120, 0.2);
}

.message-input:disabled {
  background-color: #efefef;
  color: #cccccc;
  cursor: not-allowed;
}

.input-footer {
  display: flex;
  gap: 8px;
  justify-content: flex-end;
}

.file-input-hidden {
  display: none;
}

.file-btn {
  padding: 10px 16px;
  background-color: #f5f5f5;
  color: #4f4f4f;
  border: 1px solid #d0d0d0;
  border-radius: 999px;
  font-size: 13px;
  font-weight: 600;
  cursor: pointer;
  transition: var(--transition-base);
}

.file-btn:hover:not(:disabled) {
  background-color: #ebebeb;
}

.file-btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.send-btn {
  padding: 10px 24px;
  background: #2f2f2f;
  color: #f3f3f3;
  border: 1px solid #2f2f2f;
  border-radius: 999px;
  font-size: 13px;
  font-weight: 600;
  cursor: pointer;
  transition: var(--transition-base);
}

.send-btn:hover:not(:disabled) {
  background-color: #3a3a3a;
  box-shadow: 0 8px 16px rgba(0, 0, 0, 0.12);
  transform: translateY(-1px);
}

.send-btn:active:not(:disabled) {
  transform: translateY(0);
}

.send-btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.error-message {
  padding: 10px 12px;
  background-color: #fff0ee;
  color: #a6453e;
  border-radius: 10px;
  font-size: 12px;
  border: 1px solid #efc5bf;
}

.file-preview-card {
  border: 1px solid #d0d0d0;
  border-radius: 10px;
  background: #f0f0f0;
  padding: 10px;
  display: flex;
  align-items: center;
  gap: 10px;
}

.file-preview-image-wrap {
  width: 56px;
  height: 56px;
  border-radius: 8px;
  overflow: hidden;
  border: 1px solid #d0d0d0;
  background: #f8f8f8;
  flex-shrink: 0;
}

.file-preview-image {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.file-preview-icon-wrap {
  width: 56px;
  height: 56px;
  border-radius: 8px;
  display: grid;
  place-items: center;
  border: 1px solid #d0d0d0;
  background: #f8f8f8;
  flex-shrink: 0;
}

.file-preview-icon {
  width: 24px;
  height: 24px;
  stroke: #3f3f3f;
  stroke-width: 1.8;
}

.file-preview-meta {
  min-width: 0;
  flex: 1;
}

.file-preview-name {
  margin: 0;
  font-size: 13px;
  color: #2f2f2f;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.file-preview-size {
  margin: 4px 0 0;
  font-size: 12px;
  color: #6f6f6f;
}

.file-preview-actions {
  display: flex;
  gap: 8px;
}

.preview-send-btn,
.preview-cancel-btn {
  height: 30px;
  border-radius: 999px;
  border: 1px solid #d0d0d0;
  padding: 0 12px;
  font-size: 12px;
  cursor: pointer;
  transition: var(--transition-base);
}

.preview-send-btn {
  border-color: #2f2f2f;
  background: #2f2f2f;
  color: #f3f3f3;
}

.preview-cancel-btn {
  background: #f5f5f5;
  color: #4f4f4f;
}

.preview-send-btn:disabled,
.preview-cancel-btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}
</style>
