<template>
  <div :class="['bot-card', { selected }]" @click="$emit('select')">
    <div class="card-header">
      <div class="bot-main">
        <img v-if="bot.avatar_url" :src="bot.avatar_url" :alt="bot.name" class="bot-avatar" />
        <div v-else class="bot-avatar fallback">{{ bot.name.charAt(0) }}</div>
        <h3 class="bot-name">{{ bot.name }}</h3>
      </div>
      <div class="card-actions">
        <button class="action-btn edit-btn" @click.stop="$emit('edit')" title="编辑">
          ✎
        </button>
        <button class="action-btn delete-btn" @click.stop="$emit('delete')" title="删除">
          删除
        </button>
      </div>
    </div>

    <p v-if="bot.description" class="bot-description">
      {{ bot.description }}
    </p>

    <div class="card-footer">
      <div class="meta-item">
        <span class="label">ID</span>
        <div class="meta-actions">
          <span class="value full-value">{{ bot.bot_id }}</span>
          <button class="mini-action" @click.stop="copyValue(bot.bot_id, 'ID')" title="复制 ID">
            复制
          </button>
        </div>
      </div>
      <div class="meta-item">
        <span class="label">Token</span>
        <div class="meta-actions">
          <span class="value full-value token-value">{{ displayToken }}</span>
          <button
            class="mini-action"
            @click.stop="showToken = !showToken"
            :title="showToken ? '隐藏 Token' : '显示 Token'"
          >
            {{ showToken ? '隐藏' : '显示' }}
          </button>
          <button class="mini-action" @click.stop="copyValue(bot.token, 'Token')" title="复制 Token">
            复制
          </button>
        </div>
      </div>
      <div class="meta-item">
        <span class="label">创建时间</span>
        <span class="value">{{ formatDate(bot.created_at) }}</span>
      </div>
    </div>

    <div v-if="copyTipText" class="copy-tip">{{ copyTipText }}</div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'
import type { Bot } from '@/types'

const props = defineProps<{
  bot: Bot
  selected?: boolean
}>()

defineEmits<{
  select: []
  edit: []
  delete: []
}>()

const showToken = ref(false)
const copyTipText = ref('')

const displayToken = computed(() => (showToken.value ? props.bot.token : maskToken(props.bot.token)))

const formatDate = (dateStr: string) => {
  const date = new Date(dateStr)
  return date.toLocaleDateString('zh-CN')
}

const maskToken = (token: string) => {
  if (!token) return '***'
  if (token.length <= 6) return '***'
  return `${token.slice(0, 3)}***${token.slice(-3)}`
}

const copyValue = async (value: string, label: string) => {
  try {
    await navigator.clipboard.writeText(value)
    copyTipText.value = `${label} 已复制`
    setTimeout(() => {
      copyTipText.value = ''
    }, 2000)
  } catch (error) {
    console.error(`Failed to copy ${label}:`, error)
  }
}
</script>

<style scoped>
.bot-card {
  background: white;
  border: 1px solid #d4cfc8;
  border-radius: 12px;
  padding: 22px;
  cursor: pointer;
  transition: all 0.3s ease;
  position: relative;
  min-height: 220px;
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
  gap: 16px;
  margin-bottom: 16px;
}

.bot-main {
  display: flex;
  align-items: center;
  gap: 14px;
  min-width: 0;
}

.bot-avatar {
  width: 52px;
  height: 52px;
  border-radius: 50%;
  object-fit: cover;
  background: #f5f3f1;
  flex-shrink: 0;
}

.fallback {
  display: flex;
  align-items: center;
  justify-content: center;
  color: white;
  background: #8b9d83;
  font-weight: 600;
}

.bot-name {
  font-size: 18px;
  font-weight: 600;
  color: #4a4a4a;
  margin: 0;
  flex: 1;
  line-height: 1.3;
}

.card-actions {
  display: flex;
  gap: 8px;
  align-items: center;
  flex-shrink: 0;
}

.action-btn {
  padding: 7px 12px;
  background: #f7f3ef;
  border: 1px solid #e6ddd4;
  cursor: pointer;
  font-size: 12px;
  transition: all 0.2s ease;
  border-radius: 999px;
  color: #5d574f;
}

.action-btn:hover {
  background-color: #efe7de;
}

.edit-btn:hover {
  color: #7a8c72;
}

.delete-btn:hover {
  color: #9e5647;
  border-color: #d8b3aa;
  background: #fbefec;
}

.bot-description {
  font-size: 14px;
  color: #888888;
  margin-bottom: 16px;
  line-height: 1.6;
  overflow: hidden;
  text-overflow: ellipsis;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  min-height: 44px;
}

.card-footer {
  display: flex;
  flex-direction: column;
  gap: 10px;
  padding-top: 16px;
  border-top: 1px solid #e8e3dd;
}

.meta-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 12px;
  font-size: 13px;
}

.meta-item .label {
  color: #cccccc;
  font-weight: 500;
  flex-shrink: 0;
}

.meta-item .value {
  color: #888888;
  font-family: 'Monaco', 'Courier New', monospace;
}

.meta-actions {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: 6px;
  min-width: 0;
  flex: 1;
}

.full-value {
  min-width: 0;
  text-align: right;
  overflow-wrap: anywhere;
  word-break: break-word;
}

.token-value {
  max-width: 210px;
}

.mini-action {
  padding: 4px 9px;
  border: 1px solid #ddd4ca;
  border-radius: 999px;
  background: white;
  color: #746c62;
  font-size: 12px;
  cursor: pointer;
  transition: all 0.2s ease;
}

.mini-action:hover {
  border-color: #b9aea1;
  background: #f8f4ef;
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
