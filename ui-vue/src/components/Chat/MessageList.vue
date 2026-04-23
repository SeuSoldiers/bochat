<template>
  <div class="message-list">
    <div v-if="loading" class="loading-state">
      加载消息中...
    </div>

    <div v-else-if="messages.length > 0" class="messages">
      <div v-for="message in messages" :key="message.msg_id" class="message-group">
        <MessageItem :message="message" @view-bot="$emit('view-bot', $event)" />
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

defineEmits<{
  'view-bot': [botId: string]
}>()
</script>

<style scoped>
.message-list {
  flex: 1;
  display: flex;
  flex-direction: column;
  background: #f5f5f5;
  border-radius: 10px;
  border: 1px solid #d0d0d0;
  overflow: hidden;
  padding: 16px;
  min-height: 0;
  max-height: 100%;
  box-shadow: 0 4px 10px rgba(0, 0, 0, 0.05);
}

.loading-state,
.empty-state {
  display: flex;
  align-items: center;
  justify-content: center;
  min-height: 200px;
  color: #7f7f7f;
  font-size: 14px;
}

.messages {
  display: flex;
  flex-direction: column;
  gap: 12px;
  flex: 1;
  min-height: 0;
  overflow-y: auto;
  padding-right: 4px;
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
