<template>
  <div class="selector-container">
    <div class="selector-section">
      <h3>选择 Bot</h3>
      <div v-if="bots.length > 0" class="items-list">
        <button
          v-for="bot in bots"
          :key="bot.bot_id"
          :class="['item', { active: selectedBot?.bot_id === bot.bot_id }]"
          @click="$emit('select-bot', bot.bot_id)"
        >
          <span class="item-icon">🤖</span>
          <span class="item-name">{{ bot.name }}</span>
        </button>
      </div>
      <p v-else class="empty-text">没有 Bot</p>
    </div>

    <div class="divider" />

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
          <span class="item-name">{{ group.group_name }}</span>
        </button>
      </div>
      <p v-else class="empty-text">没有群</p>
    </div>
  </div>
</template>

<script setup lang="ts">
import type { Bot, Group } from '@/types'

defineProps<{
  bots: Bot[]
  groups: Group[]
  selectedBot: Bot | null
  selectedGroup: Group | null
}>()

defineEmits<{
  'select-bot': [botId: string]
  'select-group': [groupId: string]
}>()
</script>

<style scoped>
.selector-container {
  display: grid;
  grid-template-columns: 1fr 1px 1fr;
  gap: 20px;
  padding: 15px;
  background-color: white;
  border-radius: 8px;
  border: 1px solid #d4cfc8;
}

.selector-section h3 {
  font-size: 14px;
  font-weight: 600;
  color: #4a4a4a;
  margin: 0 0 12px 0;
}

.items-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.item {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 10px 12px;
  background-color: #fafaf8;
  border: 1px solid #e8e3dd;
  border-radius: 6px;
  cursor: pointer;
  font-size: 13px;
  color: #888888;
  transition: all 0.3s ease;
  text-align: left;
}

.item:hover {
  background-color: #f5f3f1;
  border-color: #d4cfc8;
}

.item.active {
  background-color: #8b9d83;
  color: white;
  border-color: #8b9d83;
}

.item-icon {
  font-size: 16px;
}

.item-name {
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.divider {
  background-color: #e8e3dd;
  width: 1px;
}

.empty-text {
  font-size: 12px;
  color: #cccccc;
  margin: 0;
  padding: 10px 0;
}

@media (max-width: 768px) {
  .selector-container {
    grid-template-columns: 1fr;
  }

  .divider {
    display: none;
  }
}
</style>
