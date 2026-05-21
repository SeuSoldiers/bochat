import { ref, watch, onMounted, onUnmounted, type Ref } from 'vue'
import { useChatStore } from '@/stores/chat'
import type { WebSocketMessage, Message } from '@/types'

const WS_BASE_URL = import.meta.env.VITE_WS_BASE_URL || `${window.location.protocol === 'https:' ? 'wss:' : 'ws:'}//${window.location.hostname}:48080/ws`

export function useWebSocket(token: Ref<string | null>, onMessage?: (message: Message) => void) {
  const ws = ref<WebSocket | null>(null)
  const isConnected = ref(false)
  const reconnectAttempts = ref(0)
  let reconnectTimer: number | null = null

  const chatStore = useChatStore()

  const asMessagePayload = (payload: unknown): Message | null => {
    if (!payload || typeof payload !== 'object') {
      return null
    }

    const candidate = payload as Record<string, unknown>

    if ('msg_id' in candidate && 'group_id' in candidate && 'sender_id' in candidate) {
      return candidate as unknown as Message
    }

    if (candidate.message && typeof candidate.message === 'object') {
      return candidate.message as Message
    }

    if (candidate.data && typeof candidate.data === 'object') {
      const data = candidate.data as Record<string, unknown>
      if ('msg_id' in data && 'group_id' in data && 'sender_id' in data) {
        return data as unknown as Message
      }
    }

    return null
  }

  const connect = () => {
    if (!token.value) {
      return
    }

    if (ws.value && (ws.value.readyState === WebSocket.OPEN || ws.value.readyState === WebSocket.CONNECTING)) {
      return
    }

    ws.value = new WebSocket(`${WS_BASE_URL}?token=${encodeURIComponent(token.value)}`)

    ws.value.onopen = () => {
      isConnected.value = true
      reconnectAttempts.value = 0
    }

    ws.value.onmessage = (event) => {
      try {
        const message = JSON.parse(event.data) as WebSocketMessage
        if (message.type !== 'message') {
          return
        }

        const payload = asMessagePayload(message.payload)
        if (!payload) {
          return
        }
        if (onMessage) {
          onMessage(payload)
        } else {
          chatStore.addWebSocketMessage(payload)
        }
      } catch (error) {
        console.warn('Invalid WebSocket message payload:', error)
      }
    }

    ws.value.onclose = () => {
      isConnected.value = false
      ws.value = null
      scheduleReconnect()
    }

    ws.value.onerror = () => {
      isConnected.value = false
    }
  }

  const scheduleReconnect = () => {
    if (!token.value) {
      return
    }

    reconnectAttempts.value += 1
    const delay = Math.min(1000 * reconnectAttempts.value, 5000)
    reconnectTimer = window.setTimeout(() => {
      connect()
    }, delay)
  }

  const disconnect = () => {
    if (reconnectTimer) {
      window.clearTimeout(reconnectTimer)
      reconnectTimer = null
    }

    ws.value?.close()
    ws.value = null
    isConnected.value = false
  }

  watch(token, (newToken) => {
    if (newToken) {
      connect()
    } else {
      disconnect()
    }
  }, { immediate: true })

  onMounted(connect)
  onUnmounted(disconnect)

  return {
    ws,
    isConnected,
    reconnectAttempts,
    connect,
    disconnect,
  }
}
