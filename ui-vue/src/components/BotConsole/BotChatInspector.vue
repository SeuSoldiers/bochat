<template>
  <div class="inspector-column">
    <!-- 连接状态 -->
    <BotConnectionStatus
      :status="connStatus"
      :label="connLabel"
      :base-url="connBaseUrl"
      :ws-url="connWsUrl"
      :identity="connIdentity"
      @reconnect="emit('reconnect')"
    />

    <!-- 群组信息 -->
    <div class="info-panel">
      <h4 class="panel-title">群组信息</h4>
      <div class="info-rows">
        <div class="info-row">
          <span class="info-key">群组名称</span>
          <span class="info-value">{{ groupName }}</span>
        </div>
        <div class="info-row">
          <span class="info-key">群组码</span>
          <span class="info-value">{{ groupCode }}</span>
        </div>
        <div class="info-row">
          <span class="info-key">创建时间</span>
          <span class="info-value">{{ groupCreatedAt }}</span>
        </div>
        <div v-if="memberCount !== null" class="info-row">
          <span class="info-key">成员数量</span>
          <span class="info-value">{{ memberCount }} 人</span>
        </div>
        <div class="info-row">
          <span class="info-key">群组公告</span>
          <span class="info-value announce">{{ groupDescription }}</span>
        </div>
      </div>
    </div>

    <!-- 最近文件 -->
    <div class="info-panel">
      <h4 class="panel-title">最近文件</h4>
      <div class="file-list">
        <div
          v-for="file in files"
          :key="file.id"
          class="file-row"
        >
          <div class="file-icon-wrap">
            <FileText v-if="file.type === 'doc'" class="file-type-icon doc" />
            <FileText v-else-if="file.type === 'pdf'" class="file-type-icon pdf" />
            <FileCode2 v-else-if="file.type === 'code'" class="file-type-icon code" />
            <FileImage v-else-if="file.type === 'image'" class="file-type-icon image" />
            <Binary v-else-if="file.type === 'bin'" class="file-type-icon bin" />
            <File v-else class="file-type-icon" />
          </div>
          <div class="file-meta">
            <p class="file-name">{{ file.name }}</p>
            <p class="file-size">{{ file.size || '未知大小' }}</p>
          </div>
          <div class="file-time">{{ file.time }}</div>
        </div>
        <div v-if="files.length === 0" class="empty-row">
          当前群聊暂无文件
        </div>
      </div>
    </div>

  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { File, FileText, FileCode2, FileImage, Binary } from 'lucide-vue-next'
import { useBotConsoleStore } from '@/stores/botConsole'
import BotConnectionStatus from './BotConnectionStatus.vue'
import type { RecentFile } from '@/stores/botConsole'

type ConnStatus = 'idle' | 'testing' | 'connected' | 'disconnected' | 'error'

const props = defineProps<{
  connStatus?: ConnStatus
  connLabel?: string
  connBaseUrl?: string
  connWsUrl?: string
  connIdentity?: string
  groupName?: string
  groupCode?: string
  groupCreatedAt?: string
  groupDescription?: string
  memberCount?: number | null
  files?: RecentFile[]
  botName?: string
  botToken?: string
  botStatus?: string
  lastConnected?: string
}>()

const emit = defineEmits<{
  reconnect: []
}>()

const adminStore = useBotConsoleStore()

const groupName = computed(() => props.groupName ?? adminStore.selectedGroup?.name ?? '-')
const groupCode = computed(() => props.groupCode ?? adminStore.selectedGroup?.group_code ?? '-')
const groupCreatedAt = computed(() => {
  if (props.groupCreatedAt) return props.groupCreatedAt
  const d = adminStore.selectedGroup?.created_at
  if (!d) return '-'
  return new Date(d).toLocaleString('zh-CN', {
    year: 'numeric', month: '2-digit', day: '2-digit',
    hour: '2-digit', minute: '2-digit',
  })
})
const groupDescription = computed(() => props.groupDescription ?? adminStore.selectedGroup?.description ?? '暂无公告')
const memberCount = computed(() => props.memberCount ?? (props.memberCount === undefined ? adminStore.selectedMemberCount : null))
const files = computed(() => props.files ?? adminStore.visibleRecentFiles ?? [])
</script>

<style scoped>
.inspector-column {
  width: 260px;
  flex: 0 0 260px;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 10px;
  min-height: 0;
  overflow-y: auto;
  padding-right: 2px;
}

.info-panel {
  padding: 14px;
  background: #f5f5f5;
  border-radius: 8px;
  border: 1px solid #d0d0d0;
  display: flex;
  flex-direction: column;
  gap: 10px;
  flex-shrink: 0;
}

.panel-title {
  margin: 0;
  font-size: 14px;
  font-weight: 700;
  color: #1f1f1f;
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

.info-value.announce {
  color: #6f6f6f;
}

.file-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.file-row {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px;
  border-radius: 6px;
  background: #f5f5f5;
  border: 1px solid transparent;
  transition: var(--transition-base, all 0.2s ease);
}

.file-row:hover {
  background: #efefef;
  border-color: #d0d0d0;
}

.file-icon-wrap {
  width: 32px;
  height: 32px;
  border-radius: 6px;
  display: grid;
  place-items: center;
  background: #ebebeb;
  flex-shrink: 0;
}

.file-type-icon {
  width: 16px;
  height: 16px;
  stroke: #4a4a4a;
  stroke-width: 2;
}

.file-type-icon.doc { stroke: #2f5f8f; }
.file-type-icon.pdf { stroke: #c4453c; }
.file-type-icon.code { stroke: #2f8f4e; }
.file-type-icon.image { stroke: #8f5f2f; }
.file-type-icon.bin { stroke: #8f3a2f; }

.file-meta {
  min-width: 0;
  flex: 1;
}

.file-name {
  margin: 0;
  font-size: 12px;
  color: #1f1f1f;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.file-size {
  margin: 2px 0 0;
  font-size: 11px;
  color: #6f6f6f;
}

.file-time {
  font-size: 11px;
  color: #9f9f9f;
  flex-shrink: 0;
}

.empty-row {
  padding: 10px 8px;
  text-align: center;
  font-size: 12px;
  color: #9f9f9f;
  border: 1px dashed #d0d0d0;
  border-radius: 6px;
}

@media (max-width: 1200px) {
  .inspector-column {
    display: none;
  }
}
</style>
