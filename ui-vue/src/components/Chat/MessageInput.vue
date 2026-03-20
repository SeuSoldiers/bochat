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
        <button
          type="submit"
          class="send-btn"
          :disabled="!messageText.trim() || sending"
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
}>()

const messageText = ref('')
const selectedBotId = ref(props.initialBotId || '')
const sending = ref(false)
const error = ref<string | null>(null)

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
  padding: 15px;
  background-color: white;
  border-radius: 8px;
  border: 1px solid #d4cfc8;
}

.input-toolbar {
  display: flex;
}

.bot-select {
  min-width: 220px;
  padding: 10px 12px;
  border: 1px solid #d4cfc8;
  border-radius: 6px;
  font-size: 13px;
  color: #4a4a4a;
  background: white;
}

.input-wrapper {
  display: flex;
  gap: 10px;
}

.message-input {
  flex: 1;
  padding: 10px 12px;
  border: 1px solid #e8e3dd;
  border-radius: 6px;
  font-size: 13px;
  color: #4a4a4a;
  font-family: inherit;
  resize: vertical;
  min-height: 60px;
  max-height: 150px;
  transition: all 0.3s ease;
}

.message-input:focus {
  outline: none;
  border-color: #8b9d83;
  box-shadow: 0 0 0 3px rgba(139, 157, 131, 0.1);
}

.message-input:disabled {
  background-color: #fafaf8;
  color: #cccccc;
  cursor: not-allowed;
}

.input-footer {
  display: flex;
  justify-content: flex-end;
}

.send-btn {
  padding: 10px 24px;
  background-color: #8b9d83;
  color: white;
  border: none;
  border-radius: 6px;
  font-size: 13px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.3s ease;
}

.send-btn:hover:not(:disabled) {
  background-color: #9caa93;
  box-shadow: 0 2px 8px rgba(139, 157, 131, 0.3);
}

.send-btn:active:not(:disabled) {
  background-color: #7a8c72;
}

.send-btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.error-message {
  padding: 10px 12px;
  background-color: #f5e6e6;
  color: #a88b7f;
  border-radius: 6px;
  font-size: 12px;
}
</style>
