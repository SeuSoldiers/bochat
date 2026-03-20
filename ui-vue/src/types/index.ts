/**
 * 用户相关类型定义
 */
export interface User {
  id: string
  phone: string
  id_number: string
  bot_token: string
  created_at: string
}

/**
 * Bot 相关类型定义
 */
export interface Bot {
  bot_id: string
  name: string
  description?: string
  created_at: string
  created_by: string
}

export interface CreateBotRequest {
  name: string
  description?: string
}

/**
 * 群聊相关类型定义
 */
export interface Group {
  group_id: string
  group_name: string
  group_number: string
  created_at: string
  created_by: string
  member_count: number
}

export interface CreateGroupRequest {
  group_name: string
  group_number: string
}

/**
 * 消息相关类型定义
 */
export interface Message {
  message_id: string
  group_id: string
  sender_id: string
  sender_name: string
  content: string
  created_at: string
  message_type: 'text' | 'image' | 'file'
}

export interface SendMessageRequest {
  group_id: string
  content: string
  message_type?: 'text' | 'image' | 'file'
}

/**
 * API 响应类型定义
 */
export interface ApiResponse<T = any> {
  code: number
  message: string
  data?: T
}

/**
 * WebSocket 消息类型
 */
export interface WebSocketMessage {
  type: 'message' | 'group_joined' | 'member_joined' | 'member_left' | 'connection' | 'error'
  payload: any
  timestamp: string
}
