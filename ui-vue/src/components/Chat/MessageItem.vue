<template>
  <div class="message-item">
    <button class="message-avatar avatar-button" @click="$emit('view-bot', message.sender_id)">
      <img v-if="message.sender_avatar_url" :src="message.sender_avatar_url" :alt="senderLabel" class="avatar-image" />
      <span v-else>{{ senderLabel.charAt(0) }}</span>
    </button>
    <div class="message-content">
      <div class="message-header">
        <span class="message-sender">{{ senderLabel }}</span>
        <span class="message-time">{{ formatTime(message.created_at) }}</span>
      </div>
      <div v-if="isFileMessage && fileUrl" class="message-file">
        <a :href="fileUrl" target="_blank" rel="noopener noreferrer">{{ fileName }}</a>
      </div>
      <div v-else class="message-text">{{ messageText }}</div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import type { Message } from '@/types'

const props = defineProps<{
  message: Message
}>()

defineEmits<{
  'view-bot': [botId: string]
}>()

const senderLabel = computed(() => props.message.sender_name || props.message.sender_id.slice(0, 8))

const fileUrl = computed(() => {
  const { content } = props.message
  if (typeof content === 'string') {
    return null
  }
  return typeof content.url === 'string' ? content.url : null
})

const isFileMessage = computed(() => props.message.msg_type === 'file' && !!fileUrl.value)

const fileName = computed(() => {
  const { content } = props.message
  if (typeof content !== 'string' && typeof content.filename === 'string' && content.filename.trim()) {
    return content.filename
  }

  if (!fileUrl.value) {
    return '文件'
  }

  const segment = fileUrl.value.split('/').pop() || ''
  if (!segment) {
    return '文件'
  }

  try {
    return decodeURIComponent(segment)
  } catch {
    return segment
  }
})

const messageText = computed(() => {
  const { content } = props.message

  if (typeof content === 'string') {
    return content
  }

  if (typeof content.text === 'string') {
    return content.text
  }

  return JSON.stringify(content)
})

const formatTime = (dateStr: string) => {
  const date = new Date(dateStr)
  const now = new Date()
  const diffMs = now.getTime() - date.getTime()
  const diffMins = Math.floor(diffMs / (1000 * 60))

  if (diffMins < 1) return '刚刚'
  if (diffMins < 60) return `${diffMins}分钟前`

  const diffHours = Math.floor(diffMins / 60)
  if (diffHours < 24) return `${diffHours}小时前`

  return date.toLocaleDateString('zh-CN')
}
</script>

<style scoped>
.message-item {
  display: flex;
  gap: 12px;
  animation: fadeIn 0.3s ease;
}

.message-avatar {
  width: 36px;
  height: 36px;
  border-radius: 50%;
  background-color: #0e3c2f;
  color: #ebf2ee;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 16px;
  font-weight: 600;
  flex-shrink: 0;
}

.avatar-button {
  border: none;
  cursor: pointer;
  padding: 0;
  overflow: hidden;
}

.avatar-image {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.message-content {
  flex: 1;
  min-width: 0;
}

.message-header {
  display: flex;
  align-items: baseline;
  gap: 8px;
  margin-bottom: 4px;
}

.message-sender {
  font-size: 13px;
  font-weight: 700;
  color: #143127;
}

.message-time {
  font-size: 11px;
  color: #8d9994;
}

.message-text {
  font-size: 14px;
  color: #23322c;
  line-height: 1.5;
  word-break: break-word;
  white-space: pre-wrap;
  background: #f3f8ee;
  border: 1px solid #dde7d8;
  padding: 8px 12px;
  border-radius: 10px;
}

.message-file {
  font-size: 14px;
  line-height: 1.5;
  background: #f3f8ee;
  border: 1px solid #dde7d8;
  padding: 8px 12px;
  border-radius: 10px;
}

.message-file a {
  color: #1f5d4a;
  text-decoration: none;
  font-weight: 700;
}

.message-file a:hover {
  text-decoration: underline;
}

@keyframes fadeIn {
  from {
    opacity: 0;
  }
  to {
    opacity: 1;
  }
}
</style>
