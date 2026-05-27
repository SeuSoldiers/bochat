/**
 * Bot 工作台真实数据状态管理（Pinia）
 * 聚合后端 API 数据，供聊天工作台开发测试使用。
 */

import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { getBotList } from '@/services/bot'
import { getGroupList, getGroupMembers } from '@/services/group'
import { getMessages, sendMessage } from '@/services/message'
import { uploadFile } from '@/services/file'
import { useAuthStore } from '@/stores/auth'
import { getErrorMessage } from '@/utils/error'
import type { Bot, Group, GroupMember, Message } from '@/types'

export interface RecentFile {
  id: string
  groupId: string
  name: string
  size: string
  time: string
  type: 'doc' | 'pdf' | 'code' | 'image' | 'bin' | 'other'
}

export interface ConnectionLog {
  time: string
  level: 'info' | 'warn' | 'error'
  message: string
}

type ConnStatus = 'idle' | 'testing' | 'connected' | 'disconnected' | 'error'

interface PendingFile {
  file: File
  name: string
  size: string
}

const API_BASE_URL = import.meta.env.VITE_API_BASE_URL || `${window.location.protocol}//${window.location.hostname}:48080/api/v1`
const WS_BASE_URL = import.meta.env.VITE_WS_BASE_URL || `${window.location.protocol === 'https:' ? 'wss:' : 'ws:'}//${window.location.hostname}:48080/ws`

export const useBotConsoleStore = defineStore('botConsole', () => {
  const authStore = useAuthStore()

  const baseUrl = ref(API_BASE_URL)
  const wsUrl = ref(WS_BASE_URL)
  const configName = ref('后端真实数据')

  const connectionStatus = ref<ConnStatus>('idle')
  const connectionDuration = ref('00:00:00')
  const connectionLatency = ref(0)
  const lastConnectedAt = ref('—')
  let connectionStartedAt = 0
  let durationTimer: ReturnType<typeof setInterval> | undefined
  let latencyTimer: ReturnType<typeof setInterval> | undefined

  const healthUrl = (() => {
    try {
      const u = new URL(API_BASE_URL)
      return `${u.origin}/health`
    } catch {
      return `${window.location.protocol}//${window.location.hostname}:48080/health`
    }
  })()

  const bots = ref<Bot[]>([])
  const activeBotId = ref('')
  const groups = ref<Group[]>([])
  const selectedGroupId = ref<string | null>(null)
  const groupMembers = ref<Record<string, GroupMember[]>>({})
  const activeTab = ref<'all' | 'joined' | 'managed'>('all')
  const activeView = ref<'chat' | 'groups' | 'files' | 'logs'>('chat')

  const messages = ref<Message[]>([])
  const loading = ref(false)
  const loadingMessages = ref(false)
  const sending = ref(false)
  const error = ref<string | null>(null)

  const connectionLogs = ref<ConnectionLog[]>([])
  const inputText = ref('')
  const pendingFile = ref<PendingFile | null>(null)
  const notice = ref('')
  let noticeTimer: number | undefined

  const activeBot = computed(() =>
    bots.value.find((bot) => bot.bot_id === activeBotId.value) || bots.value[0] || null
  )

  const botToken = computed(() => activeBot.value?.token || '')

  const selectedGroup = computed(() =>
    groups.value.find((group) => group.group_id === selectedGroupId.value) || null
  )

  const selectedMemberCount = computed(() => {
    if (!selectedGroupId.value) return 0
    return groupMembers.value[selectedGroupId.value]?.length ?? 0
  })

  const filteredGroups = computed(() => {
    if (activeTab.value === 'all') return groups.value
    if (activeTab.value === 'managed') {
      return groups.value.filter((group) => group.creator_id === authStore.userId)
    }
    return groups.value.filter((group) => group.creator_id !== authStore.userId)
  })

  const groupMessages = computed(() =>
    messages.value.filter((message) => message.group_id === selectedGroupId.value)
  )

  const visibleRecentFiles = computed<RecentFile[]>(() => {
    const source = selectedGroupId.value
      ? messages.value.filter((message) => message.group_id === selectedGroupId.value)
      : messages.value

    return source
      .filter((message) => message.msg_type === 'file')
      .slice()
      .sort((a, b) => new Date(b.created_at).getTime() - new Date(a.created_at).getTime())
      .slice(0, 6)
      .map((message) => {
        const name = fileNameOf(message)
        return {
          id: String(message.msg_id),
          groupId: message.group_id,
          name,
          size: fileSizeOf(message),
          time: formatRecentTime(message.created_at),
          type: inferFileType(name),
        }
      })
  })

  const recentFiles = computed(() => visibleRecentFiles.value)

  const botIdentity = computed(() => ({
    name: activeBot.value?.name || '未选择 Bot',
    tokenPrefix: botToken.value ? botToken.value.slice(0, 8) : '',
    status: connectionStatus.value === 'connected' ? 'online' as const : 'offline' as const,
  }))

  const connectionLabel = computed(() => {
    switch (connectionStatus.value) {
      case 'connected':
        return '连接正常'
      case 'testing':
        return '连接中...'
      case 'error':
        return '连接异常'
      case 'disconnected':
        return '已断开'
      default:
        return '等待连接'
    }
  })

  const canSend = computed(() =>
    Boolean(selectedGroupId.value && botToken.value && (inputText.value.trim() || pendingFile.value))
  )

  const notify = (message: string) => {
    notice.value = message
    if (noticeTimer) window.clearTimeout(noticeTimer)
    noticeTimer = window.setTimeout(() => {
      notice.value = ''
    }, 2400)
  }

  const addLog = (level: ConnectionLog['level'], message: string) => {
    connectionLogs.value.unshift({
      time: new Date().toLocaleTimeString('zh-CN', {
        hour: '2-digit',
        minute: '2-digit',
        second: '2-digit',
      }),
      level,
      message,
    })
  }

  const formatDuration = (seconds: number): string => {
    const h = Math.floor(seconds / 3600)
    const m = Math.floor((seconds % 3600) / 60)
    const s = seconds % 60
    return [h, m, s].map((v) => String(v).padStart(2, '0')).join(':')
  }

  const startDurationTimer = () => {
    stopDurationTimer()
    durationTimer = setInterval(() => {
      if (connectionStartedAt > 0) {
        connectionDuration.value = formatDuration(
          Math.floor((Date.now() - connectionStartedAt) / 1000)
        )
      }
    }, 1000)
  }

  const stopDurationTimer = () => {
    if (durationTimer) {
      clearInterval(durationTimer)
      durationTimer = undefined
    }
  }

  const measureLatency = async () => {
    try {
      const start = performance.now()
      await fetch(healthUrl, { method: 'HEAD', cache: 'no-store' })
      connectionLatency.value = Math.max(1, Math.round(performance.now() - start))
    } catch {
      // 保持上次的延迟值
    }
  }

  const startLatencyTimer = () => {
    stopLatencyTimer()
    void measureLatency()
    latencyTimer = setInterval(() => {
      void measureLatency()
    }, 5_000)
  }

  const stopLatencyTimer = () => {
    if (latencyTimer) {
      clearInterval(latencyTimer)
      latencyTimer = undefined
    }
  }

  const initialize = async () => {
    loading.value = true
    error.value = null
    connectionStatus.value = 'testing'
    const startedAt = performance.now()

    try {
      const [botList, groupList] = await Promise.all([
        getBotList(),
        getGroupList(),
      ])

      bots.value = botList
      groups.value = groupList

      // 管理员用户默认选择自己的超级管理员Bot
      if (!activeBotId.value && authStore.isSuperAdmin) {
        const superAdminBot = botList.find(bot => bot.owner_id === authStore.userId)
        if (superAdminBot) {
          activeBotId.value = superAdminBot.bot_id
        }
      }
      activeBotId.value = activeBotId.value || botList[0]?.bot_id || ''

      if (!selectedGroupId.value || !groupList.some((group) => group.group_id === selectedGroupId.value)) {
        selectedGroupId.value = groupList[0]?.group_id ?? null
      }

      if (selectedGroupId.value) {
        await Promise.all([
          fetchGroupMembers(selectedGroupId.value),
          fetchMessagesForGroup(selectedGroupId.value),
        ])
      } else {
        messages.value = []
      }

      connectionLatency.value = Math.max(1, Math.round(performance.now() - startedAt))
      connectionStatus.value = 'connected'
      stopDurationTimer()
      stopLatencyTimer()
      connectionStartedAt = Date.now()
      startDurationTimer()
      startLatencyTimer()
      lastConnectedAt.value = formatDateTime(new Date().toISOString())
      addLog('info', '已从后端加载真实 Bot、群组和消息数据')
    } catch (err: any) {
      error.value = getErrorMessage(err, '加载真实数据失败')
      connectionStatus.value = 'error'
      addLog('error', error.value)
      notify(error.value)
      throw err
    } finally {
      loading.value = false
    }
  }

  const fetchMessagesForGroup = async (groupId: string) => {
    loadingMessages.value = true
    try {
      const fetched = await getMessages(groupId, null, 50, botToken.value || undefined)
      const normalized = fetched.map(normalizeMessage)
      messages.value = [
        ...messages.value.filter((message) => message.group_id !== groupId),
        ...normalized,
      ].sort(byCreatedAt)
      return normalized
    } finally {
      loadingMessages.value = false
    }
  }

  const fetchGroupMembers = async (groupId: string) => {
    const members = await getGroupMembers(groupId)
    groupMembers.value[groupId] = members
    return members
  }

  const selectGroup = async (groupId: string) => {
    selectedGroupId.value = groupId
    try {
      await Promise.all([
        fetchGroupMembers(groupId),
        fetchMessagesForGroup(groupId),
      ])
    } catch (err: any) {
      const message = getErrorMessage(err, '切换群聊失败')
      error.value = message
      notify(message)
    }
  }

  const sendDraft = async () => {
    if (!selectedGroupId.value) {
      notify('请先选择一个群聊')
      return
    }

    if (!botToken.value) {
      notify('当前账号没有可用于发送消息的 Bot')
      return
    }

    const text = inputText.value.trim()
    const file = pendingFile.value
    if (!text && !file) {
      notify('请输入消息或选择文件')
      return
    }

    sending.value = true
    error.value = null

    try {
      if (text) {
        const savedMessage = await sendMessage({
          group_id: selectedGroupId.value,
          content: { text },
          msg_type: 'text',
          idempotency_key: createIdempotencyKey(),
        }, botToken.value)
        addMessage(normalizeMessage(savedMessage))
        inputText.value = ''
      }

      if (file) {
        const uploaded = await uploadFile(file.file, botToken.value)
        const savedMessage = await sendMessage({
          group_id: selectedGroupId.value,
          content: {
            url: uploaded.url,
            filename: uploaded.filename || file.name,
            size: file.size,
          },
          msg_type: 'file',
          idempotency_key: createIdempotencyKey(),
        }, botToken.value)
        addMessage(normalizeMessage(savedMessage))
        pendingFile.value = null
      }
    } catch (err: any) {
      error.value = getErrorMessage(err, '发送失败')
      notify(error.value)
    } finally {
      sending.value = false
    }
  }

  const setPendingFile = (file: File | null) => {
    pendingFile.value = file
      ? {
          file,
          name: file.name,
          size: formatFileSize(file.size),
        }
      : null
  }

  const addMessage = (message: Message) => {
    const exists = messages.value.some((item) => item.msg_id === message.msg_id)
    if (!exists) {
      messages.value.push(message)
      messages.value.sort(byCreatedAt)
    }
  }

  const addRealtimeMessage = (message: Message) => {
    addMessage(normalizeMessage(message))
  }

  const reconnect = async () => {
    try {
      await initialize()
      notify('已重新加载后端数据')
    } catch {
      // initialize 已处理错误状态和提示
    }
  }

  const setActiveView = (view: 'chat' | 'groups' | 'files' | 'logs') => {
    activeView.value = view
  }

  const setWebSocketConnected = (connected: boolean) => {
    if (connected) {
      connectionStatus.value = 'connected'
      if (connectionStartedAt === 0) {
        connectionStartedAt = Date.now()
        startDurationTimer()
        startLatencyTimer()
      }
      return
    }

    if (connectionStatus.value === 'connected') {
      connectionStatus.value = 'disconnected'
      stopDurationTimer()
      stopLatencyTimer()
    }
  }

  return {
    baseUrl,
    wsUrl,
    configName,
    connectionStatus,
    connectionDuration,
    connectionLatency,
    lastConnectedAt,
    connectionLabel,
    bots,
    activeBotId,
    activeBot,
    botToken,
    groups,
    selectedGroupId,
    selectedGroup,
    selectedMemberCount,
    activeTab,
    activeView,
    setActiveView,
    filteredGroups,
    groupMembers,
    messages,
    groupMessages,
    loading,
    loadingMessages,
    sending,
    error,
    recentFiles,
    visibleRecentFiles,
    connectionLogs,
    botIdentity,
    inputText,
    pendingFile,
    notice,
    canSend,
    notify,
    initialize,
    selectGroup,
    sendDraft,
    setPendingFile,
    addRealtimeMessage,
    reconnect,
    setWebSocketConnected,
  }
})

function normalizeMessage(message: Message): Message {
  const normalized = { ...message }

  if (typeof normalized.msg_id !== 'number') {
    const numericId = Number(normalized.msg_id)
    if (Number.isFinite(numericId)) {
      normalized.msg_id = numericId
    }
  }

  if (typeof normalized.content === 'string') {
    const trimmed = normalized.content.trim()
    if ((trimmed.startsWith('{') && trimmed.endsWith('}')) || (trimmed.startsWith('[') && trimmed.endsWith(']'))) {
      try {
        const parsed = JSON.parse(trimmed)
        if (parsed && typeof parsed === 'object') {
          normalized.content = parsed
        }
      } catch {
        // 保持原始字符串
      }
    }
  }

  if (typeof normalized.content !== 'string' && normalized.content) {
    const content = { ...normalized.content } as Record<string, unknown>
    if (typeof content.file_url === 'string' && typeof content.url !== 'string') {
      content.url = content.file_url
    }
    if (typeof content.file_name === 'string' && typeof content.filename !== 'string') {
      content.filename = content.file_name
    }
    normalized.content = content
  }

  if (
    normalized.msg_type === 'file' &&
    typeof normalized.content === 'string' &&
    normalized.content.trim() &&
    !normalized.content.trim().startsWith('{')
  ) {
    normalized.content = { url: normalized.content.trim() }
  }

  return normalized
}

function addContentField(message: Message, field: string): string {
  const content = message.content
  if (content && typeof content === 'object' && field in content && content[field]) {
    return String(content[field])
  }
  return ''
}

function fileNameOf(message: Message): string {
  const filename = addContentField(message, 'filename') || addContentField(message, 'file_name')
  if (filename) return filename

  const url = addContentField(message, 'url') || addContentField(message, 'file_url')
  const segment = url.split('/').pop() || ''
  return segment ? decodeURIComponent(segment) : '文件'
}

function fileSizeOf(message: Message): string {
  return addContentField(message, 'size') || addContentField(message, 'file_size') || ''
}

function inferFileType(fileName: string): RecentFile['type'] {
  const lower = fileName.toLowerCase()
  if (/\.(pdf)$/.test(lower)) return 'pdf'
  if (/\.(png|jpe?g|gif|webp|bmp|svg|ico)$/.test(lower)) return 'image'
  if (/\.(js|ts|tsx|jsx|py|rs|go|java|c|cpp|h|hpp|json|yaml|yml|toml|md|sql)$/.test(lower)) return 'code'
  if (/\.(dll|so|dylib|exe|bin|msi|apk|ipa|deb|rpm)$/.test(lower)) return 'bin'
  if (/\.(doc|docx|odt|rtf|ppt|pptx|key|txt|xls|xlsx|csv)$/.test(lower)) return 'doc'
  return 'other'
}

function formatRecentTime(dateStr: string): string {
  const date = new Date(dateStr)
  const now = new Date()
  if (date.toDateString() === now.toDateString()) {
    return date.toLocaleTimeString('zh-CN', { hour: '2-digit', minute: '2-digit' })
  }
  return date.toLocaleDateString('zh-CN', { month: '2-digit', day: '2-digit' })
}

function formatDateTime(dateStr: string): string {
  return new Date(dateStr).toLocaleString('zh-CN', {
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit',
    hour12: false,
  })
}

function formatFileSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`
  return `${(bytes / 1024 / 1024).toFixed(1)} MB`
}

function createIdempotencyKey() {
  if (typeof crypto !== 'undefined' && typeof crypto.randomUUID === 'function') {
    return crypto.randomUUID()
  }
  return `msg-${Date.now()}-${Math.random().toString(16).slice(2)}`
}

function byCreatedAt(a: Message, b: Message) {
  return new Date(a.created_at).getTime() - new Date(b.created_at).getTime()
}
