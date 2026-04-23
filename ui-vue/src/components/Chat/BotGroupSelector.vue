<template>
  <div class="selector-container">
    <div class="selector-section">
      <h3>选择群</h3>
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
  background: linear-gradient(140deg, #ffffff 0%, #f8fbf6 100%);
  border-radius: 16px;
  border: 1px solid #d8dfd8;
  box-shadow: 0 10px 22px rgba(10, 36, 27, 0.08);
}

.selector-section h3 {
  font-size: 14px;
  font-weight: 700;
  color: #112f25;
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
  background-color: #f8fbf6;
  border: 1px solid #dce4dc;
  border-radius: 10px;
  cursor: pointer;
  font-size: 13px;
  color: #54655f;
  transition: var(--transition-base);
  text-align: left;
  max-width: 220px;
}

.item:hover {
  background-color: #edf5e5;
  border-color: #b9d87e;
}

.item.active {
  background-color: #0e3c2f;
  color: #ebf2ee;
  border-color: #1f5d4a;
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
  color: #8d9994;
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
