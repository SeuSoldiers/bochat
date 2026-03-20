/**
 * Bot 服务
 */

import { apiClient, unwrapCollectionResponse } from './api'
import type { Bot, CreateBotRequest, UpdateBotRequest } from '@/types'

/**
 * 获取 Bot 列表
 */
export async function getBotList() {
  const response = await apiClient.get<Bot[] | { bots: Bot[] }>('/bots')
  return unwrapCollectionResponse(response)
}

/**
 * 获取单个 Bot 详情
 */
export async function getBot(botId: string) {
  return apiClient.get<Bot>(`/bots/${botId}`)
}

/**
 * 创建 Bot
 */
export async function createBot(data: CreateBotRequest) {
  return apiClient.post<Bot>('/bots', data)
}

export async function updateBot(botId: string, data: UpdateBotRequest) {
  return apiClient.put<Bot>(`/bots/${botId}`, data)
}

/**
 * 删除 Bot
 */
export async function deleteBot(botId: string) {
  return apiClient.delete(`/bots/${botId}`)
}
