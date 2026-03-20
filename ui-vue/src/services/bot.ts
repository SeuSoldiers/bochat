/**
 * Bot 服务
 */

import { apiClient } from './api'
import type { Bot, CreateBotRequest } from '@/types'

/**
 * 获取 Bot 列表
 */
export async function getBotList() {
  return apiClient.get<Bot[]>('/bots')
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

/**
 * 更新 Bot
 */
export async function updateBot(botId: string, data: Partial<CreateBotRequest>) {
  return apiClient.put<Bot>(`/bots/${botId}`, data)
}

/**
 * 删除 Bot
 */
export async function deleteBot(botId: string) {
  return apiClient.delete(`/bots/${botId}`)
}

/**
 * 获取 Bot 加入的群列表
 */
export async function getBotGroups(botId: string) {
  return apiClient.get(`/bots/${botId}/groups`)
}
