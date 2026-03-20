/**
 * 群聊服务
 */

import { apiClient } from './api'
import type { Group, CreateGroupRequest } from '@/types'

/**
 * 获取群列表
 */
export async function getGroupList() {
  return apiClient.get<Group[]>('/groups')
}

/**
 * 获取单个群详情
 */
export async function getGroup(groupId: string) {
  return apiClient.get<Group>(`/groups/${groupId}`)
}

/**
 * 创建群
 */
export async function createGroup(data: CreateGroupRequest) {
  return apiClient.post<Group>('/groups', data)
}

/**
 * 更新群信息
 */
export async function updateGroup(groupId: string, data: Partial<CreateGroupRequest>) {
  return apiClient.put<Group>(`/groups/${groupId}`, data)
}

/**
 * 删除群
 */
export async function deleteGroup(groupId: string) {
  return apiClient.delete(`/groups/${groupId}`)
}

/**
 * Bot 加入群
 */
export async function joinGroup(groupId: string, botId: string) {
  return apiClient.post(`/groups/${groupId}/join`, { bot_id: botId })
}

/**
 * Bot 离开群
 */
export async function leaveGroup(groupId: string, botId: string) {
  return apiClient.post(`/groups/${groupId}/leave`, { bot_id: botId })
}

/**
 * 获取群成员
 */
export async function getGroupMembers(groupId: string) {
  return apiClient.get(`/groups/${groupId}/members`)
}
