/**
 * 用户相关类型定义
 */
export interface User {
  name: string
  avatar_url?: string | null
  created_at?: string
  updated_at?: string
}

/**
 * Bot 相关类型定义
 */
export interface Bot {
  bot_id: string
  owner_id: string
  name: string
  description?: string
  avatar_url?: string
  status: string
  token: string
  created_at: string
  updated_at: string
}

export interface BotSearchItem {
  bot_id: string
  owner_id: string
  name: string
  avatar_url?: string
  status: string
}

export interface CreateBotRequest {
  name: string
  description?: string
  avatar_url?: string
}

export interface UpdateBotRequest {
  name: string
  description?: string
  avatar_url?: string
}

/**
 * 群聊相关类型定义
 */
export interface Group {
  group_id: string
  group_code?: string
  creator_id: string
  name: string
  description?: string
  avatar_url?: string
  is_public: boolean
  status: string
  created_at: string
  updated_at: string
}

export interface CreateGroupRequest {
  name: string
  group_code?: string
  description?: string
  avatar_url?: string
  bot_id?: string
  is_public?: boolean
}

export interface UpdateGroupRequest {
  name: string
  group_code?: string
  description?: string
  avatar_url?: string
  is_public?: boolean
}

export interface GroupJoinResult {
  message: string
  group_id: string
  bot_id: string
  result_status?: 'joined' | 'pending_approval'
  request_id?: string
  approver_user_id?: string
  request_type?: 'bot_owner_approval' | 'group_owner_approval'
}

export interface GroupMember {
  group_id: string
  member_id: string
  member_type: string
  joined_at: string
  bot_name?: string
  owner_id?: string
}

export interface GroupJoinRequestItem {
  request_id: string
  group_id: string
  group_name: string
  group_code?: string
  bot_id: string
  bot_name: string
  bot_owner_id: string
  requester_user_id: string
  approver_user_id: string
  request_type: 'bot_owner_approval' | 'group_owner_approval' | string
  request_reason: string
  status: 'pending' | 'approved' | 'rejected' | string
  review_note?: string
  created_at: string
  updated_at: string
  reviewed_at?: string
}

/**
 * 消息相关类型定义
 */
export interface Message {
  msg_id: number
  group_id: string
  sender_id: string
  sender_name?: string
  sender_avatar_url?: string
  content: {
    text?: string
    url?: string
    filename?: string
    [key: string]: unknown
  } | string
  created_at: string
  msg_type: 'text' | 'file'
}

export interface SendMessageRequest {
  group_id: string
  content: {
    text?: string
    url?: string
    filename?: string
    [key: string]: unknown
  }
  msg_type?: 'text' | 'file'
  idempotency_key: string
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
