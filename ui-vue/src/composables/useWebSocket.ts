/**
 * WebSocket 连接管理（Composition API）
 */

import { ref, onMounted, onUnmounted, watch } from 'vue'
import { useChatStore } from '@/stores/chat'
import { useGroupStore } from '@/stores/groups'
import { useAuthStore } from '@/stores/auth'
import type { WebSocketMessage, Message } from '@/types'

const WS_BASE_URL = import.meta.env.VITE_WS_BASE_URL || 'ws://localhost:8080/ws'

export function useWebSocket(token: string | null) {
  const ws = ref<WebSocket | null>(null)
  const isConnected = ref(false)
  const reconnectAttempts = ref(0)
  const maxReconnectAttempts = 10
  const reconnectDelay = ref(1000)

  const chatStore = useChatStore()
  const groupStore = useGroupStore()
  const authStore = useAuthStore()

  /**
   * 连接 WebSocket
   */
  const connect = () => {
    if (!token) {
      console.warn('No token available for WebSocket connection')
      return
    }

    if (ws.value?.readyState === WebSocket.OPEN) {
      return // 已连接
    }

    try {
      const wsUrl = `${WS_BASE_URL}?token=${token}`
      ws.value = new WebSocket(wsUrl)

      ws.value.onopen = () => {
        console.log('WebSocket connected')
        isConnected.value = true
        reconnectAttempts.value = 0
        reconnectDelay.value = 1000
      }

      ws.value.onmessage = handleMessage
      ws.value.onerror = handleError
      ws.value.onclose = handleClose
    } catch (error) {
      console.error('Failed to create WebSocket connection:', error)
      scheduleReconnect()
    }
  }

  /**
   * 处理 WebSocket 消息
   */
  const handleMessage = (event: MessageEvent) => {
    try {
      const message = JSON.parse(event.data) as WebSocketMessage

      switch (message.type) {
        case 'message':
          // 新消息
          if (message.payload) {
            chatStore.addWebSocketMessage(message.payload as Message)
          }
          break

        case 'group_joined':
          // 群加入成功
          console.log('Bot joined group:', message.payload)
          groupStore.fetchGroups()
          break

        case 'member_joined':
          // 成员加入
          console.log('Member joined:', message.payload)
          if (chatStore.currentGroupId) {
            groupStore.fetchGroupMembers(chatStore.currentGroupId)
          }
          break

        case 'member_left':
          // 成员离开
          console.log('Member left:', message.payload)
          if (chatStore.currentGroupId) {
            groupStore.fetchGroupMembers(chatStore.currentGroupId)
          }
          break

        case 'connection':
          // 连接确认
          console.log('WebSocket connection confirmed')
          break

        case 'error':
          // 错误消息
          console.error('WebSocket error:', message.payload)
          break

        default:
          console.log('Unknown message type:', message.type)
      }
    } catch (error) {
      console.error('Failed to handle WebSocket message:', error)
    }
  }

  /**
   * 处理 WebSocket 错误
   */
  const handleError = (event: Event) => {
    console.error('WebSocket error:', event)
    isConnected.value = false
  }

  /**
   * 处理连接关闭
   */
  const handleClose = () => {
    console.log('WebSocket closed')
    isConnected.value = false
    scheduleReconnect()
  }

  /**
   * 安排重新连接
   */
  const scheduleReconnect = () => {
    if (reconnectAttempts.value < maxReconnectAttempts) {
      reconnectAttempts.value++
      console.log(`Attempting to reconnect (${reconnectAttempts.value}/${maxReconnectAttempts})...`)

      setTimeout(() => {
        connect()
      }, reconnectDelay.value)

      // 指数退避
      reconnectDelay.value = Math.min(reconnectDelay.value * 2, 30000)
    } else {
      console.error('Max reconnection attempts reached')
    }
  }

  /**
   * 断开连接
   */
  const disconnect = () => {
    if (ws.value) {
      ws.value.close()
      ws.value = null
    }
    isConnected.value = false
  }

  /**
   * 发送消息
   */
  const send = (message: any) => {
    if (ws.value?.readyState === WebSocket.OPEN) {
      ws.value.send(JSON.stringify(message))
    } else {
      console.warn('WebSocket is not connected')
    }
  }

  // 监听 token 变化
  watch(
    () => authStore.token,
    (newToken) => {
      if (newToken) {
        connect()
      } else {
        disconnect()
      }
    }
  )

  // 生命周期：挂载时连接
  onMounted(() => {
    if (token) {
      connect()
    }
  })

  // 生命周期：卸载时断开连接
  onUnmounted(() => {
    disconnect()
  })

  return {
    ws,
    isConnected,
    reconnectAttempts,
    connect,
    disconnect,
    send,
  }
}
