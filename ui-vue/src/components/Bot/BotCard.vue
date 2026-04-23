<template>
  <div :class="['bot-card', { selected }]">
    <div class="card-header">
      <div class="bot-main">
        <div class="avatar-wrap">
          <img v-if="bot.avatar_url" :src="bot.avatar_url" :alt="bot.name" class="bot-avatar" />
          <div v-else class="bot-avatar fallback">{{ bot.name.charAt(0) }}</div>
          <span class="avatar-status"></span>
        </div>
        <div class="bot-title-wrap">
          <h3 class="bot-name">{{ bot.name }}</h3>
          <p v-if="bot.description" class="bot-description">{{ bot.description }}</p>
          <p v-else class="bot-description">用户注册时自动创建的默认Bot</p>
        </div>
      </div>
      <div class="card-actions">
        <button class="action-btn edit-btn" @click.stop="$emit('edit')" title="编辑">
          <svg viewBox="0 0 24 24" fill="none" aria-hidden="true">
            <path d="m4 20 4.3-.8L19 8.6 15.4 5 4.8 15.6 4 20Z" />
            <path d="m13.8 6.6 3.6 3.6" />
          </svg>
          编辑
        </button>
        <button class="action-btn delete-btn" @click.stop="$emit('delete')" title="删除">
          <svg viewBox="0 0 24 24" fill="none" aria-hidden="true">
            <path d="M5 7h14M9 7V5.6A1.6 1.6 0 0 1 10.6 4h2.8A1.6 1.6 0 0 1 15 5.6V7" />
            <path d="M8 7v11a2 2 0 0 0 2 2h4a2 2 0 0 0 2-2V7M10.5 11v5M13.5 11v5" />
          </svg>
          删除
        </button>
      </div>
    </div>

    <div class="card-divider"></div>

    <div class="card-footer">
      <div class="meta-item">
        <div class="meta-label">
          <svg viewBox="0 0 24 24" fill="none" aria-hidden="true">
            <rect x="4" y="4" width="16" height="16" rx="2.5" />
            <path d="M8 9h8M8 15h4M8 12h8" />
          </svg>
          <span>编号</span>
        </div>
        <div class="meta-value">{{ bot.bot_id }}</div>
        <div class="meta-actions">
          <button class="mini-action" @click.stop="copyValue(bot.bot_id, '编号')" title="复制编号">
            复制
          </button>
        </div>
      </div>

      <div class="meta-item">
        <div class="meta-label">
          <svg viewBox="0 0 24 24" fill="none" aria-hidden="true">
            <path d="M7.8 13.2 6.4 14.6a3.5 3.5 0 0 0 5 5l1.4-1.4" />
            <path d="m10.9 10.1 2.2-2.2a3.5 3.5 0 0 1 5 5l-2.2 2.2" />
            <path d="m9.8 14.2 4.4-4.4" />
          </svg>
          <span>令牌</span>
        </div>
        <div class="meta-value">{{ displayToken }}</div>
        <div class="meta-actions">
          <button
            class="mini-action"
            @click.stop="showToken = !showToken"
            :title="showToken ? '隐藏令牌' : '显示令牌'"
          >
            {{ showToken ? '隐藏' : '显示' }}
          </button>
          <button class="mini-action" @click.stop="copyValue(bot.token, '令牌')" title="复制令牌">
            复制
          </button>
        </div>
      </div>

      <div class="meta-item">
        <div class="meta-label">
          <svg viewBox="0 0 24 24" fill="none" aria-hidden="true">
            <rect x="4" y="5" width="16" height="15" rx="2.5" />
            <path d="M8 3v4M16 3v4M4 10h16" />
          </svg>
          <span>创建时间</span>
        </div>
        <div class="meta-value">{{ formatDate(bot.created_at) }}</div>
        <div class="meta-actions"></div>
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
  return date.toLocaleString('zh-CN', {
    hour12: false,
  })
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
  background: #f3f5f4;
  border: 1px solid #d2d8d3;
  border-radius: 22px;
  padding: 24px 28px;
  transition: var(--transition-base);
  position: relative;
  box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.92), 0 4px 10px rgba(10, 36, 27, 0.05);
}

.bot-card.selected {
  border-color: #0e3c2f;
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  gap: 16px;
  margin-bottom: 14px;
}

.bot-main {
  display: flex;
  align-items: center;
  gap: 18px;
  min-width: 0;
  flex: 1;
}

.avatar-wrap {
  position: relative;
}

.bot-avatar {
  width: 72px;
  height: 72px;
  border-radius: 50%;
  object-fit: cover;
  background: #edf2ea;
  flex-shrink: 0;
}

.avatar-status {
  position: absolute;
  right: 4px;
  bottom: 2px;
  width: 18px;
  height: 18px;
  border-radius: 50%;
  background: #12bf78;
  border: 4px solid #f3f5f4;
}

.fallback {
  display: flex;
  align-items: center;
  justify-content: center;
  color: white;
  background: #074b3b;
  font-size: 30px;
  font-weight: 700;
}

.bot-name {
  font-size: 36px;
  font-weight: 700;
  color: #112f25;
  margin: 0;
  line-height: 1.1;
}

.bot-title-wrap {
  min-width: 0;
}

.card-actions {
  display: flex;
  gap: 10px;
  align-items: center;
  flex-shrink: 0;
  margin-left: auto;
}

.action-btn {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  padding: 10px 18px;
  background: #f5f7f5;
  border: 1px solid #ced4ce;
  cursor: pointer;
  font-size: 14px;
  font-weight: 700;
  transition: all 0.2s ease;
  border-radius: 16px;
  color: #1a2c25;
}

.action-btn svg {
  width: 20px;
  height: 20px;
  stroke: currentColor;
  stroke-width: 1.8;
  stroke-linecap: round;
  stroke-linejoin: round;
}

.delete-btn {
  color: #f0413e;
}

.bot-description {
  font-size: 14px;
  color: #5f6d67;
  margin-top: 8px;
  line-height: 1.4;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.card-footer {
  display: flex;
  flex-direction: column;
  gap: 12px;
  padding-top: 10px;
}

.card-divider {
  width: 100%;
  height: 1px;
  background: #d5dbd5;
  margin-bottom: 14px;
}

.meta-item {
  display: grid;
  grid-template-columns: 140px 1fr auto;
  align-items: center;
  gap: 14px;
  min-height: 58px;
}

.meta-label {
  display: inline-flex;
  align-items: center;
  gap: 10px;
  color: #707a75;
  font-size: 14px;
}

.meta-label svg {
  width: 24px;
  height: 24px;
  stroke: currentColor;
  stroke-width: 1.8;
  stroke-linecap: round;
  stroke-linejoin: round;
}

.meta-value {
  color: #5f6762;
  font-family: 'Monaco', 'Courier New', monospace;
  font-size: 16px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.meta-actions {
  min-width: 180px;
  display: inline-flex;
  justify-content: flex-end;
  gap: 10px;
}

.mini-action {
  min-width: 88px;
  padding: 10px 16px;
  border: 1px solid #ced4ce;
  border-radius: 14px;
  background: #f5f7f5;
  color: #4f5d58;
  font-size: 14px;
  font-weight: 700;
  cursor: pointer;
  transition: all 0.2s ease;
}

.copy-tip {
  position: absolute;
  top: 24px;
  right: 24px;
  background-color: #0e3c2f;
  color: #ebf2ee;
  padding: 8px 14px;
  border-radius: 8px;
  font-size: 12px;
  animation: fadeInOut 2s ease;
}

@media (max-width: 768px) {
  .bot-card {
    padding: 16px;
  }

  .card-header {
    flex-direction: column;
    align-items: stretch;
  }

  .bot-avatar {
    width: 56px;
    height: 56px;
  }

  .avatar-status {
    width: 14px;
    height: 14px;
    border-width: 2px;
    right: 2px;
    bottom: 1px;
  }

  .fallback {
    font-size: 24px;
  }

  .bot-name {
    font-size: 22px;
  }

  .bot-description {
    font-size: 14px;
  }

  .action-btn {
    font-size: 14px;
    border-radius: 10px;
    padding: 8px 10px;
  }

  .meta-item {
    grid-template-columns: 1fr;
    gap: 8px;
    padding: 8px 0;
  }

  .meta-label {
    font-size: 14px;
  }

  .meta-value {
    font-size: 13px;
    white-space: normal;
    word-break: break-all;
  }

  .meta-actions {
    min-width: 0;
    justify-content: flex-start;
  }

  .mini-action {
    font-size: 12px;
    min-width: 66px;
    padding: 6px 10px;
    border-radius: 10px;
  }
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
