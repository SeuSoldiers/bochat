<template>
  <div :class="['group-card', { selected }]" @click="$emit('select')">
    <div class="card-header">
      <div>
        <h3 class="group-name">{{ group.group_name }}</h3>
        <p class="group-number">群号: {{ group.group_number }}</p>
      </div>
      <div class="card-actions">
        <button class="action-btn view-btn" @click.stop="$emit('view-members')" title="查看成员">
          👥
        </button>
        <button class="action-btn delete-btn" @click.stop="$emit('delete')" title="删除">
          🗑️
        </button>
      </div>
    </div>

    <div class="card-footer">
      <div class="stat">
        <span class="stat-label">成员数</span>
        <span class="stat-value">{{ group.member_count }}</span>
      </div>
      <div class="stat">
        <span class="stat-label">创建时间</span>
        <span class="stat-value">{{ formatDate(group.created_at) }}</span>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import type { Group } from '@/types'

defineProps<{
  group: Group
  selected?: boolean
}>()

defineEmits<{
  select: []
  delete: []
  'view-members': []
}>()

const formatDate = (dateStr: string) => {
  const date = new Date(dateStr)
  return date.toLocaleDateString('zh-CN')
}
</script>

<style scoped>
.group-card {
  background: white;
  border: 1px solid #d4cfc8;
  border-radius: 8px;
  padding: 16px;
  cursor: pointer;
  transition: all 0.3s ease;
}

.group-card:hover {
  border-color: #8b9d83;
  box-shadow: 0 4px 12px rgba(139, 157, 131, 0.15);
  transform: translateY(-2px);
}

.group-card.selected {
  border-color: #8b9d83;
  background-color: #fafaf8;
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  margin-bottom: 12px;
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

.view-btn:hover {
  color: #8b9d83;
}

.delete-btn:hover {
  color: #a88b7f;
}

.card-footer {
  display: flex;
  gap: 15px;
  padding-top: 12px;
  border-top: 1px solid #e8e3dd;
}

.stat {
  flex: 1;
  text-align: center;
}

.stat-label {
  display: block;
  font-size: 11px;
  color: #cccccc;
  margin-bottom: 4px;
}

.stat-value {
  display: block;
  font-size: 14px;
  font-weight: 600;
  color: #8b9d83;
}
</style>
