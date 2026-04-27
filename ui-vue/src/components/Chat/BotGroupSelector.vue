<template>
  <div class="selector-container">
    <div class="selector-section">
      <div v-if="groups.length > 0" class="items-list">
        <button
          v-for="group in groups"
          :key="group.group_id"
          :class="['item', { active: selectedGroup?.group_id === group.group_id }]"
          @click="$emit('select-group', group.group_id)"
        >
          <span class="item-icon">💬</span>
          <span class="item-name">{{ group.name }}</span>
        </button>
      </div>
      <p v-else class="empty-text">没有群</p>
    </div>
  </div>
</template>

<script setup lang="ts">
import type { Group } from '@/types'

defineProps<{
  groups: Group[]
  selectedGroup: Group | null
}>()

defineEmits<{
  'select-group': [groupId: string]
}>()
</script>

<style scoped>
.selector-container {
  flex: 1;
  min-width: 0;
  padding: 16px;
  background: #f5f5f5;
  border-radius: 10px;
  border: 1px solid #d0d0d0;
  box-shadow: 0 4px 10px rgba(0, 0, 0, 0.05);
}

.selector-section h3 {
  font-size: 14px;
  font-weight: 700;
  color: #1f1f1f;
  margin: 0 0 10px 0;
}

.items-list {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}

.item {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 9px 12px;
  background-color: #f5f5f5;
  border: 1px solid #d0d0d0;
  border-radius: 8px;
  cursor: pointer;
  font-size: 13px;
  color: #4f4f4f;
  transition: var(--transition-base);
  text-align: left;
  max-width: 220px;
}

.item:hover {
  background-color: #ebebeb;
  border-color: #c5c5c5;
}

.item.active {
  background-color: #2f2f2f;
  color: #f3f3f3;
  border-color: #2f2f2f;
}

.item-icon {
  font-size: 16px;
}

.item-name {
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.empty-text {
  font-size: 12px;
  color: #7f7f7f;
  margin: 0;
  padding: 10px 0;
}

@media (max-width: 768px) {
  .item {
    max-width: none;
    width: 100%;
  }
}
</style>
