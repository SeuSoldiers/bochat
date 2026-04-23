<template>
  <div class="message-input-container">
    <form @submit.prevent="handleSend" class="message-input-form">
      <div class="input-toolbar">
        <select v-model="selectedBotId" class="bot-select" :disabled="sending">
          <option value="">选择发送 Bot</option>
          <option v-for="bot in bots" :key="bot.bot_id" :value="bot.bot_id">
            {{ bot.name }} ({{ bot.bot_id.slice(0, 8) }}...)
          </option>
        </select>
      </div>

      <div class="input-wrapper">
        <textarea
          v-model="messageText"
          placeholder="输入消息... (Shift + Enter 换行, Enter 发送)"
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
    </form>

    <div v-if="error" class="error-message">
      {{ error }}
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue'
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
    error.value = '请选择 Bot 和群'
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
    error.value = '请选择 Bot 和群'
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
    error.value = '请选择 Bot 和群'
    input.value = ''
    return
  }

  uploading.value = true
  error.value = null
  try {
    emit('send-file', selectedFile, selectedBotId.value)
  } catch (err: any) {
    error.value = err.message || '文件发送失败'
  } finally {
    uploading.value = false
    input.value = ''
  }
}
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
  background: linear-gradient(145deg, #ffffff 0%, #f8fbf6 100%);
  border-radius: 16px;
  border: 1px solid #d8dfd8;
  box-shadow: 0 10px 22px rgba(10, 36, 27, 0.08);
}

.input-toolbar {
  display: flex;
}

.bot-select {
  min-width: 220px;
  padding: 10px 12px;
  border: 1px solid #d7e0d7;
  border-radius: 10px;
  font-size: 13px;
  color: #23322c;
  background: #fbfdf9;
}

.input-wrapper {
  display: flex;
  gap: 10px;
}

.message-input {
  flex: 1;
  padding: 10px 12px;
  border: 1px solid #d7e0d7;
  border-radius: 10px;
  font-size: 13px;
  color: #23322c;
  font-family: inherit;
  resize: vertical;
  min-height: 60px;
  max-height: 150px;
  transition: var(--transition-base);
  background: #fbfdf9;
}

.message-input:focus {
  outline: none;
  border-color: #99cb27;
  box-shadow: 0 0 0 4px rgba(166, 215, 46, 0.2);
}

.message-input:disabled {
  background-color: #fafaf8;
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
  background-color: #f5f8f2;
  color: #3f4d47;
  border: 1px solid #d8dfd8;
  border-radius: 999px;
  font-size: 13px;
  font-weight: 600;
  cursor: pointer;
  transition: var(--transition-base);
}

.file-btn:hover:not(:disabled) {
  background-color: #edf3e8;
}

.file-btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.send-btn {
  padding: 10px 24px;
  background: #a6d72e;
  color: #143227;
  border: 1px solid #98c52c;
  border-radius: 999px;
  font-size: 13px;
  font-weight: 600;
  cursor: pointer;
  transition: var(--transition-base);
}

.send-btn:hover:not(:disabled) {
  background-color: #b5de46;
  box-shadow: 0 12px 22px rgba(14, 60, 47, 0.18);
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
</style>
