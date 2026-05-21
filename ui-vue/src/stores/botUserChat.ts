import { computed, ref } from 'vue'
import { defineStore } from 'pinia'
import { STORAGE_KEYS } from '@/constants/storageKeys'
import { getBotVisibleGroups } from '@/services/botUser'
import { getMessages, sendMessage } from '@/services/message'
import { uploadFile } from '@/services/file'
import { useBotUserAuthStore } from '@/stores/botUserAuth'
import type { Group, Message } from '@/types'
import { getErrorMessage } from '@/utils/error'

interface PendingFile {
  file: File
  name: string
  size: string
}

type ConnStatus = 'idle' | 'testing' | 'connected' | 'disconnected' | 'error'

export const useBotUserChatStore = defineStore('botUserChat', () => {
  const authStore = useBotUserAuthStore()

  const groups = ref<Group[]>([])
  const selectedGroupId = ref<string | null>(localStorage.getItem(STORAGE_KEYS.BOT_SELECTED_GROUP_ID))
  const messages = ref<Message[]>([])
  const loading = ref(false)
  const loadingMessages = ref(false)
  const sending = ref(false)
  const error = ref<string | null>(null)
  const inputText = ref('')
  const pendingFile = ref<PendingFile | null>(null)
  const notice = ref('')
  const connectionStatus = ref<ConnStatus>('idle')
  const connectionDuration = ref('00:00:00')
  const connectionLatency = ref(0)
  let connectionStartedAt = 0
  let durationTimer: ReturnType<typeof setInterval> | undefined
  let latencyTimer: ReturnType<typeof setInterval> | undefined
  let noticeTimer: number | undefined

  const token = computed(() => authStore.token || '')
  const bot = computed(() => authStore.bot)
  const selectedGroup = computed(() =>
    groups.value.find((group) => group.group_id === selectedGroupId.value) || null
  )
  const groupMessages = computed(() =>
    messages.value.filter((message) => message.group_id === selectedGroupId.value)
  )
  const canSend = computed(() =>
    Boolean(selectedGroupId.value && token.value && (inputText.value.trim() || pendingFile.value))
  )
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

  const healthUrl = (() => {
    const base = import.meta.env.VITE_API_BASE_URL || `${window.location.protocol}//${window.location.hostname}:48080/api/v1`
    try {
      const u = new URL(base)
      return `${u.origin}/health`
    } catch {
      return `${window.location.protocol}//${window.location.hostname}:48080/health`
    }
  })()

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

  const notify = (message: string) => {
    notice.value = message
    if (noticeTimer) window.clearTimeout(noticeTimer)
    noticeTimer = window.setTimeout(() => {
      notice.value = ''
    }, 2400)
  }

  const initialize = async () => {
    if (!token.value) return

    loading.value = true
    error.value = null
    connectionStatus.value = 'testing'
    const startedAt = performance.now()
    connectionStartedAt = Date.now()

    try {
      const fetchedGroups = await getBotVisibleGroups(token.value)
      groups.value = fetchedGroups

      if (!selectedGroupId.value || !fetchedGroups.some((group) => group.group_id === selectedGroupId.value)) {
        selectedGroupId.value = fetchedGroups[0]?.group_id ?? null
      }

      if (selectedGroupId.value) {
        localStorage.setItem(STORAGE_KEYS.BOT_SELECTED_GROUP_ID, selectedGroupId.value)
        await fetchMessagesForGroup(selectedGroupId.value)
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
    } catch (err: any) {
      error.value = getErrorMessage(err, '加载 Bot 聊天数据失败')
      connectionStatus.value = 'error'
      notify(error.value)
      throw err
    } finally {
      loading.value = false
    }
  }

  const fetchMessagesForGroup = async (groupId: string) => {
    if (!token.value) return []

    loadingMessages.value = true
    try {
      const fetched = await getMessages(groupId, null, 50, token.value)
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

  const selectGroup = async (groupId: string) => {
    selectedGroupId.value = groupId
    localStorage.setItem(STORAGE_KEYS.BOT_SELECTED_GROUP_ID, groupId)

    try {
      await fetchMessagesForGroup(groupId)
    } catch (err: any) {
      error.value = getErrorMessage(err, '切换群聊失败')
      notify(error.value)
    }
  }

  const sendDraft = async () => {
    if (!selectedGroupId.value) {
      notify('请先选择一个群聊')
      return
    }

    if (!token.value) {
      notify('Bot Token 已失效，请重新登录')
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
        }, token.value)
        addMessage(normalizeMessage(savedMessage))
        inputText.value = ''
      }

      if (file) {
        const uploaded = await uploadFile(file.file, token.value)
        const savedMessage = await sendMessage({
          group_id: selectedGroupId.value,
          content: {
            url: uploaded.url,
            filename: uploaded.filename || file.name,
            size: file.size,
          },
          msg_type: 'file',
          idempotency_key: createIdempotencyKey(),
        }, token.value)
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

  const setWebSocketConnected = (connected: boolean) => {
    if (connected) {
      connectionStatus.value = 'connected'
      return
    }

    if (connectionStatus.value === 'connected') {
      connectionStatus.value = 'disconnected'
    }
  }

  const reset = () => {
    stopDurationTimer()
    stopLatencyTimer()
    groups.value = []
    selectedGroupId.value = null
    messages.value = []
    inputText.value = ''
    pendingFile.value = null
    connectionStatus.value = 'idle'
    connectionDuration.value = '00:00:00'
    connectionLatency.value = 0
    connectionStartedAt = 0
    localStorage.removeItem(STORAGE_KEYS.BOT_SELECTED_GROUP_ID)
  }

  return {
    groups,
    selectedGroupId,
    selectedGroup,
    messages,
    groupMessages,
    bot,
    loading,
    loadingMessages,
    sending,
    error,
    inputText,
    pendingFile,
    notice,
    canSend,
    connectionStatus,
    connectionDuration,
    connectionLatency,
    connectionLabel,
    initialize,
    fetchMessagesForGroup,
    selectGroup,
    sendDraft,
    setPendingFile,
    addRealtimeMessage,
    setWebSocketConnected,
    notify,
    reset,
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
        // Keep raw string content.
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

function createIdempotencyKey() {
  if (typeof crypto !== 'undefined' && typeof crypto.randomUUID === 'function') {
    return crypto.randomUUID()
  }
  return `msg-${Date.now()}-${Math.random().toString(16).slice(2)}`
}

function formatFileSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`
  return `${(bytes / 1024 / 1024).toFixed(1)} MB`
}

function byCreatedAt(a: Message, b: Message) {
  return new Date(a.created_at).getTime() - new Date(b.created_at).getTime()
}
