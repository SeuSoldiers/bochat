<template>
  <div :class="['group-card', { selected }]" @click="$emit('select')">
    <div class="card-header">
      <div>
        <h3 class="group-name">{{ group.name }}</h3>
        <p class="group-number">群号: {{ group.group_code || '未设置' }}</p>
      </div>
      <div class="card-actions">
        <button class="action-btn view-btn" @click.stop="$emit('view-members')" title="查看成员">
          👥
        </button>
        <button
          v-if="canDelete"
          class="action-btn delete-btn"
          @click.stop="$emit('delete')"
          title="删除"
        >
          🗑️
        </button>
      </div>
    </div>

    <div class="card-footer">
      <div class="stat">
        <span class="stat-label">状态</span>
        <span class="stat-value">{{ group.status }}</span>
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
  canDelete?: boolean
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
  background: linear-gradient(150deg, #ffffff 0%, #f8fbf6 100%);
  border: 1px solid #d8dfd8;
  border-radius: 16px;
  padding: 18px;
  cursor: pointer;
  transition: var(--transition-base);
  box-shadow: 0 10px 22px rgba(10, 36, 27, 0.08);
}

.group-card:hover {
  border-color: #a9d53c;
  box-shadow: 0 20px 34px rgba(10, 36, 27, 0.15);
  transform: translateY(-2px);
}

.group-card.selected {
  border-color: #0e3c2f;
  background: linear-gradient(130deg, #f2fbdf 0%, #f8fcf2 100%);
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  margin-bottom: 12px;
}

.group-name {
  font-size: 16px;
  font-weight: 700;
  color: #112f25;
  margin: 0 0 4px 0;
}

.group-number {
  font-size: 12px;
  color: #5f6d67;
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
  transition: var(--transition-base);
  border-radius: 8px;
  color: #3f504a;
}

.action-btn:hover {
  background-color: #eef6e4;
}

.view-btn:hover {
  color: #1f5d4a;
}

.delete-btn:hover {
  color: #a14139;
}

.card-footer {
  display: flex;
  gap: 15px;
  padding-top: 12px;
  border-top: 1px solid #e5ebe4;
}

.stat {
  flex: 1;
  text-align: center;
}

.stat-label {
  display: block;
  font-size: 11px;
  color: #87928c;
  margin-bottom: 4px;
}

.stat-value {
  display: block;
  font-size: 14px;
  font-weight: 700;
  color: #1a5b45;
}
</style>
