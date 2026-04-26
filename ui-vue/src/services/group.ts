/**
 * 群聊服务
 */

import { apiClient, unwrapCollectionResponse } from './api'
import type { Group, CreateGroupRequest, UpdateGroupRequest, GroupJoinResult, GroupMember } from '@/types'

/**
 * 获取群列表
 */
export async function getGroupList() {
  const response = await apiClient.get<Group[] | { groups: Group[] }>('/groups')
  return unwrapCollectionResponse(response)
}

/**
 * 获取单个群详情
 */
export async function getGroup(groupId: string) {
  return apiClient.get<Group>(`/groups/${groupId}`)
}

/**
 * 按群号查询群信息
 */
export async function searchGroupByCode(groupCode: string) {
  const response = await apiClient.get<Group[] | { groups: Group[] }>(
    `/groups/search?group_code=${encodeURIComponent(groupCode)}`
  )
  return unwrapCollectionResponse(response)
}

/**
 * 创建群
 */
export async function createGroup(data: CreateGroupRequest) {
  return apiClient.post<Group>('/groups', data)
}

export async function updateGroup(groupId: string, data: UpdateGroupRequest) {
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
export async function joinGroup(data: { groupId?: string; groupCode?: string; botId?: string }) {
  return apiClient.post<GroupJoinResult>('/groups/join', {
    group_id: data.groupId,
    group_code: data.groupCode,
    bot_id: data.botId,
  })
}

/**
 * Bot 离开群
 */
export async function leaveGroup(groupId: string) {
  return apiClient.delete(`/groups/${groupId}/leave`)
}

export async function removeGroupMember(groupId: string, botId: string) {
  return apiClient.delete(`/groups/${groupId}/members/${botId}`)
}

/**
 * 获取群成员
 */
export async function getGroupMembers(groupId: string) {
  const response = await apiClient.get<GroupMember[] | { members: GroupMember[] }>(`/groups/${groupId}/members`)
  return unwrapCollectionResponse(response)
}
