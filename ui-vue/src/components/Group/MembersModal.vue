<template>
  <div class="modal-overlay" @click="$emit('close')">
    <div class="modal-content" @click.stop>
      <div class="modal-header">
        <h2>群成员</h2>
        <button class="close-btn" @click="$emit('close')">✕</button>
      </div>

      <div class="modal-body">
        <div class="group-info">
          <p class="group-name">{{ group.name }}</p>
          <p class="group-number">群号: {{ group.group_code || '未设置' }}</p>
        </div>

        <div class="toolbar">
          <select v-model="selectedOwnedBotId" class="toolbar-select" :disabled="loading">
            <option value="">请选择你的 Bot</option>
            <option v-for="bot in ownedBots" :key="bot.bot_id" :value="bot.bot_id">
              {{ bot.name }} ({{ bot.bot_id.slice(0, 8) }}...)
            </option>
          </select>
          <button class="toolbar-btn" :disabled="!canAddSelectedBot || loading" @click="handleAddBot">
            {{ loading ? '处理中...' : '加群' }}
          </button>
          <button class="secondary-btn" :disabled="!canRemoveSelectedBot || loading" @click="handleRemoveBot">
            {{ loading ? '处理中...' : '退群' }}
          </button>
        </div>

        <div v-if="members.length > 0" class="members-list">
          <div v-for="member in members" :key="member.member_id" class="member-item">
            <div class="member-avatar">🤖</div>
            <div class="member-info">
              <p class="member-name">{{ member.bot_name || member.member_type }}</p>
              <p class="member-id">ID: {{ member.member_id.substring(0, 8) }}...</p>
            </div>
            <p class="join-time">{{ formatDate(member.joined_at) }}</p>
          </div>
        </div>

        <div v-else class="empty-members">
          <p>暂无成员</p>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'
import type { Group, GroupMember, Bot } from '@/types'

const props = defineProps<{
  group: Group
  members: GroupMember[]
  ownedBots: Bot[]
  loading?: boolean
}>()

const emit = defineEmits<{
  'add-bot': [botId: string]
  'remove-bot': [botId: string]
  close: []
}>()

const selectedOwnedBotId = ref('')

const canAddSelectedBot = computed(() => {
  if (!selectedOwnedBotId.value) {
    return false
  }

  return !props.members.some((member) => member.member_id === selectedOwnedBotId.value)
})

const canRemoveSelectedBot = computed(() => {
  if (!selectedOwnedBotId.value) {
    return false
  }

  return props.members.some((member) => member.member_id === selectedOwnedBotId.value)
})

const handleAddBot = () => {
  if (!selectedOwnedBotId.value) {
    return
  }

  emit('add-bot', selectedOwnedBotId.value)
}

const handleRemoveBot = () => {
  if (!selectedOwnedBotId.value) {
    return
  }

  emit('remove-bot', selectedOwnedBotId.value)
}

const formatDate = (dateStr: string) => {
  const date = new Date(dateStr)
  return date.toLocaleDateString('zh-CN', {
    month: 'short',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
  })
}
</script>

<style scoped>
.modal-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background-color: rgba(0, 0, 0, 0.3);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
  padding: 20px;
}

.modal-content {
  background: white;
  border-radius: 8px;
  width: 100%;
  max-width: 560px;
  max-height: 70vh;
  overflow: hidden;
  display: flex;
  flex-direction: column;
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.15);
}

.modal-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 20px;
  border-bottom: 1px solid #d4cfc8;
}

.modal-header h2 {
  font-size: 18px;
  font-weight: 600;
  color: #4a4a4a;
  margin: 0;
}

.close-btn {
  background: none;
  border: none;
  font-size: 20px;
  cursor: pointer;
  color: #888888;
}

.modal-body {
  flex: 1;
  overflow-y: auto;
  padding: 20px;
}

.group-info {
  margin-bottom: 20px;
  padding-bottom: 20px;
  border-bottom: 1px solid #e8e3dd;
}

.group-name {
  font-size: 16px;
  font-weight: 600;
  color: #4a4a4a;
  margin: 0 0 4px 0;
}

.group-number {
  font-size: 12px;
  color: #888888;
  margin: 0;
  font-family: 'Monaco', 'Courier New', monospace;
}

.toolbar {
  display: grid;
  grid-template-columns: minmax(0, 1fr) auto auto;
  gap: 10px;
  margin-bottom: 16px;
}

.toolbar-select {
  width: 100%;
  padding: 10px 12px;
  border: 1px solid #d4cfc8;
  border-radius: 6px;
  font-size: 14px;
  color: #4a4a4a;
  background: white;
}

.toolbar-btn,
.secondary-btn {
  border: none;
  border-radius: 6px;
  cursor: pointer;
  transition: all 0.3s ease;
  padding: 10px 14px;
  font-size: 13px;
}

.toolbar-btn {
  background-color: #8b9d83;
  color: white;
}

.secondary-btn {
  background-color: #f5e6e6;
  color: #a88b7f;
}

.toolbar-btn:disabled,
.secondary-btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.members-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.member-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px;
  background-color: #fafaf8;
  border-radius: 6px;
}

.member-avatar {
  font-size: 24px;
  flex-shrink: 0;
}

.member-info {
  flex: 1;
  min-width: 0;
}

.member-name {
  font-size: 14px;
  font-weight: 500;
  color: #4a4a4a;
  margin: 0 0 2px 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.member-id {
  font-size: 11px;
  color: #cccccc;
  margin: 0;
  font-family: 'Monaco', 'Courier New', monospace;
}

.join-time {
  font-size: 11px;
  color: #cccccc;
  margin: 0;
  white-space: nowrap;
}

.empty-members {
  text-align: center;
  padding: 40px 20px;
  color: #cccccc;
}

.empty-members p {
  margin: 0;
  font-size: 14px;
}

@media (max-width: 768px) {
  .toolbar {
    grid-template-columns: 1fr;
  }
}
</style>
