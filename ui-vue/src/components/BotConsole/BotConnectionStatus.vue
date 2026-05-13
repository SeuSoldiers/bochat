<template>
  <div class="info-panel">
    <div class="panel-header">
      <h4 class="panel-title">连接状态</h4>
      <div class="conn-badge">
        <span class="status-dot-small" :class="statusClass"></span>
        <span class="status-label" :class="statusClass">{{ store.connectionLabel }}</span>
      </div>
    </div>

    <div class="info-rows">
      <div class="info-row">
        <span class="info-key">后端地址</span>
        <span class="info-value">{{ store.baseUrl || '未配置' }}</span>
      </div>
      <div class="info-row">
        <span class="info-key">WebSocket</span>
        <span class="info-value">{{ store.wsUrl || '未配置' }}</span>
      </div>
      <div class="info-row">
        <span class="info-key">连接时长</span>
        <span class="info-value">{{ store.connectionDuration }}</span>
      </div>
      <div class="info-row">
        <span class="info-key">延迟</span>
        <span class="info-value latency">
          <Signal class="latency-icon" />
          {{ store.connectionLatency }}ms
        </span>
      </div>
    </div>

    <button type="button" class="action-btn secondary" @click="store.reconnect">
      {{ store.connectionStatus === 'testing' ? '连接中...' : '重新连接' }}
    </button>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { Signal } from 'lucide-vue-next'
import { useBotConsoleStore } from '@/stores/botConsole'

const store = useBotConsoleStore()

const statusClass = computed(() => {
  switch (store.connectionStatus) {
    case 'connected':
      return 'online'
    case 'testing':
      return 'testing'
    case 'error':
    case 'disconnected':
      return 'offline'
    default:
      return 'idle'
  }
})

</script>

<style scoped>
.info-panel {
  padding: 14px;
  background: #f5f5f5;
  border-radius: 8px;
  border: 1px solid #d0d0d0;
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.panel-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
}

.panel-title {
  margin: 0;
  font-size: 14px;
  font-weight: 700;
  color: #1f1f1f;
}

.conn-badge {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  font-size: 12px;
  font-weight: 600;
}

.status-label.online {
  color: #2f8f4e;
}

.status-label.testing {
  color: #a57d11;
}

.status-label.offline {
  color: #c4453c;
}

.status-label.idle {
  color: #6f6f6f;
}

.status-dot-small {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: #9f9f9f;
  flex-shrink: 0;
}

.status-dot-small.online {
  background: #2f8f4e;
}

.status-dot-small.testing {
  background: #c9a227;
}

.status-dot-small.offline {
  background: #c4453c;
}

.info-rows {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.info-row {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
  gap: 8px;
  font-size: 12px;
}

.info-key {
  color: #6f6f6f;
  flex-shrink: 0;
}

.info-value {
  color: #1f1f1f;
  text-align: right;
  word-break: break-all;
  min-width: 0;
}

.latency {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  color: #2f8f4e;
  font-weight: 600;
}

.latency-icon {
  width: 14px;
  height: 14px;
  stroke: currentColor;
  stroke-width: 2;
}

.action-btn {
  padding: 8px 12px;
  border-radius: 8px;
  font-size: 12px;
  font-weight: 600;
  cursor: pointer;
  transition: var(--transition-base, all 0.2s ease);
  text-align: center;
}

.action-btn.secondary {
  background: #f5f5f5;
  color: #4f4f4f;
  border: 1px solid #d0d0d0;
}

.action-btn.secondary:hover {
  background: #ebebeb;
  border-color: #c5c5c5;
}
</style>
