<template>
  <div class="group-card">
    <div class="card-header">
      <div class="group-main">
        <div class="group-avatar-wrap">
          <img v-if="group.avatar_url" :src="group.avatar_url" :alt="group.name" class="group-avatar" />
          <div v-else class="group-avatar fallback">{{ group.name.charAt(0) }}</div>
          <span :class="['group-avatar-status', { offline: !isGroupActive }]" aria-hidden="true"></span>
        </div>
        <div class="group-title-wrap">
          <h3 class="group-name" :title="group.name">{{ group.name }}</h3>
          <p v-if="group.description" class="group-description">{{ group.description }}</p>
          <p v-else class="group-description">暂无群简介</p>
        </div>
      </div>
      <div class="card-actions">
        <button class="action-btn view-btn" @click.stop="$emit('view-members')" aria-label="查看成员">
          <svg viewBox="0 0 24 24" fill="none" aria-hidden="true">
            <path d="M8.5 11.5a2.5 2.5 0 1 0 0-5 2.5 2.5 0 0 0 0 5Z" />
            <path d="M15.5 11a2 2 0 1 0 0-4 2 2 0 0 0 0 4Z" />
            <path d="M4.5 18a4 4 0 0 1 8 0" />
            <path d="M13 18a3.5 3.5 0 0 1 7 0" />
          </svg>
        </button>
        <button
          v-if="canEdit"
          class="action-btn edit-btn"
          @click.stop="$emit('edit')"
          aria-label="编辑"
        >
          <svg viewBox="0 0 24 24" fill="none" aria-hidden="true">
            <path d="m4 20 4.3-.8L19 8.6 15.4 5 4.8 15.6 4 20Z" />
            <path d="m13.8 6.6 3.6 3.6" />
          </svg>
        </button>
        <button
          v-if="canDelete"
          class="action-btn delete-btn"
          @click.stop="$emit('delete')"
          aria-label="删除"
        >
          <svg viewBox="0 0 24 24" fill="none" aria-hidden="true">
            <path d="M5 7h14M9 7V5.6A1.6 1.6 0 0 1 10.6 4h2.8A1.6 1.6 0 0 1 15 5.6V7" />
            <path d="M8 7v11a2 2 0 0 0 2 2h4a2 2 0 0 0 2-2V7M10.5 11v5M13.5 11v5" />
          </svg>
        </button>
      </div>
    </div>

    <div class="card-divider"></div>

    <div class="card-footer">
      <div class="meta-item">
        <div class="meta-label">
          <svg viewBox="0 0 24 24" fill="none" aria-hidden="true">
            <path d="M7.8 13.2 6.4 14.6a3.5 3.5 0 0 0 5 5l1.4-1.4" />
            <path d="m10.9 10.1 2.2-2.2a3.5 3.5 0 0 1 5 5l-2.2 2.2" />
            <path d="m9.8 14.2 4.4-4.4" />
          </svg>
          <span>群号</span>
        </div>
        <div class="meta-value">{{ group.group_code || '未设置' }}</div>
        <div class="meta-actions">
          <button
            class="mini-action icon-action"
            @click.stop="copyValue(group.group_code || '', '群号')"
            title="复制群号"
            aria-label="复制群号"
            :disabled="!group.group_code"
          >
            <svg viewBox="0 0 24 24" fill="none" aria-hidden="true">
              <rect x="9" y="9" width="10" height="10" rx="2" />
              <path d="M7 15H6a2 2 0 0 1-2-2V6a2 2 0 0 1 2-2h7a2 2 0 0 1 2 2v1" />
            </svg>
          </button>
        </div>
      </div>
      <div class="meta-item" :class="{ expanded: showGroupId }">
        <div class="meta-label">
          <svg viewBox="0 0 24 24" fill="none" aria-hidden="true">
            <rect x="4" y="4" width="16" height="16" rx="2.5" />
            <path d="M8 9h8M8 15h4M8 12h8" />
          </svg>
          <span>编号</span>
        </div>
        <div class="meta-value id-value" :class="{ expanded: showGroupId }">{{ displayGroupId }}</div>
        <div class="meta-actions">
          <button
            class="mini-action icon-action"
            @click.stop="showGroupId = !showGroupId"
            :title="showGroupId ? '隐藏编号' : '显示编号'"
            :aria-label="showGroupId ? '隐藏编号' : '显示编号'"
          >
            <svg v-if="showGroupId" viewBox="0 0 24 24" fill="none" aria-hidden="true">
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
            @click.stop="copyValue(group.group_id, '编号')"
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
    </div>

    <div v-if="copyTipText" class="copy-tip">{{ copyTipText }}</div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'
import type { Group } from '@/types'

const props = defineProps<{
  group: Group
  canEdit?: boolean
  canDelete?: boolean
}>()

defineEmits<{
  edit: []
  delete: []
  'view-members': []
}>()

const isGroupActive = ['active', 'running', 'enabled'].includes((props.group.status || '').toLowerCase())
const showGroupId = ref(false)
const copyTipText = ref('')
const displayGroupId = computed(() =>
  showGroupId.value || props.group.group_id.length <= 12
    ? props.group.group_id
    : `${props.group.group_id.slice(0, 6)}...${props.group.group_id.slice(-6)}`
)

const copyValue = async (value: string, label: string) => {
  if (!value) return
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
.group-card {
  background: #f5f5f5;
  border: 1px solid #d0d0d0;
  border-radius: 10px;
  padding: 24px 28px;
  transition: var(--transition-base);
  position: relative;
  box-shadow: 0 4px 10px rgba(0, 0, 0, 0.05);
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  gap: 16px;
  margin-bottom: 14px;
}

.group-main {
  display: flex;
  align-items: center;
  gap: 18px;
  min-width: 0;
  flex: 1;
}

.group-avatar-wrap {
  position: relative;
  flex-shrink: 0;
}

.group-avatar {
  width: 64px;
  height: 64px;
  border-radius: 50%;
  object-fit: cover;
  background: #ebebeb;
}

.fallback {
  display: flex;
  align-items: center;
  justify-content: center;
  color: #2f2f2f;
  font-size: 30px;
  font-weight: 700;
}

.group-title-wrap {
  min-width: 0;
}

.group-name {
  font-size: 24px;
  font-weight: 700;
  color: #1f1f1f;
  margin: 0;
  line-height: 1.1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.group-description {
  font-size: 14px;
  color: #5f5f5f;
  margin: 8px 0 0;
  line-height: 1.4;
  font-family: inherit;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
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

.delete-btn:hover {
  color: #a14139;
}

.delete-btn {
  color: #f0413e;
}

.card-divider {
  width: 100%;
  height: 1px;
  background: #d0d0d0;
  margin-bottom: 14px;
}

.card-footer {
  display: flex;
  flex-direction: column;
  gap: 8px;
  padding-top: 6px;
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
  line-height: 1.2;
  text-align: center;
  justify-self: center;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
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

.icon-action:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.group-avatar-status {
  position: absolute;
  right: 4px;
  bottom: 2px;
  width: 18px;
  height: 18px;
  border-radius: 50%;
  background: #2f8f4e;
}

.group-avatar-status.offline {
  background: #ba3b3b;
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
  .group-card {
    padding: 16px;
  }

  .card-header {
    flex-direction: column;
    align-items: stretch;
  }

  .group-avatar {
    width: 52px;
    height: 52px;
  }

  .group-avatar-status {
    width: 14px;
    height: 14px;
    right: 2px;
    bottom: 1px;
  }

  .fallback {
    font-size: 24px;
  }

  .group-name {
    font-size: 22px;
  }

  .group-description {
    font-size: 14px;
  }

  .action-btn {
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
