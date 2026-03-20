/**
 * 消息服务
 */

import { apiClient } from './api'
import type { Message, SendMessageRequest } from '@/types'

/**
 * 获取群消息列表
 */
export async function getMessages(groupId: string, limit: number = 50, offset: number = 0) {
  return apiClient.get<Message[]>(`/groups/${groupId}/messages`, {
    params: { limit, offset },
  })
}

/**
 * 发送消息
 */
export async function sendMessage(data: SendMessageRequest) {
  return apiClient.post<Message>('/messages', data)
}

/**
 * 删除消息
 */
export async function deleteMessage(messageId: string) {
  return apiClient.delete(`/messages/${messageId}`)
}

/**
 * 搜索消息
 */
export async function searchMessages(groupId: string, keyword: string) {
  return apiClient.get<Message[]>(`/groups/${groupId}/messages/search`, {
    params: { keyword },
  })
}
