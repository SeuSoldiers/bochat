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
  baseId?: number | null,
  limit: number = 50,
  botToken?: string
) {
  const response = await apiClient.get<Message[] | { messages: Message[] }>(`/groups/${groupId}/messages`, {
    headers: botToken ? { Authorization: `Bearer ${botToken}` } : undefined,
    params: { limit, base_id: baseId ?? undefined },
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
