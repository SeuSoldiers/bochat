/**
 * 聊天消息状态管理（Pinia）
 */

import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { getMessages, sendMessage } from '@/services/message'
import { STORAGE_KEYS } from '@/constants/storageKeys'
import type { Message, SendMessageRequest } from '@/types'
import { getErrorMessage } from '@/utils/error'

export const useChatStore = defineStore('chat', () => {
  // 状态
  const messages = ref<Message[]>([])
  const currentGroupId = ref<string | null>(null)
  const loading = ref(false)
  const sending = ref(false)
  const error = ref<string | null>(null)

  // 计算属性
  const groupMessages = computed(() => {
    return messages.value.filter((m: Message) => m.group_id === currentGroupId.value)
  })

  const messagesCount = computed(() => groupMessages.value.length)

  const normalizeMessage = (message: Message): Message => {
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
          // 保持原始字符串，不中断消息流
        }
      }
    }

    if (typeof normalized.content !== 'string' && normalized.content) {
      const content = { ...normalized.content } as Record<string, unknown>
      // 兼容后端/网关可能返回的文件字段别名
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

  // 方法：设置当前群
  const setCurrentGroup = (groupId: string | null) => {
    currentGroupId.value = groupId
    if (groupId) {
      localStorage.setItem(STORAGE_KEYS.CURRENT_GROUP_ID, groupId)
    } else {
      localStorage.removeItem(STORAGE_KEYS.CURRENT_GROUP_ID)
    }
  }

  // 方法：获取消息列表
  const fetchMessages = async (
    groupId: string,
    baseId?: number | null,
    limit: number = 50,
    botToken?: string
  ) => {
    loading.value = true
    error.value = null

    try {
      const newMessages = await getMessages(groupId, baseId, limit, botToken)
      const normalizedNewMessages = newMessages.map((m: Message) => normalizeMessage(m))

      // 合并消息（避免重复）
      const existingIds = new Set(messages.value.map((m: Message) => m.msg_id))
      const uniqueNewMessages = normalizedNewMessages.filter((m: Message) => !existingIds.has(m.msg_id))

      // 按时间升序排列，便于聊天窗口自然阅读
      messages.value = [...messages.value, ...uniqueNewMessages].sort(
        (a: Message, b: Message) => new Date(a.created_at).getTime() - new Date(b.created_at).getTime()
      )

      return normalizedNewMessages
    } catch (err: any) {
      error.value = getErrorMessage(err, '获取消息失败')
      throw err
    } finally {
      loading.value = false
    }
  }

  // 方法：发送消息
  const addMessage = async (data: SendMessageRequest, botToken: string) => {
    sending.value = true
    error.value = null

    try {
      return await sendMessage(data, botToken)
    } catch (err: any) {
      error.value = getErrorMessage(err, '发送消息失败')
      throw err
    } finally {
      sending.value = false
    }
  }

  // 方法：添加 WebSocket 接收的消息
  const addWebSocketMessage = (message: Message) => {
    const normalizedMessage = normalizeMessage(message)
    // 检查消息是否已存在
    const exists = messages.value.some((m) => m.msg_id === normalizedMessage.msg_id)
    if (!exists) {
      messages.value.push(normalizedMessage)
      messages.value.sort(
        (a: Message, b: Message) => new Date(a.created_at).getTime() - new Date(b.created_at).getTime()
      )
    }
  }

  // 方法：清除群消息
  const clearGroupMessages = (groupId: string) => {
    messages.value = messages.value.filter((m: Message) => m.group_id !== groupId)
  }

  // 方法：清空所有消息
  const clearAllMessages = () => {
    messages.value = []
  }

  // 方法：清除错误
  const clearError = () => {
    error.value = null
  }

  return {
    // 状态
    messages,
    currentGroupId,
    loading,
    sending,
    error,

    // 计算属性
    groupMessages,
    messagesCount,

    // 方法
    setCurrentGroup,
    fetchMessages,
    addMessage,
    addWebSocketMessage,
    clearGroupMessages,
    clearAllMessages,
    clearError,
  }
})
