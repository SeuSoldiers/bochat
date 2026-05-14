import { apiClient, unwrapCollectionResponse } from './api'
import type { Bot, Group } from '@/types'

function botAuthHeaders(botToken: string) {
  return {
    Authorization: `Bearer ${botToken}`,
  }
}

export async function getCurrentBotProfile(botToken: string) {
  return apiClient.get<Bot>('/bot/profile', {
    headers: botAuthHeaders(botToken),
  })
}

export async function getBotVisibleGroups(botToken: string) {
  const response = await apiClient.get<Group[] | { groups: Group[] }>('/bot/groups', {
    headers: botAuthHeaders(botToken),
  })
  return unwrapCollectionResponse(response)
}
