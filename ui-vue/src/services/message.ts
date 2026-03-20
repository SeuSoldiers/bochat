/**
 * 消息服务
 */

import { apiClient, unwrapCollectionResponse } from './api'
import type { Message, SendMessageRequest } from '@/types'

/**
 * 获取群消息列表
 */
export async function getMessages(
  groupId: string,
  botId?: string,
  limit: number = 50,
  offset: number = 0,
  botToken?: string
) {
  const response = await apiClient.get<Message[] | { messages: Message[] }>(`/groups/${groupId}/messages`, {
    headers: botToken ? { Authorization: `Bearer ${botToken}` } : undefined,
    params: { limit, offset, bot_id: botId },
  })
  return unwrapCollectionResponse(response)
}

/**
 * 发送消息
 */
export async function sendMessage(data: SendMessageRequest, botToken: string) {
  return apiClient.post<Message>('/message/send', data, {
    headers: { Authorization: `Bearer ${botToken}` },
  })
}
