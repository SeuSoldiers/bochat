<template>
  <section class="join-request-panel">
    <div class="panel-header">
      <h3>加群申请</h3>
      <span class="panel-meta">{{ inbox.length }}/{{ outbox.length }}</span>
    </div>

    <div class="panel-grid">
      <div class="panel-block">
        <p class="block-title">待我处理</p>
        <div v-if="inbox.length > 0" class="request-list">
          <div v-for="item in inbox" :key="item.request_id" class="request-item">
            <div class="request-main">
              <p class="request-title">{{ item.group_name }} · {{ item.bot_name }}</p>
              <p class="request-reason">理由：{{ item.request_reason }}</p>
            </div>
            <div class="request-actions">
              <button class="mini-btn approve" :disabled="loading" @click="$emit('approve', item.request_id)">同意</button>
              <button class="mini-btn reject" :disabled="loading" @click="$emit('reject', item.request_id)">拒绝</button>
            </div>
          </div>
        </div>
        <p v-else class="empty-tip">暂无待处理申请</p>
      </div>

      <div class="panel-block">
        <p class="block-title">我发起的申请</p>
        <div v-if="outbox.length > 0" class="request-list">
          <div v-for="item in outbox" :key="item.request_id" class="request-item readonly">
            <div class="request-main">
              <p class="request-title">{{ item.group_name }} · {{ item.bot_name }}</p>
              <p class="request-reason">理由：{{ item.request_reason }}</p>
            </div>
            <span class="status-pill">{{ item.status === 'pending' ? '待处理' : item.status }}</span>
          </div>
        </div>
        <p v-else class="empty-tip">暂无已发起申请</p>
      </div>
    </div>
  </section>
</template>

<script setup lang="ts">
import type { GroupJoinRequestItem } from '@/types'

defineProps<{
  inbox: GroupJoinRequestItem[]
  outbox: GroupJoinRequestItem[]
  loading?: boolean
}>()

defineEmits<{
  approve: [requestId: string]
  reject: [requestId: string]
}>()
</script>

<style scoped>
.join-request-panel {
  border: 1px solid #d0d0d0;
  border-radius: 10px;
  background: #f5f5f5;
  padding: 14px;
  margin-bottom: 14px;
}

.panel-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 10px;
}

.panel-header h3 {
  margin: 0;
  font-size: 14px;
  color: #2f2f2f;
}

.panel-meta {
  font-size: 12px;
  color: #6f6f6f;
}

.panel-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 10px;
}

.panel-block {
  border: 1px solid #d8d8d8;
  border-radius: 8px;
  padding: 10px;
  background: #f1f1f1;
  min-height: 80px;
}

.block-title {
  margin: 0 0 8px;
  font-size: 12px;
  color: #5f5f5f;
}

.request-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.request-item {
  border: 1px solid #d0d0d0;
  border-radius: 6px;
  background: #fafafa;
  padding: 8px;
  display: flex;
  justify-content: space-between;
  gap: 8px;
}

.request-main {
  min-width: 0;
}

.request-title {
  margin: 0;
  font-size: 13px;
  color: #2f2f2f;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.request-reason {
  margin: 4px 0 0;
  font-size: 11px;
  color: #666;
  line-height: 1.35;
}

.request-actions {
  display: flex;
  align-items: center;
  gap: 6px;
}

.mini-btn {
  height: 28px;
  border-radius: 6px;
  border: 1px solid #cfcfcf;
  background: #f4f4f4;
  color: #2f2f2f;
  font-size: 12px;
  padding: 0 10px;
  cursor: pointer;
}

.mini-btn.approve {
  border-color: #2f2f2f;
  background: #2f2f2f;
  color: #f3f3f3;
}

.mini-btn.reject {
  border-color: #cfb5b5;
  color: #8f3535;
}

.status-pill {
  align-self: flex-start;
  border-radius: 999px;
  background: #e5e5e5;
  color: #666;
  font-size: 11px;
  padding: 2px 8px;
}

.empty-tip {
  margin: 0;
  font-size: 12px;
  color: #8a8a8a;
}
</style>
