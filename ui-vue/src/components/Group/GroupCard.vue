<template>
  <div class="group-card">
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
  canDelete?: boolean
}>()

defineEmits<{
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
  background: #f5f5f5;
  border: 1px solid #d0d0d0;
  border-radius: 10px;
  padding: 18px;
  cursor: default;
  transition: var(--transition-base);
  box-shadow: 0 4px 10px rgba(0, 0, 0, 0.05);
}

.group-card:hover {
  border-color: #bbbbbb;
  box-shadow: 0 8px 16px rgba(0, 0, 0, 0.08);
  transform: translateY(-1px);
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
  color: #1f1f1f;
  margin: 0 0 4px 0;
}

.group-number {
  font-size: 12px;
  color: #5f5f5f;
  margin: 0;
  font-family: inherit;
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
  background-color: #ebebeb;
}

.view-btn:hover {
  color: #2f2f2f;
}

.delete-btn:hover {
  color: #a14139;
}

.card-footer {
  display: flex;
  gap: 15px;
  padding-top: 12px;
  border-top: 1px solid #d0d0d0;
}

.stat {
  flex: 1;
  text-align: center;
}

.stat-label {
  display: block;
  font-size: 11px;
  color: #7a7a7a;
  margin-bottom: 4px;
}

.stat-value {
  display: block;
  font-size: 14px;
  font-weight: 700;
  color: #2f2f2f;
}

</style>
