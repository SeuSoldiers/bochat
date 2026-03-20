<template>
  <div :class="['bot-card', { selected }]" @click="$emit('select')">
    <div class="card-header">
      <h3 class="bot-name">{{ bot.name }}</h3>
      <div class="card-actions">
        <button class="action-btn copy-btn" @click.stop="copyToken" title="复制 Token">
          📋
        </button>
        <button class="action-btn delete-btn" @click.stop="$emit('delete')" title="删除">
          🗑️
        </button>
      </div>
    </div>

    <p v-if="bot.description" class="bot-description">
      {{ bot.description }}
    </p>

    <div class="card-footer">
      <div class="meta-item">
        <span class="label">ID:</span>
        <span class="value">{{ bot.bot_id.substring(0, 8) }}...</span>
      </div>
      <div class="meta-item">
        <span class="label">创建时间:</span>
        <span class="value">{{ formatDate(bot.created_at) }}</span>
      </div>
    </div>

    <!-- 复制提示 -->
    <div v-if="showCopyTip" class="copy-tip">已复制!</div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import type { Bot } from '@/types'

defineProps<{
  bot: Bot
  selected?: boolean
}>()

defineEmits<{
  select: []
  delete: []
}>()

const showCopyTip = ref(false)

const formatDate = (dateStr: string) => {
  const date = new Date(dateStr)
  return date.toLocaleDateString('zh-CN')
}

const copyToken = async () => {
  try {
    // 在真实应用中，应该从 API 获取完整的 token
    await navigator.clipboard.writeText(import.meta.env.VITE_API_BASE_URL || 'localhost')
    showCopyTip.value = true
    setTimeout(() => {
      showCopyTip.value = false
    }, 2000)
  } catch (error) {
    console.error('Failed to copy token:', error)
  }
}
</script>

<style scoped>
.bot-card {
  background: white;
  border: 1px solid #d4cfc8;
  border-radius: 8px;
  padding: 16px;
  cursor: pointer;
  transition: all 0.3s ease;
  position: relative;
}

.bot-card:hover {
  border-color: #8b9d83;
  box-shadow: 0 4px 12px rgba(139, 157, 131, 0.15);
  transform: translateY(-2px);
}

.bot-card.selected {
  border-color: #8b9d83;
  background-color: #fafaf8;
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  margin-bottom: 12px;
}

.bot-name {
  font-size: 16px;
  font-weight: 600;
  color: #4a4a4a;
  margin: 0;
  flex: 1;
}

.card-actions {
  display: flex;
  gap: 6px;
}

.action-btn {
  padding: 4px 8px;
  background: none;
  border: none;
  cursor: pointer;
  font-size: 14px;
  transition: all 0.2s ease;
  border-radius: 4px;
}

.action-btn:hover {
  background-color: #f5f3f1;
}

.copy-btn:hover {
  color: #8b9d83;
}

.delete-btn:hover {
  color: #a88b7f;
}

.bot-description {
  font-size: 13px;
  color: #888888;
  margin-bottom: 12px;
  line-height: 1.5;
  overflow: hidden;
  text-overflow: ellipsis;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
}

.card-footer {
  display: flex;
  flex-direction: column;
  gap: 6px;
  padding-top: 12px;
  border-top: 1px solid #e8e3dd;
}

.meta-item {
  display: flex;
  justify-content: space-between;
  font-size: 12px;
}

.meta-item .label {
  color: #cccccc;
  font-weight: 500;
}

.meta-item .value {
  color: #888888;
  font-family: 'Monaco', 'Courier New', monospace;
}

.copy-tip {
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  background-color: #8b9d83;
  color: white;
  padding: 8px 16px;
  border-radius: 4px;
  font-size: 12px;
  animation: fadeInOut 2s ease;
}

@keyframes fadeInOut {
  0% {
    opacity: 0;
  }
  10% {
    opacity: 1;
  }
  90% {
    opacity: 1;
  }
  100% {
    opacity: 0;
  }
}
</style>
