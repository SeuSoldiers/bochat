/**
 * 消息服务
 */

import { apiClient, unwrapCollectionResponse } from './api'
import type { Message, SendMessageRequest } from '@/types'

/**
 * 获取群消息列表
 */
export async function getMessages(groupId: string, botId?: string, limit: number = 50, offset: number = 0) {
  const response = await apiClient.get<Message[] | { messages: Message[] }>(`/groups/${groupId}/messages`, {
    params: { limit, offset, bot_id: botId },
  })
  return unwrapCollectionResponse(response)
}

/**
 * 发送消息
 */
export async function sendMessage(data: SendMessageRequest) {
  return apiClient.post<Message>('/message/send', data)
}
