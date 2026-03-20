import { ref, watch, onMounted, onUnmounted, type Ref } from 'vue'
import { useChatStore } from '@/stores/chat'
import type { WebSocketMessage, Message } from '@/types'

const WS_BASE_URL = import.meta.env.VITE_WS_BASE_URL || 'ws://localhost:8080/ws'

export function useWebSocket(token: Ref<string | null>) {
  const ws = ref<WebSocket | null>(null)
  const isConnected = ref(false)
  const reconnectAttempts = ref(0)
  let reconnectTimer: number | null = null

  const chatStore = useChatStore()

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
      const message = JSON.parse(event.data) as WebSocketMessage
      if (message.type === 'message') {
        chatStore.addWebSocketMessage(message.payload as Message)
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
