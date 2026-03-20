<template>
  <div class="modal-overlay" @click="$emit('close')">
    <div class="modal-content" @click.stop>
      <div class="modal-header">
        <h2>群成员</h2>
        <button class="close-btn" @click="$emit('close')">✕</button>
      </div>

      <div class="modal-body">
        <div class="group-info">
          <p class="group-name">{{ group.group_name }}</p>
          <p class="group-number">群号: {{ group.group_number }}</p>
        </div>

        <div v-if="members.length > 0" class="members-list">
          <div v-for="member in members" :key="member.id" class="member-item">
            <div class="member-avatar">🤖</div>
            <div class="member-info">
              <p class="member-name">{{ member.name }}</p>
              <p class="member-id">ID: {{ member.bot_id.substring(0, 8) }}...</p>
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
import type { Group } from '@/types'

interface Member {
  id: string
  name: string
  bot_id: string
  joined_at: string
}

defineProps<{
  group: Group
  members: Member[]
}>()

defineEmits<{
  close: []
}>()

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
  max-width: 500px;
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
</style>
