/**
 * 后端 WebSocket 握手尚未实现，前端在这里显式降级为 no-op。
 */

import { ref } from 'vue'

export function useWebSocket(_token: string | null) {
  const ws = ref<WebSocket | null>(null)
  const isConnected = ref(false)
  const reconnectAttempts = ref(0)

  const connect = () => {
    console.warn('WebSocket is disabled because the backend endpoint is not implemented yet')
  }

  const disconnect = () => {
    ws.value = null
    isConnected.value = false
  }

  const send = (_message: unknown) => {
    console.warn('WebSocket send is disabled because the backend endpoint is not implemented yet')
  }

  return {
    ws,
    isConnected,
    reconnectAttempts,
    connect,
    disconnect,
    send,
  }
}
