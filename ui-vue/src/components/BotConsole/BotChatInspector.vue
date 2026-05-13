<template>
  <div class="inspector-column">
    <!-- 连接状态 -->
    <BotConnectionStatus />

    <!-- 群组信息 -->
    <div class="info-panel">
      <h4 class="panel-title">群组信息</h4>
      <div class="info-rows">
        <div class="info-row">
          <span class="info-key">群组名称</span>
          <span class="info-value">{{ group?.name ?? '—' }}</span>
        </div>
        <div class="info-row">
          <span class="info-key">群组码</span>
          <span class="info-value">{{ group?.group_code ?? '—' }}</span>
        </div>
        <div class="info-row">
          <span class="info-key">创建时间</span>
          <span class="info-value">{{ formatDate(group?.created_at) }}</span>
        </div>
        <div class="info-row">
          <span class="info-key">成员数量</span>
          <span class="info-value">{{ store.selectedMemberCount }} 人</span>
        </div>
        <div class="info-row">
          <span class="info-key">群组公告</span>
          <span class="info-value announce">{{ group?.description ?? '暂无公告' }}</span>
        </div>
      </div>
    </div>

    <!-- 最近文件 -->
    <div class="info-panel">
      <h4 class="panel-title">最近文件</h4>
      <div class="file-list">
        <div
          v-for="file in store.visibleRecentFiles"
          :key="file.id"
          class="file-row"
        >
          <div class="file-icon-wrap">
            <FileText v-if="file.type === 'doc'" class="file-type-icon doc" />
            <FileText v-else-if="file.type === 'pdf'" class="file-type-icon pdf" />
            <FileCode2 v-else-if="file.type === 'code'" class="file-type-icon code" />
            <FileImage v-else-if="file.type === 'image'" class="file-type-icon image" />
            <File v-else class="file-type-icon" />
          </div>
          <div class="file-meta">
            <p class="file-name">{{ file.name }}</p>
            <p class="file-size">{{ file.size }}</p>
          </div>
          <div class="file-time">{{ file.time }}</div>
        </div>
        <div v-if="store.visibleRecentFiles.length === 0" class="empty-row">
          当前群聊暂无文件
        </div>
      </div>
      <button type="button" class="action-btn text" @click="emit('view-more-files')">
        查看更多
      </button>
    </div>

    <!-- Bot 信息 -->
    <div class="info-panel">
      <h4 class="panel-title">Bot 信息</h4>
      <div class="info-rows">
        <div class="info-row">
          <span class="info-key">当前 Bot</span>
          <span class="info-value">{{ store.activeBot?.name ?? '未选择' }}</span>
        </div>
        <div class="info-row">
          <span class="info-key">身份凭证</span>
          <button type="button" class="toggle-token-btn" @click="showToken = !showToken">
            <Eye v-if="!showToken" class="icon-xs" />
            <EyeOff v-else class="icon-xs" />
            {{ showToken ? '隐藏' : '查看' }}
          </button>
        </div>
        <div v-if="showToken" class="token-display">
          <code class="token-text">{{ store.botToken || '—' }}</code>
          <button type="button" class="copy-token-btn" @click="copyToken">
            <Copy v-if="!copied" class="icon-xs" />
            <Check v-else class="icon-xs" />
            {{ copied ? '已复制' : '复制' }}
          </button>
        </div>
        <div class="info-row">
          <span class="info-key">最近连接</span>
          <span class="info-value">{{ store.lastConnectedAt }}</span>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'
import { File, FileText, FileCode2, FileImage, Eye, EyeOff, Copy, Check } from 'lucide-vue-next'
import { useBotConsoleStore } from '@/stores/botConsole'
import BotConnectionStatus from './BotConnectionStatus.vue'

const store = useBotConsoleStore()
const showToken = ref(false)
const copied = ref(false)

const copyToken = async () => {
  if (!store.botToken) return
  try {
    await navigator.clipboard.writeText(store.botToken)
    copied.value = true
    setTimeout(() => { copied.value = false }, 2000)
  } catch {
    // fallback for older browsers
    const textarea = document.createElement('textarea')
    textarea.value = store.botToken
    textarea.style.position = 'fixed'
    textarea.style.opacity = '0'
    document.body.appendChild(textarea)
    textarea.select()
    document.execCommand('copy')
    document.body.removeChild(textarea)
    copied.value = true
    setTimeout(() => { copied.value = false }, 2000)
  }
}

const group = computed(() => store.selectedGroup)

const formatDate = (dateStr?: string) => {
  if (!dateStr) return '—'
  const d = new Date(dateStr)
  return d.toLocaleString('zh-CN', {
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit',
  })
}

const emit = defineEmits<{
  'view-more-files': []
}>()
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

.token-display {
  display: flex;
  flex-direction: column;
  gap: 8px;
  padding: 10px;
  background: #fff;
  border: 1px solid #d0d0d0;
  border-radius: 6px;
}

.token-text {
  font-family: ui-monospace, SFMono-Regular, 'SF Mono', Menlo, Consolas, monospace;
  font-size: 11px;
  color: #1f1f1f;
  word-break: break-all;
  line-height: 1.5;
  user-select: all;
  padding: 0;
  background: none;
}

.copy-token-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 4px;
  padding: 5px 10px;
  border: 1px solid #d0d0d0;
  border-radius: 5px;
  background: #f5f5f5;
  color: #4f4f4f;
  font-size: 11px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.15s ease;
}

.copy-token-btn:hover {
  background: #ebebeb;
  border-color: #c5c5c5;
}

.toggle-token-btn {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: 0;
  border: none;
  background: none;
  color: #6f6f6f;
  font-size: 12px;
  cursor: pointer;
}

.toggle-token-btn:hover {
  color: #1f1f1f;
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

.file-type-icon.doc {
  stroke: #2f5f8f;
}

.file-type-icon.pdf {
  stroke: #c4453c;
}

.file-type-icon.code {
  stroke: #2f8f4e;
}

.file-type-icon.image {
  stroke: #8f5f2f;
}

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

.action-btn {
  padding: 6px 10px;
  border-radius: 6px;
  font-size: 12px;
  font-weight: 600;
  cursor: pointer;
  transition: var(--transition-base, all 0.2s ease);
  text-align: center;
  border: 1px solid transparent;
  background: transparent;
  color: #6f6f6f;
}

.action-btn.text:hover {
  background: #ebebeb;
  color: #1f1f1f;
}

@media (max-width: 1200px) {
  .inspector-column {
    display: none;
  }
}
</style>
