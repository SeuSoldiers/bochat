<template>
  <div class="modal-overlay" @click="$emit('close')">
    <div class="modal-content" @click.stop>
      <div class="modal-header">
        <h2>Bot 信息</h2>
        <button class="close-btn" @click="$emit('close')">✕</button>
      </div>

      <div v-if="loading" class="state">加载中...</div>
      <div v-else-if="bot" class="body">
        <div class="profile">
          <img v-if="bot.avatar_url" :src="bot.avatar_url" :alt="bot.name" class="avatar" />
          <div v-else class="avatar fallback">{{ bot.name.charAt(0) }}</div>
          <div>
            <h3>{{ bot.name }}</h3>
            <p>{{ bot.bot_id }}</p>
          </div>
        </div>

        <p v-if="bot.description" class="description">{{ bot.description }}</p>
      </div>
      <div v-else class="state">未找到 Bot 信息</div>
    </div>
  </div>
</template>

<script setup lang="ts">
import type { Bot } from '@/types'

defineProps<{
  bot: Bot | null
  loading?: boolean
}>()

defineEmits<{
  close: []
}>()
</script>

<style scoped>
.modal-overlay { position: fixed; inset: 0; background: rgba(0,0,0,.3); display: flex; align-items: center; justify-content: center; z-index: 1000; padding: 20px; }
.modal-content { background: white; border-radius: 8px; width: 100%; max-width: 420px; box-shadow: 0 4px 20px rgba(0,0,0,.15); }
.modal-header { display: flex; justify-content: space-between; align-items: center; padding: 20px; border-bottom: 1px solid #d4cfc8; }
.close-btn { border: none; background: none; font-size: 20px; cursor: pointer; }
.body,.state { padding: 20px; }
.profile { display: flex; gap: 16px; align-items: center; }
.avatar { width: 56px; height: 56px; border-radius: 50%; object-fit: cover; background: #f5f3f1; }
.fallback { display: flex; align-items: center; justify-content: center; font-size: 24px; color: white; background: #8b9d83; }
.description { margin-top: 16px; color: #4a4a4a; line-height: 1.6; }
</style>
