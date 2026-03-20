<template>
  <div class="message-list">
    <div v-if="loading" class="loading-state">
      加载消息中...
    </div>

    <div v-else-if="messages.length > 0" class="messages">
      <div v-for="message in messages" :key="message.message_id" class="message-group">
        <MessageItem :message="message" />
      </div>
    </div>

    <div v-else class="empty-state">
      <p>暂无消息</p>
    </div>
  </div>
</template>

<script setup lang="ts">
import type { Message } from '@/types'
import MessageItem from './MessageItem.vue'

defineProps<{
  messages: Message[]
  loading?: boolean
}>()
</script>

<style scoped>
.message-list {
  flex: 1;
  display: flex;
  flex-direction: column;
  background-color: white;
  border-radius: 8px;
  border: 1px solid #d4cfc8;
  overflow-y: auto;
  padding: 15px;
}

.loading-state,
.empty-state {
  display: flex;
  align-items: center;
  justify-content: center;
  min-height: 200px;
  color: #cccccc;
  font-size: 14px;
}

.messages {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.message-group {
  animation: slideIn 0.3s ease;
}

@keyframes slideIn {
  from {
    opacity: 0;
    transform: translateY(10px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}
</style>
