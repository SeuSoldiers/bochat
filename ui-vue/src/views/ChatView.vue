<template>
  <div class="chat-shell">
    <!-- 顶部栏：Bot 选择器 + 当前群 + 连接状态 -->
    <header class="chat-topbar">
      <div class="topbar-left">
        <div class="bot-selector">
          <label class="selector-label">发言 Bot</label>
          <select
            v-model="selectedBotId"
            class="bot-select"
            @change="handleBotChange"
          >
            <option
              v-for="bot in store.bots"
              :key="bot.bot_id"
              :value="bot.bot_id"
            >
              {{ bot.name }}
            </option>
            <option v-if="store.bots.length === 0" value="" disabled>
              暂无 Bot
            </option>
          </select>
        </div>
        <div class="topbar-divider"></div>
        <div class="current-group">
          <span class="group-label">当前群组</span>
          <span class="group-value">{{ store.selectedGroup?.name ?? '未选择' }}</span>
          <span v-if="store.selectedGroupId" class="group-meta">
            {{ store.selectedMemberCount }} 人
          </span>
        </div>
      </div>
      <div class="topbar-right">
        <div class="conn-indicator">
          <span class="conn-dot" :class="store.connectionStatus"></span>
          <span class="conn-label">{{ store.connectionLabel }}</span>
        </div>
        <button
          type="button"
          class="toggle-panel-btn"
          :class="{ active: showInfoPanel }"
          @click="showInfoPanel = !showInfoPanel"
          title="信息面板"
        >
          <Info class="icon-sm" />
        </button>
      </div>
    </header>

    <div class="chat-body">
      <!-- 群聊列表 -->
      <aside class="group-list-column">
        <div class="list-tabs">
          <button
            v-for="tab in groupTabs"
            :key="tab.key"
            type="button"
            class="tab-btn"
            :class="{ active: store.activeTab === tab.key }"
            @click="store.activeTab = tab.key"
          >
            {{ tab.label }}
          </button>
        </div>

        <div v-if="store.groups.length === 0" class="empty-group-list">
          <Users class="empty-icon" />
          <p class="empty-title">暂无群组</p>
          <p class="empty-desc">创建或加入群组后即可开始聊天</p>
          <router-link to="/groups" class="empty-link">前往群组管理</router-link>
        </div>

        <div v-else class="group-list">
          <button
            v-for="group in store.filteredGroups"
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
          <div v-if="store.filteredGroups.length === 0" class="empty-tip">
            暂无匹配的群聊
          </div>
        </div>
      </aside>

      <!-- 消息区 -->
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
              :class="{ 'is-bot': item.message.sender_id === 'bot-customer' }"
            >
              <template v-if="item.message.sender_id !== 'bot-customer'">
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
                <div class="msg-body bot-body">
                  <div class="msg-header bot-header">
                    <span class="msg-time">{{ formatTime(item.message.created_at) }}</span>
                    <span class="msg-sender">{{ item.message.sender_name ?? 'Bot' }}</span>
                  </div>
                  <div class="msg-bubble bot-bubble">
                    {{ textOf(item.message) }}
                  </div>
                </div>
                <div class="msg-avatar bot-avatar">
                  <Bot class="bot-avatar-svg" />
                </div>
              </template>
            </div>
          </template>
        </div>

        <!-- 输入区 -->
        <div class="chat-input-area">
          <div class="input-hint">Enter 发送，Shift + Enter 换行</div>

          <div v-if="store.bots.length === 0" class="no-bot-banner">
            <AlertCircle class="banner-icon" />
            <span>请先创建 Bot 才能发送消息</span>
            <router-link to="/bots" class="banner-action">创建 Bot</router-link>
          </div>

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
              :disabled="!store.selectedGroupId || store.bots.length === 0"
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
              :disabled="!store.selectedGroupId || store.bots.length === 0"
              @keydown.enter="handleKeydown"
            />
            <div class="send-actions">
              <button
                type="button"
                class="send-btn"
                :disabled="!store.canSend || store.sending"
                @click="handleSendText"
              >
                <Send class="icon-sm" />
                {{ store.sending ? '发送中' : '发送' }}
              </button>
            </div>
          </div>
        </div>
      </section>

      <!-- 右侧信息面板（可折叠） -->
      <aside v-if="showInfoPanel" class="info-panel-column">
        <BotChatInspector />
      </aside>
    </div>

    <div v-if="store.notice" class="notice-toast" role="status">
      {{ store.notice }}
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue'
import {
  Users,
  MessageSquare,
  Bot,
  FileText,
  X,
  Paperclip,
  Send,
  AlertCircle,
  Info,
} from 'lucide-vue-next'
import { useBotConsoleStore } from '@/stores/botConsole'
import type { Message } from '@/types'
import { useWebSocket } from '@/composables/useWebSocket'
import BotChatInspector from '@/components/BotConsole/BotChatInspector.vue'
import FileMessageCard from '@/components/BotConsole/FileMessageCard.vue'

const store = useBotConsoleStore()
const fileInput = ref<HTMLInputElement | null>(null)
const showInfoPanel = ref(true)
const selectedBotId = ref(store.activeBotId)

const wsToken = computed(() => store.botToken || null)
const { isConnected } = useWebSocket(wsToken, store.addRealtimeMessage)

watch(() => store.activeBotId, (newId) => {
  selectedBotId.value = newId
})

const groupTabs = [
  { key: 'all' as const, label: '全部' },
  { key: 'joined' as const, label: '我加入的' },
  { key: 'managed' as const, label: '我管理的' },
]

const textareaPlaceholder = computed(() => {
  if (store.bots.length === 0) return '请先创建 Bot'
  if (!store.selectedGroupId) return '请先选择群聊'
  return '输入消息...'
})

interface DateBlock { type: 'date'; label: string }
interface MsgBlock { type: 'msg'; message: Message }

const messageBlocks = computed<(DateBlock | MsgBlock)[]>(() => {
  const list = store.groupMessages
  const blocks: (DateBlock | MsgBlock)[] = []
  let lastDate = ''

  for (const msg of list) {
    const d = new Date(msg.created_at).toLocaleDateString('zh-CN', {
      year: 'numeric', month: '2-digit', day: '2-digit',
    })
    if (d !== lastDate) {
      lastDate = d
      blocks.push({ type: 'date', label: d })
    }
    blocks.push({ type: 'msg', message: msg })
  }

  return blocks
})

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

const handleBotChange = () => {
  store.activeBotId = selectedBotId.value
}

const handleKeydown = (e: KeyboardEvent) => {
  if (e.key === 'Enter' && !e.shiftKey) {
    e.preventDefault()
    void handleSendText()
  }
}

const handleSendText = async () => {
  await store.sendDraft()
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

watch(isConnected, (connected) => {
  store.setWebSocketConnected(connected)
})

onMounted(() => {
  void store.initialize()
})
</script>

<style scoped>
.chat-shell {
  height: 100%;
  display: flex;
  flex-direction: column;
  gap: 10px;
  min-height: 0;
}

/* ===== 顶部栏 ===== */
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

.topbar-left {
  display: flex;
  align-items: center;
  gap: 12px;
  min-width: 0;
}

.bot-selector {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-shrink: 0;
}

.selector-label {
  font-size: 11px;
  font-weight: 600;
  color: #6f6f6f;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  white-space: nowrap;
}

.bot-select {
  padding: 5px 8px;
  border: 1px solid #d0d0d0;
  border-radius: 6px;
  background: #fff;
  color: #1f1f1f;
  font-size: 12px;
  font-weight: 600;
  font-family: inherit;
  cursor: pointer;
  outline: none;
  min-width: 120px;
}

.bot-select:focus {
  border-color: #7a7a7a;
}

.topbar-divider {
  width: 1px;
  height: 24px;
  background: #d0d0d0;
  flex-shrink: 0;
}

.current-group {
  display: flex;
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

.group-meta {
  font-size: 11px;
  color: #9f9f9f;
  white-space: nowrap;
}

.topbar-right {
  display: flex;
  align-items: center;
  gap: 10px;
  flex-shrink: 0;
}

.conn-indicator {
  display: inline-flex;
  align-items: center;
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

.toggle-panel-btn {
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

.toggle-panel-btn:hover,
.toggle-panel-btn.active {
  background: #ebebeb;
  color: #1f1f1f;
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

/* ===== Chat Body ===== */
.chat-body {
  flex: 1;
  min-height: 0;
  display: flex;
  gap: 10px;
  overflow: hidden;
}

/* ===== 群聊列表 ===== */
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

.list-tabs {
  display: flex;
  gap: 4px;
  flex-shrink: 0;
}

.tab-btn {
  flex: 1;
  padding: 5px 6px;
  border-radius: 6px;
  border: 1px solid transparent;
  background: transparent;
  color: #6f6f6f;
  font-size: 11px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.15s ease;
}

.tab-btn:hover {
  background: #ebebeb;
  color: #1f1f1f;
}

.tab-btn.active {
  background: #2f2f2f;
  color: #f3f3f3;
  border-color: #2f2f2f;
}

.group-list {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: 4px;
  padding-right: 2px;
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

.group-item-name {
  font-size: 12px;
  font-weight: 700;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.group-item-preview {
  font-size: 11px;
  color: #6f6f6f;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.group-item.active .group-item-preview {
  color: #bfbfbf;
}

.empty-group-list {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 8px;
  padding: 16px;
  text-align: center;
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

.empty-link {
  display: inline-block;
  padding: 4px 12px;
  border: 1px solid #2f2f2f;
  border-radius: 6px;
  background: #2f2f2f;
  color: #f3f3f3;
  font-size: 11px;
  font-weight: 600;
  text-decoration: none;
  transition: background 0.15s ease;
}

.empty-link:hover {
  background: #3a3a3a;
}

.empty-tip {
  padding: 16px 8px;
  text-align: center;
  font-size: 12px;
  color: #9f9f9f;
}

/* ===== 消息区 ===== */
.message-column {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 0;
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

.message-row.is-bot {
  justify-content: flex-end;
}

.msg-avatar {
  width: 32px;
  height: 32px;
  border-radius: 50%;
  background: #ebebeb;
  color: #2f2f2f;
  display: grid;
  place-items: center;
  font-size: 13px;
  font-weight: 700;
  flex-shrink: 0;
}

.msg-avatar.bot-avatar {
  background: #2f2f2f;
  color: #f3f3f3;
}

.bot-avatar-svg {
  width: 16px;
  height: 16px;
  stroke: currentColor;
  stroke-width: 2;
}

.msg-body {
  max-width: 70%;
  min-width: 0;
}

.msg-body.bot-body {
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

.msg-header.bot-header {
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

.msg-bubble.bot-bubble {
  background: #f0f0f0;
}

.file-bubble {
  background: transparent;
  border: none;
  padding: 0;
}

/* ===== 输入区 ===== */
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

.no-bot-banner {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 12px;
  background: #fff8e6;
  border: 1px solid #e6d594;
  border-radius: 6px;
  font-size: 12px;
  color: #8a6d14;
  font-weight: 600;
}

.banner-icon {
  width: 16px;
  height: 16px;
  stroke: #c9a227;
  stroke-width: 2;
  flex-shrink: 0;
}

.banner-action {
  margin-left: auto;
  padding: 4px 10px;
  border: 1px solid #c9a227;
  border-radius: 5px;
  background: transparent;
  color: #8a6d14;
  font-size: 12px;
  font-weight: 600;
  text-decoration: none;
  cursor: pointer;
  transition: all 0.15s ease;
}

.banner-action:hover {
  background: #c9a227;
  color: #fff;
}

.pending-file-bar {
  display: inline-flex;
  align-items: center;
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
  display: flex;
  gap: 8px;
  align-items: flex-end;
}

.file-action-btn {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: 8px 12px;
  border-radius: 6px;
  border: 1px solid #d0d0d0;
  background: #f5f5f5;
  color: #4f4f4f;
  font-size: 12px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.15s ease;
  flex-shrink: 0;
}

.file-action-btn:hover {
  background: #ebebeb;
  border-color: #c5c5c5;
}

.file-action-btn:disabled {
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
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: 8px 16px;
  border-radius: 999px;
  border: 1px solid #2f2f2f;
  background: #2f2f2f;
  color: #f3f3f3;
  font-size: 13px;
  font-weight: 700;
  cursor: pointer;
  transition: all 0.15s ease;
}

.send-btn:hover:not(:disabled) {
  background: #3a3a3a;
}

.send-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

/* ===== 右侧信息面板 ===== */
.info-panel-column {
  width: 260px;
  flex: 0 0 260px;
  min-width: 0;
  overflow-y: auto;
}

/* ===== 空状态 ===== */
.empty-messages {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 8px;
  color: #9f9f9f;
  font-size: 13px;
  min-height: 200px;
}

/* ===== Toast 通知 ===== */
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

/* ===== 响应式 ===== */
@media (max-width: 1200px) {
  .group-list-column {
    width: 180px;
    flex-basis: 180px;
  }

  .info-panel-column {
    width: 220px;
    flex-basis: 220px;
  }
}

@media (max-width: 768px) {
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

  .info-panel-column {
    display: none;
  }
}
</style>
