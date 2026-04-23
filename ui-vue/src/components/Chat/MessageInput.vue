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
</style>
