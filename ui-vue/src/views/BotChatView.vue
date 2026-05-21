<template>
  <div class="bot-chat-page">
    <header class="chat-topbar">
      <div class="topbar-left">
        <div class="bot-identity">
          <div class="bot-avatar">
            <Bot class="bot-avatar-icon" />
          </div>
          <div class="bot-meta">
            <span class="bot-name">{{ authStore.botName }}</span>
            <span class="bot-token">{{ tokenPrefix }}</span>
          </div>
        </div>
        <div class="topbar-divider"></div>
        <div class="current-group">
          <span class="group-label">当前群组</span>
          <span class="group-value">{{ store.selectedGroup?.name ?? '未选择' }}</span>
        </div>
      </div>

      <div class="topbar-right">
        <div class="conn-indicator">
          <span class="conn-dot" :class="store.connectionStatus"></span>
          <span class="conn-label">{{ store.connectionLabel }}</span>
        </div>
        <button type="button" class="logout-btn" title="退出登录" @click="handleLogout">
          <LogOut class="icon-sm" />
        </button>
      </div>
    </header>

    <div class="chat-body">
      <aside class="group-list-column">
        <div class="list-title">群聊</div>

        <div v-if="store.loading" class="empty-group-list">
          <MessageSquare class="empty-icon" />
          <p class="empty-title">加载中...</p>
        </div>

        <div v-else-if="store.groups.length === 0" class="empty-group-list">
          <Users class="empty-icon" />
          <p class="empty-title">暂无群组</p>
          <p class="empty-desc">此 Bot 尚未加入任何群聊</p>
        </div>

        <div v-else class="group-list">
          <button
            v-for="group in store.groups"
            :key="group.group_id"
            type="button"
            class="group-item"
            :class="{ active: store.selectedGroupId === group.group_id }"
            @click="store.selectGroup(group.group_id)"
          >
            <MessageSquare class="group-item-icon" />
            <div class="group-item-main">
              <span class="group-item-name">{{ group.name }}</span>
              <span class="group-item-preview">{{ group.description?.slice(0, 20) ?? '暂无描述' }}</span>
            </div>
          </button>
        </div>
      </aside>

      <section class="message-column">
        <div class="message-list-scroll">
          <div v-if="store.loadingMessages" class="empty-messages">
            <MessageSquare class="empty-icon" />
            <p>消息加载中...</p>
          </div>
          <div v-else-if="!store.selectedGroupId" class="empty-messages">
            <MessageSquare class="empty-icon" />
            <p>请选择一个群组开始聊天</p>
          </div>
          <div v-else-if="messageBlocks.length === 0" class="empty-messages">
            <MessageSquare class="empty-icon" />
            <p>暂无消息，发送第一条消息吧</p>
          </div>

          <template v-for="(item, idx) in messageBlocks" :key="idx">
            <div v-if="item.type === 'date'" class="date-divider">
              <span>{{ item.label }}</span>
            </div>

            <div
              v-else
              class="message-row"
              :class="{ 'is-self': item.message.sender_id === authStore.botId }"
            >
              <template v-if="item.message.sender_id !== authStore.botId">
                <div class="msg-avatar">
                  {{ item.message.sender_name?.charAt(0) ?? '?' }}
                </div>
                <div class="msg-body">
                  <div class="msg-header">
                    <span class="msg-sender">{{ item.message.sender_name ?? '未知用户' }}</span>
                    <span class="msg-time">{{ formatTime(item.message.created_at) }}</span>
                  </div>
                  <div v-if="item.message.msg_type === 'file'" class="msg-bubble file-bubble">
                    <FileMessageCard
                      :file-name="fileNameOf(item.message)"
                      :file-size="fileSizeOf(item.message)"
                      :file-url="fileUrlOf(item.message)"
                    />
                  </div>
                  <div v-else class="msg-bubble">
                    {{ textOf(item.message) }}
                  </div>
                </div>
              </template>

              <template v-else>
                <div class="msg-body self-body">
                  <div class="msg-header self-header">
                    <span class="msg-time">{{ formatTime(item.message.created_at) }}</span>
                    <span class="msg-sender">{{ item.message.sender_name ?? authStore.botName }}</span>
                  </div>
                  <div v-if="item.message.msg_type === 'file'" class="msg-bubble file-bubble">
                    <FileMessageCard
                      :file-name="fileNameOf(item.message)"
                      :file-size="fileSizeOf(item.message)"
                      :file-url="fileUrlOf(item.message)"
                    />
                  </div>
                  <div v-else class="msg-bubble self-bubble">
                    {{ textOf(item.message) }}
                  </div>
                </div>
                <div class="msg-avatar self-avatar">
                  <Bot class="bot-avatar-svg" />
                </div>
              </template>
            </div>
          </template>
        </div>

        <div class="chat-input-area">
          <div class="input-hint">Enter 发送，Shift + Enter 换行</div>

          <div v-if="store.pendingFile" class="pending-file-bar">
            <FileText class="pending-file-icon" />
            <span class="pending-file-name">{{ store.pendingFile.name }}</span>
            <span class="pending-file-size">{{ store.pendingFile.size }}</span>
            <button type="button" class="pending-file-remove" @click="store.setPendingFile(null)">
              <X class="icon-xs" />
            </button>
          </div>

          <div class="input-row">
            <button
              type="button"
              class="file-action-btn"
              :disabled="!store.selectedGroupId"
              @click="triggerFileSelect"
            >
              <Paperclip class="icon-sm" />
              发文件
            </button>
            <input
              ref="fileInput"
              class="file-input"
              type="file"
              @change="handleFileChange"
            />
            <textarea
              v-model="store.inputText"
              class="chat-textarea"
              :placeholder="textareaPlaceholder"
              :disabled="!store.selectedGroupId"
              @keydown.enter="handleKeydown"
            />
            <div class="send-actions">
              <button
                type="button"
                class="send-btn"
                :disabled="!store.canSend || store.sending"
                @click="store.sendDraft"
              >
                <Send class="icon-sm" />
                {{ store.sending ? '发送中' : '发送' }}
              </button>
            </div>
          </div>
        </div>
      </section>

      <aside class="inspector-column">
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
              <span class="info-value">{{ apiBaseUrl }}</span>
            </div>
            <div class="info-row">
              <span class="info-key">WebSocket</span>
              <span class="info-value">{{ wsBaseUrl }}</span>
            </div>
            <div class="info-row">
              <span class="info-key">连接身份</span>
              <span class="info-value">{{ authStore.botId || '未认证' }}</span>
            </div>
          </div>

          <button type="button" class="action-btn secondary" @click="store.initialize">
            {{ store.connectionStatus === 'testing' ? '连接中...' : '重新连接' }}
          </button>
        </div>

        <div class="info-panel">
          <h4 class="panel-title">群组信息</h4>
          <div class="info-rows">
            <div class="info-row">
              <span class="info-key">群组名称</span>
              <span class="info-value">{{ store.selectedGroup?.name ?? '-' }}</span>
            </div>
            <div class="info-row">
              <span class="info-key">群组码</span>
              <span class="info-value">{{ store.selectedGroup?.group_code ?? '-' }}</span>
            </div>
            <div class="info-row">
              <span class="info-key">创建时间</span>
              <span class="info-value">{{ formatDate(store.selectedGroup?.created_at) }}</span>
            </div>
            <div class="info-row">
              <span class="info-key">群组公告</span>
              <span class="info-value announce">{{ store.selectedGroup?.description ?? '暂无公告' }}</span>
            </div>
          </div>
        </div>

        <div class="info-panel">
          <h4 class="panel-title">最近文件</h4>
          <div class="file-list">
            <div
              v-for="file in recentFiles"
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
                <p class="file-size">{{ file.size || '未知大小' }}</p>
              </div>
              <div class="file-time">{{ file.time }}</div>
            </div>
            <div v-if="recentFiles.length === 0" class="empty-row">
              当前群聊暂无文件
            </div>
          </div>
        </div>

        <div class="info-panel">
          <h4 class="panel-title">Bot 信息</h4>
          <div class="info-rows">
            <div class="info-row">
              <span class="info-key">当前 Bot</span>
              <span class="info-value">{{ authStore.botName }}</span>
            </div>
            <div class="info-row">
              <span class="info-key">状态</span>
              <span class="info-value">{{ authStore.bot?.status ?? '-' }}</span>
            </div>
            <div class="info-row">
              <span class="info-key">Token</span>
              <span class="info-value">{{ tokenPrefix }}</span>
            </div>
          </div>
        </div>
      </aside>
    </div>

    <div v-if="store.notice" class="notice-toast" role="status">
      {{ store.notice }}
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue'
import { useRouter } from 'vue-router'
import {
  Bot,
  File,
  FileCode2,
  FileImage,
  FileText,
  LogOut,
  MessageSquare,
  Paperclip,
  Send,
  Users,
  X,
} from 'lucide-vue-next'
import { useBotUserAuthStore } from '@/stores/botUserAuth'
import { useBotUserChatStore } from '@/stores/botUserChat'
import { useWebSocket } from '@/composables/useWebSocket'
import FileMessageCard from '@/components/BotConsole/FileMessageCard.vue'
import type { Message } from '@/types'

const router = useRouter()
const authStore = useBotUserAuthStore()
const store = useBotUserChatStore()
const fileInput = ref<HTMLInputElement | null>(null)

const wsToken = computed(() => authStore.token)
const { isConnected } = useWebSocket(wsToken, store.addRealtimeMessage)
const apiBaseUrl = import.meta.env.VITE_API_BASE_URL || 'http://localhost:8080/api/v1'
const wsBaseUrl = import.meta.env.VITE_WS_BASE_URL || 'ws://localhost:8080/ws'

const tokenPrefix = computed(() => {
  const token = authStore.token || ''
  return token ? `${token.slice(0, 10)}...` : '未连接'
})

const textareaPlaceholder = computed(() => {
  if (!store.selectedGroupId) return '请先选择群聊'
  return '输入消息...'
})

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

const recentFiles = computed(() => {
  return store.groupMessages
    .filter((message) => message.msg_type === 'file')
    .slice()
    .sort((a, b) => new Date(b.created_at).getTime() - new Date(a.created_at).getTime())
    .slice(0, 6)
    .map((message) => {
      const name = fileNameOf(message)
      return {
        id: String(message.msg_id),
        name,
        size: fileSizeOf(message),
        time: formatRecentTime(message.created_at),
        type: inferFileType(name),
      }
    })
})

interface DateBlock { type: 'date'; label: string }
interface MsgBlock { type: 'msg'; message: Message }

const messageBlocks = computed<(DateBlock | MsgBlock)[]>(() => {
  const blocks: (DateBlock | MsgBlock)[] = []
  let lastDate = ''

  for (const msg of store.groupMessages) {
    const d = new Date(msg.created_at).toLocaleDateString('zh-CN', {
      year: 'numeric',
      month: '2-digit',
      day: '2-digit',
    })
    if (d !== lastDate) {
      lastDate = d
      blocks.push({ type: 'date', label: d })
    }
    blocks.push({ type: 'msg', message: msg })
  }

  return blocks
})

const handleKeydown = (e: KeyboardEvent) => {
  if (e.key === 'Enter' && !e.shiftKey) {
    e.preventDefault()
    void store.sendDraft()
  }
}

const triggerFileSelect = () => {
  if (!store.selectedGroupId) {
    store.notify('请先选择一个群聊')
    return
  }
  fileInput.value?.click()
}

const handleFileChange = (event: Event) => {
  const input = event.target as HTMLInputElement
  const file = input.files?.[0]
  if (file) store.setPendingFile(file)
  input.value = ''
}

const handleLogout = () => {
  store.reset()
  authStore.handleLogout()
  void router.replace('/bot/login')
}

const textOf = (msg: Message): string => {
  const c = msg.content
  if (typeof c === 'string') return c
  if (c && typeof c === 'object' && 'text' in c) return String(c.text ?? '')
  return ''
}

const fileNameOf = (msg: Message): string => {
  const c = msg.content
  if (typeof c === 'string') return '文件'
  if (c && typeof c === 'object') {
    if ('filename' in c && c.filename) return String(c.filename)
    if ('file_name' in c && c.file_name) return String(c.file_name)
    if ('url' in c && c.url) {
      const url = String(c.url)
      const seg = url.split('/').pop() || ''
      return seg ? decodeURIComponent(seg) : '文件'
    }
  }
  return '文件'
}

const fileSizeOf = (msg: Message): string => {
  const c = msg.content
  if (c && typeof c === 'object' && 'size' in c && c.size) return String(c.size)
  return ''
}

const fileUrlOf = (msg: Message): string => {
  const c = msg.content
  if (c && typeof c === 'object') {
    if ('url' in c && c.url) return String(c.url)
    if ('file_url' in c && c.file_url) return String(c.file_url)
  }
  return ''
}

const formatTime = (dateStr: string) => {
  return new Date(dateStr).toLocaleTimeString('zh-CN', { hour: '2-digit', minute: '2-digit' })
}

const formatDate = (dateStr?: string) => {
  if (!dateStr) return '-'
  return new Date(dateStr).toLocaleString('zh-CN', {
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit',
  })
}

const formatRecentTime = (dateStr: string) => {
  const date = new Date(dateStr)
  const now = new Date()
  if (date.toDateString() === now.toDateString()) {
    return date.toLocaleTimeString('zh-CN', { hour: '2-digit', minute: '2-digit' })
  }
  return date.toLocaleDateString('zh-CN', { month: '2-digit', day: '2-digit' })
}

const inferFileType = (fileName: string): 'doc' | 'pdf' | 'code' | 'image' | 'other' => {
  const lower = fileName.toLowerCase()
  if (/\.(pdf)$/.test(lower)) return 'pdf'
  if (/\.(png|jpe?g|gif|webp|svg)$/.test(lower)) return 'image'
  if (/\.(js|ts|tsx|jsx|py|rs|go|java|json|md|sql|yaml|yml|toml)$/.test(lower)) return 'code'
  if (/\.(doc|docx|xls|xlsx|csv|ppt|pptx|txt)$/.test(lower)) return 'doc'
  return 'other'
}

watch(isConnected, (connected) => {
  store.setWebSocketConnected(connected)
})

onMounted(() => {
  void store.initialize()
})
</script>

<style scoped>
.bot-chat-page {
  height: 100vh;
  display: flex;
  flex-direction: column;
  gap: 10px;
  min-height: 0;
  background: #f2f2f2;
  padding: 16px;
  overflow: hidden;
}

.chat-topbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  padding: 10px 16px;
  background: #f5f5f5;
  border: 1px solid #d0d0d0;
  border-radius: 8px;
  flex-shrink: 0;
}

.topbar-left,
.topbar-right,
.bot-identity,
.current-group,
.conn-indicator,
.input-row,
.pending-file-bar {
  display: flex;
  align-items: center;
}

.topbar-left {
  gap: 12px;
  min-width: 0;
}

.topbar-right {
  gap: 10px;
  flex-shrink: 0;
}

.bot-identity {
  gap: 9px;
  min-width: 0;
}

.bot-avatar,
.msg-avatar {
  width: 32px;
  height: 32px;
  border-radius: 50%;
  background: #ebebeb;
  color: #2f2f2f;
  display: grid;
  place-items: center;
  flex-shrink: 0;
}

.bot-avatar {
  background: #2f2f2f;
  color: #f3f3f3;
}

.bot-avatar-icon,
.bot-avatar-svg {
  width: 16px;
  height: 16px;
  stroke: currentColor;
  stroke-width: 2;
}

.bot-meta {
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 1px;
}

.bot-name {
  font-size: 14px;
  font-weight: 800;
  color: #1f1f1f;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.bot-token {
  font-size: 11px;
  color: #6f6f6f;
  font-family: ui-monospace, SFMono-Regular, Consolas, monospace;
}

.topbar-divider {
  width: 1px;
  height: 24px;
  background: #d0d0d0;
  flex-shrink: 0;
}

.current-group {
  align-items: baseline;
  gap: 8px;
  min-width: 0;
}

.group-label {
  font-size: 11px;
  font-weight: 600;
  color: #6f6f6f;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  white-space: nowrap;
}

.group-value {
  font-size: 14px;
  font-weight: 700;
  color: #1f1f1f;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.conn-indicator {
  gap: 6px;
}

.conn-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: #9f9f9f;
  flex-shrink: 0;
}

.conn-dot.connected {
  background: #2f8f4e;
}

.conn-dot.testing {
  background: #c9a227;
}

.conn-dot.error,
.conn-dot.disconnected {
  background: #c4453c;
}

.conn-label {
  font-size: 12px;
  font-weight: 600;
  color: #4f4f4f;
}

.logout-btn {
  width: 30px;
  height: 30px;
  border-radius: 6px;
  border: 1px solid #d0d0d0;
  background: #f5f5f5;
  color: #6f6f6f;
  display: grid;
  place-items: center;
  cursor: pointer;
  transition: all 0.15s ease;
}

.logout-btn:hover {
  background: #ebebeb;
  color: #c4453c;
}

.chat-body {
  flex: 1;
  min-height: 0;
  display: flex;
  gap: 10px;
  overflow: hidden;
}

.group-list-column {
  width: 220px;
  flex: 0 0 220px;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 8px;
  min-height: 0;
  background: #f5f5f5;
  border: 1px solid #d0d0d0;
  border-radius: 8px;
  padding: 10px;
}

.list-title {
  padding: 4px 2px 8px;
  color: #6f6f6f;
  font-size: 11px;
  font-weight: 800;
  letter-spacing: 0.5px;
}

.group-list {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.group-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 10px;
  border-radius: 6px;
  border: 1px solid transparent;
  background: transparent;
  cursor: pointer;
  text-align: left;
  transition: all 0.15s ease;
}

.group-item:hover {
  background: #ebebeb;
  border-color: #d0d0d0;
}

.group-item.active {
  background: #2f2f2f;
  color: #f3f3f3;
  border-color: #2f2f2f;
}

.group-item-icon {
  width: 14px;
  height: 14px;
  stroke: currentColor;
  stroke-width: 2;
  flex-shrink: 0;
}

.group-item-main {
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 1px;
}

.group-item-name,
.group-item-preview {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.group-item-name {
  font-size: 12px;
  font-weight: 700;
}

.group-item-preview {
  font-size: 11px;
  color: #6f6f6f;
}

.group-item.active .group-item-preview {
  color: #bfbfbf;
}

.message-column {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  min-height: 0;
  background: #f5f5f5;
  border: 1px solid #d0d0d0;
  border-radius: 8px;
  overflow: hidden;
}

.message-list-scroll {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
  padding: 14px;
  display: flex;
  flex-direction: column;
  gap: 14px;
}

.date-divider {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 10px;
  font-size: 11px;
  color: #9f9f9f;
  margin: 2px 0;
}

.date-divider::before,
.date-divider::after {
  content: '';
  flex: 1;
  height: 1px;
  background: #e0e0e0;
}

.message-row {
  display: flex;
  gap: 8px;
  align-items: flex-start;
}

.message-row.is-self {
  justify-content: flex-end;
}

.msg-avatar {
  font-size: 13px;
  font-weight: 700;
}

.msg-avatar.self-avatar {
  background: #2f2f2f;
  color: #f3f3f3;
}

.msg-body {
  max-width: 70%;
  min-width: 0;
}

.msg-body.self-body {
  display: flex;
  flex-direction: column;
  align-items: flex-end;
}

.msg-header {
  display: flex;
  align-items: baseline;
  gap: 6px;
  margin-bottom: 3px;
}

.msg-header.self-header {
  justify-content: flex-end;
}

.msg-sender {
  font-size: 12px;
  font-weight: 700;
  color: #2f2f2f;
}

.msg-time {
  font-size: 11px;
  color: #9f9f9f;
}

.msg-bubble {
  display: inline-block;
  padding: 8px 12px;
  background: #ffffff;
  border: 1px solid #d0d0d0;
  border-radius: 8px;
  font-size: 13px;
  color: #1f1f1f;
  line-height: 1.5;
  word-break: break-word;
  white-space: pre-wrap;
}

.msg-bubble.self-bubble {
  background: #f0f0f0;
}

.file-bubble {
  background: transparent;
  border: none;
  padding: 0;
}

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

.chat-input-area {
  flex-shrink: 0;
  padding: 10px 14px 14px;
  border-top: 1px solid #e0e0e0;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.input-hint {
  font-size: 11px;
  color: #9f9f9f;
}

.pending-file-bar {
  gap: 8px;
  padding: 6px 10px;
  background: #f0f0f0;
  border: 1px solid #d0d0d0;
  border-radius: 6px;
  align-self: flex-start;
}

.pending-file-icon {
  width: 14px;
  height: 14px;
  stroke: #c4453c;
  stroke-width: 2;
  flex-shrink: 0;
}

.pending-file-name {
  font-size: 12px;
  color: #1f1f1f;
}

.pending-file-size {
  font-size: 11px;
  color: #6f6f6f;
}

.pending-file-remove {
  width: 20px;
  height: 20px;
  border: none;
  background: transparent;
  color: #9f9f9f;
  display: grid;
  place-items: center;
  border-radius: 4px;
  cursor: pointer;
}

.pending-file-remove:hover {
  background: #ebebeb;
  color: #c4453c;
}

.input-row {
  gap: 8px;
  align-items: flex-end;
}

.file-action-btn,
.send-btn {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  cursor: pointer;
  transition: all 0.15s ease;
  flex-shrink: 0;
}

.file-action-btn {
  padding: 8px 12px;
  border-radius: 6px;
  border: 1px solid #d0d0d0;
  background: #f5f5f5;
  color: #4f4f4f;
  font-size: 12px;
  font-weight: 600;
}

.file-action-btn:hover {
  background: #ebebeb;
  border-color: #c5c5c5;
}

.file-action-btn:disabled,
.send-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.file-input {
  display: none;
}

.chat-textarea {
  flex: 1;
  min-width: 0;
  padding: 8px 10px;
  border: 1px solid #d0d0d0;
  border-radius: 6px;
  font-size: 13px;
  color: #1f1f1f;
  font-family: inherit;
  resize: none;
  min-height: 40px;
  max-height: 120px;
  background: #f5f5f5;
  transition: border-color 0.15s ease;
}

.chat-textarea:focus {
  outline: none;
  border-color: #7a7a7a;
  box-shadow: 0 0 0 3px rgba(120, 120, 120, 0.12);
}

.chat-textarea:disabled {
  color: #9f9f9f;
  background: #efefef;
  cursor: not-allowed;
}

.send-actions {
  flex-shrink: 0;
}

.send-btn {
  padding: 8px 16px;
  border-radius: 999px;
  border: 1px solid #2f2f2f;
  background: #2f2f2f;
  color: #f3f3f3;
  font-size: 13px;
  font-weight: 700;
}

.send-btn:hover:not(:disabled) {
  background: #3a3a3a;
}

.empty-group-list,
.empty-messages {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 8px;
  color: #9f9f9f;
  font-size: 13px;
  text-align: center;
}

.empty-messages {
  min-height: 200px;
}

.empty-icon {
  width: 28px;
  height: 28px;
  stroke: #bfbfbf;
  stroke-width: 1.5;
}

.empty-title {
  margin: 0;
  font-size: 13px;
  font-weight: 600;
  color: #6f6f6f;
}

.empty-desc {
  margin: 0;
  font-size: 11px;
  color: #9f9f9f;
}

.notice-toast {
  position: fixed;
  left: 50%;
  bottom: 28px;
  z-index: 50;
  transform: translateX(-50%);
  max-width: min(400px, calc(100vw - 32px));
  padding: 10px 14px;
  border: 1px solid #d0d0d0;
  border-radius: 8px;
  background: #2f2f2f;
  color: #f3f3f3;
  font-size: 13px;
  font-weight: 600;
  box-shadow: 0 12px 30px rgba(0, 0, 0, 0.18);
}

.icon-sm {
  width: 16px;
  height: 16px;
  stroke: currentColor;
  stroke-width: 2;
}

.icon-xs {
  width: 14px;
  height: 14px;
  stroke: currentColor;
  stroke-width: 2;
}

@media (max-width: 1200px) {
  .inspector-column {
    display: none;
  }
}

@media (max-width: 768px) {
  .bot-chat-page {
    padding: 10px;
  }

  .chat-topbar {
    flex-wrap: wrap;
    gap: 8px;
  }

  .chat-body {
    flex-direction: column;
  }

  .group-list-column {
    width: 100%;
    flex-basis: auto;
    max-height: 200px;
  }

  .message-column {
    min-height: 350px;
  }

  .msg-body {
    max-width: 82%;
  }
}
</style>
