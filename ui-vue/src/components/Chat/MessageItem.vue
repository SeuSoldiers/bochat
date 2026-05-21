<template>
  <div class="message-item">
    <button class="message-avatar avatar-button" @click="$emit('view-bot', message.sender_id)">
      <img v-if="senderAvatarUrl" :src="senderAvatarUrl" :alt="senderLabel" class="avatar-image" />
      <span v-else>{{ senderLabel.charAt(0) }}</span>
    </button>
    <div class="message-content">
      <div class="message-header">
        <span class="message-sender">{{ senderLabel }}</span>
        <span class="message-time">{{ formatTime(message.created_at) }}</span>
      </div>
      <div v-if="isFileMessage && fileUrl" class="message-file">
        <a
          v-if="isImageFile"
          :href="fileUrl"
          target="_blank"
          rel="noopener noreferrer"
          class="image-file-link"
        >
          <img :src="fileUrl" :alt="fileName" class="image-preview" />
          <span class="file-name">{{ fileName }}</span>
        </a>
        <a v-else :href="fileUrl" target="_blank" rel="noopener noreferrer" class="file-link">
          <component :is="fileIcon" class="file-icon" aria-hidden="true" />
          <span class="file-name">{{ fileName }}</span>
        </a>
      </div>
      <div v-else class="message-text">{{ messageText }}</div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import type { Component } from 'vue'
import {
  File as IconFile,
  FileArchive as IconFileArchive,
  FileAudio as IconFileAudio,
  FileCode2 as IconFileCode,
  FileSpreadsheet as IconFileSheet,
  FileText as IconFileText,
  FileVideo as IconFileVideo,
} from 'lucide-vue-next'
import type { Message } from '@/types'

const props = defineProps<{
  message: Message
}>()

defineEmits<{
  'view-bot': [botId: string]
}>()

const senderLabel = computed(() => props.message.sender_name || props.message.sender_id.slice(0, 8))
const senderAvatarUrl = computed(() => props.message.sender_avatar_url || '')

const normalizedContent = computed(() => {
  const { content } = props.message
  if (typeof content !== 'string') {
    return content
  }

  const trimmed = content.trim()
  if ((trimmed.startsWith('{') && trimmed.endsWith('}')) || (trimmed.startsWith('[') && trimmed.endsWith(']'))) {
    try {
      const parsed = JSON.parse(trimmed)
      if (parsed && typeof parsed === 'object') {
        return parsed as Record<string, unknown>
      }
    } catch {
      // 保持原始内容
    }
  }

  return null
})

const fileUrl = computed(() => {
  const parsed = normalizedContent.value
  if (parsed && typeof parsed.url === 'string') {
    return parsed.url
  }
  if (parsed && typeof parsed.file_url === 'string') {
    return parsed.file_url
  }

  const { content } = props.message
  if (typeof content !== 'string' && typeof content.url === 'string') {
    return content.url
  }
  if (typeof content !== 'string' && typeof content.file_url === 'string') {
    return content.file_url
  }

  return null
})

const isFileMessage = computed(() => props.message.msg_type === 'file' && !!fileUrl.value)
const isImageFile = computed(() => /\.(png|jpe?g|gif|webp|bmp|svg|ico)$/i.test(fileName.value))

const fileName = computed(() => {
  const parsed = normalizedContent.value
  if (parsed && typeof parsed.filename === 'string' && parsed.filename.trim()) {
    return parsed.filename
  }
  if (parsed && typeof parsed.file_name === 'string' && parsed.file_name.trim()) {
    return parsed.file_name
  }

  const { content } = props.message
  if (typeof content !== 'string' && typeof content.filename === 'string' && content.filename.trim()) {
    return content.filename
  }
  if (typeof content !== 'string' && typeof content.file_name === 'string' && content.file_name.trim()) {
    return content.file_name
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

const fileIcon = computed<Component>(() => {
  const lower = fileName.value.toLowerCase()
  if (/\.(zip|rar|7z|tar|gz)$/.test(lower)) return IconFileArchive
  if (/\.(mp4|mov|mkv|avi|webm)$/.test(lower)) return IconFileVideo
  if (/\.(mp3|wav|flac|aac|ogg)$/.test(lower)) return IconFileAudio
  if (/\.(xls|xlsx|csv)$/.test(lower)) return IconFileSheet
  if (/\.(pdf|doc|docx|odt|rtf|ppt|pptx|key)$/.test(lower)) return IconFileText
  if (/\.(js|ts|tsx|jsx|py|rs|go|java|c|cpp|h|hpp|json|yaml|yml|toml|md)$/.test(lower)) return IconFileCode
  return IconFile
})

const messageText = computed(() => {
  const parsed = normalizedContent.value
  if (parsed && typeof parsed.text === 'string') {
    return parsed.text
  }

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
  background-color: #ebebeb;
  color: #2f2f2f;
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
  color: #2f2f2f;
}

.message-time {
  font-size: 11px;
  color: #7f7f7f;
}

.message-text {
  font-size: 14px;
  color: #2f2f2f;
  line-height: 1.5;
  word-break: break-word;
  white-space: pre-wrap;
  background: #f5f5f5;
  border: 1px solid #d0d0d0;
  padding: 8px 12px;
  border-radius: 10px;
}

.message-file {
  font-size: 14px;
  line-height: 1.5;
  background: #f5f5f5;
  border: 1px solid #d0d0d0;
  padding: 8px 12px;
  border-radius: 10px;
}

.message-file a {
  color: #2f2f2f;
  text-decoration: none;
}

.message-file a:hover {
  text-decoration: underline;
}

.file-link {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  font-weight: 700;
}

.file-icon {
  width: 18px;
  height: 18px;
  stroke: #3f3f3f;
  stroke-width: 1.8;
  flex-shrink: 0;
}

.file-name {
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.image-file-link {
  display: flex;
  flex-direction: column;
  gap: 6px;
  max-width: 260px;
}

.image-preview {
  max-width: 240px;
  max-height: 180px;
  border-radius: 8px;
  border: 1px solid #d0d0d0;
  object-fit: cover;
  display: block;
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
