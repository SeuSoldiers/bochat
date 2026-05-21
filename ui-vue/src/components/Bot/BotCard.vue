<template>
  <div :class="['bot-card', { selected }]">
    <div class="card-header">
      <div class="bot-main">
        <div class="avatar-wrap">
          <img v-if="bot.avatar_url" :src="bot.avatar_url" :alt="bot.name" class="bot-avatar" />
          <div v-else class="bot-avatar fallback">{{ bot.name.charAt(0) }}</div>
          <span :class="['avatar-status', { offline: !isBotRunning }]" aria-hidden="true"></span>
        </div>
        <div class="bot-title-wrap">
          <h3 class="bot-name" :title="bot.name">{{ displayBotName }}</h3>
          <p v-if="bot.description" class="bot-description">{{ bot.description }}</p>
          <p v-else class="bot-description">用户注册时自动创建的默认Bot</p>
        </div>
      </div>
      <div class="card-actions">
        <button class="action-btn join-btn" @click.stop="$emit('join-group')">
          <svg viewBox="0 0 24 24" fill="none" aria-hidden="true">
            <path d="M8 12h8" />
            <path d="M12 8v8" />
            <path d="M5 5.5h14A1.5 1.5 0 0 1 20.5 7v10a1.5 1.5 0 0 1-1.5 1.5H5A1.5 1.5 0 0 1 3.5 17V7A1.5 1.5 0 0 1 5 5.5Z" />
          </svg>
        </button>
        <button class="action-btn edit-btn" @click.stop="$emit('edit')">
          <svg viewBox="0 0 24 24" fill="none" aria-hidden="true">
            <path d="m4 20 4.3-.8L19 8.6 15.4 5 4.8 15.6 4 20Z" />
            <path d="m13.8 6.6 3.6 3.6" />
          </svg>
        </button>
        <button class="action-btn delete-btn" @click.stop="$emit('delete')">
          <svg viewBox="0 0 24 24" fill="none" aria-hidden="true">
            <path d="M5 7h14M9 7V5.6A1.6 1.6 0 0 1 10.6 4h2.8A1.6 1.6 0 0 1 15 5.6V7" />
            <path d="M8 7v11a2 2 0 0 0 2 2h4a2 2 0 0 0 2-2V7M10.5 11v5M13.5 11v5" />
          </svg>
        </button>
      </div>
    </div>

    <div class="card-divider"></div>

    <div class="card-footer">
      <div class="meta-item" :class="{ expanded: showBotId }">
        <div class="meta-label">
          <svg viewBox="0 0 24 24" fill="none" aria-hidden="true">
            <rect x="4" y="4" width="16" height="16" rx="2.5" />
            <path d="M8 9h8M8 15h4M8 12h8" />
          </svg>
          <span>编号</span>
        </div>
        <div class="meta-value id-value" :class="{ expanded: showBotId }">{{ displayBotId }}</div>
        <div class="meta-actions">
          <button
            class="mini-action icon-action"
            @click.stop="showBotId = !showBotId"
            :title="showBotId ? '隐藏编号' : '显示编号'"
            :aria-label="showBotId ? '隐藏编号' : '显示编号'"
          >
            <svg v-if="showBotId" viewBox="0 0 24 24" fill="none" aria-hidden="true">
              <path d="m3 3 18 18" />
              <path d="M10.6 10.7a2 2 0 0 0 2.8 2.8" />
              <path d="M9.4 5.2A10.8 10.8 0 0 1 12 5c5.5 0 9.4 4.6 10 7-.3 1.2-1.5 3.1-3.5 4.7" />
              <path d="M6.1 8.2C4 9.9 2.7 11.9 2.3 13c.6 2.4 4.5 7 9.9 7a10.5 10.5 0 0 0 3.4-.5" />
            </svg>
            <svg v-else viewBox="0 0 24 24" fill="none" aria-hidden="true">
              <path d="M2.3 12c.6-2.4 4.5-7 9.7-7s9.1 4.6 9.7 7c-.6 2.4-4.5 7-9.7 7s-9.1-4.6-9.7-7Z" />
              <circle cx="12" cy="12" r="3" />
            </svg>
          </button>
          <button
            class="mini-action icon-action"
            @click.stop="copyValue(bot.bot_id, '编号')"
            title="复制编号"
            aria-label="复制编号"
          >
            <svg viewBox="0 0 24 24" fill="none" aria-hidden="true">
              <rect x="9" y="9" width="10" height="10" rx="2" />
              <path d="M7 15H6a2 2 0 0 1-2-2V6a2 2 0 0 1 2-2h7a2 2 0 0 1 2 2v1" />
            </svg>
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
        <div class="meta-value token-value" :class="{ expanded: showToken }">{{ displayToken }}</div>
        <div class="meta-actions">
          <button
            class="mini-action icon-action"
            @click.stop="showToken = !showToken"
            :title="showToken ? '隐藏令牌' : '显示令牌'"
            :aria-label="showToken ? '隐藏令牌' : '显示令牌'"
          >
            <svg v-if="showToken" viewBox="0 0 24 24" fill="none" aria-hidden="true">
              <path d="m3 3 18 18" />
              <path d="M10.6 10.7a2 2 0 0 0 2.8 2.8" />
              <path d="M9.4 5.2A10.8 10.8 0 0 1 12 5c5.5 0 9.4 4.6 10 7-.3 1.2-1.5 3.1-3.5 4.7" />
              <path d="M6.1 8.2C4 9.9 2.7 11.9 2.3 13c.6 2.4 4.5 7 9.9 7a10.5 10.5 0 0 0 3.4-.5" />
            </svg>
            <svg v-else viewBox="0 0 24 24" fill="none" aria-hidden="true">
              <path d="M2.3 12c.6-2.4 4.5-7 9.7-7s9.1 4.6 9.7 7c-.6 2.4-4.5 7-9.7 7s-9.1-4.6-9.7-7Z" />
              <circle cx="12" cy="12" r="3" />
            </svg>
          </button>
          <button
            class="mini-action icon-action"
            @click.stop="copyValue(bot.token, '令牌')"
            title="复制令牌"
            aria-label="复制令牌"
          >
            <svg viewBox="0 0 24 24" fill="none" aria-hidden="true">
              <rect x="9" y="9" width="10" height="10" rx="2" />
              <path d="M7 15H6a2 2 0 0 1-2-2V6a2 2 0 0 1 2-2h7a2 2 0 0 1 2 2v1" />
            </svg>
          </button>
        </div>
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
  'join-group': []
  delete: []
}>()

const showToken = ref(false)
const showBotId = ref(false)
const copyTipText = ref('')

const displayBotId = computed(() => (showBotId.value ? props.bot.bot_id : maskSecret(props.bot.bot_id)))
const displayToken = computed(() => (showToken.value ? props.bot.token : maskToken(props.bot.token)))
const displayBotName = computed(() =>
  props.bot.name.length > 5 ? `${props.bot.name.slice(0, 5)}...` : props.bot.name
)
const disabledStatuses = new Set(['disabled', 'stopped', 'paused', 'inactive'])
const isBotRunning = computed(() => !disabledStatuses.has((props.bot.status || '').toLowerCase()))

const maskToken = (token: string) => {
  return maskSecret(token)
}

const maskSecret = (value: string) => {
  if (!value) return '***'
  if (value.length <= 12) return '***'
  return `${value.slice(0, 6)}...${value.slice(-6)}`
}

const copyValue = async (value: string, label: string) => {
  try {
    if (navigator.clipboard) {
      await navigator.clipboard.writeText(value)
    } else {
      const textarea = document.createElement('textarea')
      textarea.value = value
      textarea.style.position = 'fixed'
      textarea.style.opacity = '0'
      document.body.appendChild(textarea)
      textarea.select()
      document.execCommand('copy')
      document.body.removeChild(textarea)
    }
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
  background: #f5f5f5;
  border: 1px solid #d0d0d0;
  border-radius: 10px;
  padding: 24px 28px;
  transition: var(--transition-base);
  position: relative;
  box-shadow: 0 4px 10px rgba(0, 0, 0, 0.05);
}

.bot-card.selected {
  border-color: #2f2f2f;
  background: #2f2f2f;
  color: #f3f3f3;
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
  width: 64px;
  height: 64px;
  border-radius: 50%;
  object-fit: cover;
  background: #ebebeb;
  flex-shrink: 0;
}

.avatar-status {
  position: absolute;
  right: 4px;
  bottom: 2px;
  width: 18px;
  height: 18px;
  border-radius: 50%;
  background: #2f8f4e;
}

.avatar-status.offline {
  background: #ba3b3b;
}

.fallback {
  display: flex;
  align-items: center;
  justify-content: center;
  color: #2f2f2f;
  background: #ebebeb;
  font-size: 30px;
  font-weight: 700;
}

.bot-name {
  font-size: 24px;
  font-weight: 700;
  color: #1f1f1f;
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
  justify-content: center;
  width: 40px;
  height: 40px;
  padding: 0;
  background: #f5f5f5;
  border: 1px solid #d0d0d0;
  cursor: pointer;
  transition: all 0.2s ease;
  border-radius: 8px;
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

.join-btn {
  color: #2f2f2f;
}

.bot-description {
  font-size: 14px;
  color: #5f5f5f;
  margin-top: 8px;
  line-height: 1.4;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.card-footer {
  display: flex;
  flex-direction: column;
  gap: 8px;
  padding-top: 6px;
}

.card-divider {
  width: 100%;
  height: 1px;
  background: #d0d0d0;
  margin-bottom: 14px;
}

.meta-item {
  display: grid;
  grid-template-columns: max-content minmax(0, 1fr) max-content;
  align-items: center;
  gap: 8px;
  min-height: 46px;
}

.meta-item.expanded {
  align-items: flex-start;
}

.meta-label {
  display: inline-flex;
  align-items: center;
  gap: 10px;
  color: #696969;
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
  color: #4f4f4f;
  font-family: inherit;
  font-size: 16px;
  text-align: center;
  justify-self: center;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.token-value.expanded {
  white-space: normal;
  overflow: visible;
  text-overflow: clip;
  overflow-wrap: anywhere;
  word-break: break-all;
}

.id-value.expanded {
  white-space: normal;
  overflow: visible;
  text-overflow: clip;
  overflow-wrap: anywhere;
  word-break: break-all;
}

.meta-actions {
  min-width: 0;
  width: auto;
  display: inline-flex;
  justify-content: flex-end;
  gap: 10px;
  justify-self: end;
}

.mini-action {
  min-width: 88px;
  padding: 10px 16px;
  border: 1px solid #d0d0d0;
  border-radius: 8px;
  background: #f5f5f5;
  color: #4f5d58;
  font-size: 14px;
  font-weight: 700;
  cursor: pointer;
  transition: all 0.2s ease;
}

.bot-card.selected .bot-name,
.bot-card.selected .bot-description,
.bot-card.selected .meta-label,
.bot-card.selected .meta-value {
  color: #f3f3f3;
}

.bot-card.selected .card-divider {
  background: rgba(243, 243, 243, 0.28);
}

.bot-card.selected .mini-action,
.bot-card.selected .action-btn {
  background: #3a3a3a;
  border-color: #4a4a4a;
  color: #f3f3f3;
}

.icon-action {
  min-width: 40px;
  width: 40px;
  height: 40px;
  padding: 0;
  display: grid;
  place-items: center;
}

.icon-action svg {
  width: 18px;
  height: 18px;
  stroke: currentColor;
  stroke-width: 1.8;
  stroke-linecap: round;
  stroke-linejoin: round;
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
    width: 52px;
    height: 52px;
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
