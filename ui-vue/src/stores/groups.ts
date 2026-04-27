/**
 * 群聊状态管理（Pinia）
 */

import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import {
  getGroupList,
  createGroup,
  deleteGroup,
  getGroupMembers,
  joinGroup,
  getJoinRequests,
  approveJoinRequest,
  rejectJoinRequest,
  getGroup,
  removeGroupMember,
  updateGroup,
} from '@/services/group'
import { STORAGE_KEYS } from '@/constants/storageKeys'
import type {
  Group,
  CreateGroupRequest,
  UpdateGroupRequest,
  GroupMember,
  GroupJoinRequestItem,
  GroupJoinResult,
} from '@/types'
import { getErrorMessage } from '@/utils/error'

export const useGroupStore = defineStore('groups', () => {
  // 状态
  const groups = ref<Group[]>([])
  const selectedGroupId = ref<string | null>(null)
  const groupMembers = ref<Record<string, GroupMember[]>>({})
  const joinRequestsInbox = ref<GroupJoinRequestItem[]>([])
  const joinRequestsOutbox = ref<GroupJoinRequestItem[]>([])
  const loading = ref(false)
  const error = ref<string | null>(null)

  // 从 localStorage 恢复选中的群
  const initializeSelectedGroup = () => {
    const stored = localStorage.getItem(STORAGE_KEYS.SELECTED_GROUP_ID)
    if (stored) {
      selectedGroupId.value = stored
    }
  }

  // 计算属性
  const selectedGroup = computed(() => {
    return groups.value.find((g: Group) => g.group_id === selectedGroupId.value) || null
  })

  const groupsCount = computed(() => groups.value.length)

  const selectedGroupMembers = computed(() => {
    return selectedGroupId.value ? (groupMembers.value[selectedGroupId.value] || []) : []
  })

  const pendingInboxCount = computed(() => joinRequestsInbox.value.filter((item) => item.status === 'pending').length)

  const pendingOutboxCount = computed(() => joinRequestsOutbox.value.filter((item) => item.status === 'pending').length)

  // 方法：获取群列表
  const fetchGroups = async () => {
    loading.value = true
    error.value = null

    try {
      groups.value = await getGroupList()
      return groups.value
    } catch (err: any) {
      error.value = getErrorMessage(err, '获取群列表失败')
      throw err
    } finally {
      loading.value = false
    }
  }

  // 方法：创建群
  const addGroup = async (data: CreateGroupRequest) => {
    loading.value = true
    error.value = null

    try {
      const newGroup = await createGroup(data)
      groups.value.push(newGroup)

      // 自动选中新创建的群
      selectedGroupId.value = newGroup.group_id
      localStorage.setItem(STORAGE_KEYS.SELECTED_GROUP_ID, newGroup.group_id)

      return newGroup
    } catch (err: any) {
      error.value = getErrorMessage(err, '创建群失败')
      throw err
    } finally {
      loading.value = false
    }
  }

  // 方法：删除群
  const removeGroupById = async (groupId: string) => {
    loading.value = true
    error.value = null

    try {
      await deleteGroup(groupId)

      // 从列表中删除
      groups.value = groups.value.filter((g: Group) => g.group_id !== groupId)

      // 清除成员缓存
      delete groupMembers.value[groupId]

      // 如果删除的是当前选中的群，清除选中
      if (selectedGroupId.value === groupId) {
        selectedGroupId.value = groups.value.length > 0 ? groups.value[0].group_id : null
        if (selectedGroupId.value) {
          localStorage.setItem(STORAGE_KEYS.SELECTED_GROUP_ID, selectedGroupId.value)
        } else {
          localStorage.removeItem(STORAGE_KEYS.SELECTED_GROUP_ID)
        }
      }
    } catch (err: any) {
      error.value = getErrorMessage(err, '删除群失败')
      throw err
    } finally {
      loading.value = false
    }
  }

  // 方法：更新群信息
  const updateGroupInfo = async (groupId: string, data: UpdateGroupRequest) => {
    loading.value = true
    error.value = null

    try {
      const updatedGroup = await updateGroup(groupId, data)
      const index = groups.value.findIndex((group) => group.group_id === groupId)
      if (index >= 0) {
        groups.value[index] = updatedGroup
      }
      return updatedGroup
    } catch (err: any) {
      error.value = getErrorMessage(err, '更新群信息失败')
      throw err
    } finally {
      loading.value = false
    }
  }

  // 方法：获取群成员
  const fetchGroupMembers = async (groupId: string) => {
    loading.value = true
    error.value = null

    try {
      const members = await getGroupMembers(groupId)
      groupMembers.value[groupId] = members
      return members
    } catch (err: any) {
      error.value = getErrorMessage(err, '获取群成员失败')
      throw err
    } finally {
      loading.value = false
    }
  }

  // 方法：选择群
  const selectGroup = (groupId: string) => {
    const group = groups.value.find((g: Group) => g.group_id === groupId)
    if (group) {
      selectedGroupId.value = groupId
      localStorage.setItem(STORAGE_KEYS.SELECTED_GROUP_ID, groupId)
    }
  }

  // 方法：加入群
  const joinGroupByNumber = async (
    groupNumber: string,
    botId: string,
    requestReason: string
  ): Promise<GroupJoinResult> => {
    loading.value = true
    error.value = null

    try {
      const result = await joinGroup({ groupCode: groupNumber, botId, requestReason })
      if (result.result_status === 'joined') {
        const joinedGroup = await getGroup(result.group_id)
        if (!groups.value.some((group) => group.group_id === joinedGroup.group_id)) {
          groups.value.unshift(joinedGroup)
        }
      } else {
        await fetchJoinRequests('outbox')
      }
      return result
    } catch (err: any) {
      error.value = getErrorMessage(err, '加入群失败')
      throw err
    } finally {
      loading.value = false
    }
  }

  const joinGroupById = async (
    groupId: string,
    botId: string,
    requestReason: string
  ): Promise<GroupJoinResult> => {
    loading.value = true
    error.value = null

    try {
      const result = await joinGroup({ groupId, botId, requestReason })
      if (result.result_status === 'joined') {
        const joinedGroup = await getGroup(result.group_id)
        if (!groups.value.some((group) => group.group_id === joinedGroup.group_id)) {
          groups.value.unshift(joinedGroup)
        }
      } else {
        await fetchJoinRequests('outbox')
      }
      return result
    } catch (err: any) {
      error.value = getErrorMessage(err, '加入群失败')
      throw err
    } finally {
      loading.value = false
    }
  }

  const addBotToGroup = async (
    groupId: string,
    botId: string,
    requestReason: string
  ): Promise<GroupJoinResult> => {
    loading.value = true
    error.value = null

    try {
      const result = await joinGroup({ groupId, botId, requestReason })
      if (result.result_status === 'joined') {
        const members = await getGroupMembers(groupId)
        groupMembers.value[groupId] = members
      } else {
        await fetchJoinRequests('outbox')
      }
      return result
    } catch (err: any) {
      error.value = getErrorMessage(err, '添加机器人到群聊失败')
      throw err
    } finally {
      loading.value = false
    }
  }

  const removeBotFromGroup = async (groupId: string, botId: string) => {
    loading.value = true
    error.value = null

    try {
      await removeGroupMember(groupId, botId)
      groupMembers.value[groupId] = (groupMembers.value[groupId] || []).filter((member) => member.member_id !== botId)
    } catch (err: any) {
      error.value = getErrorMessage(err, '移出群聊失败')
      throw err
    } finally {
      loading.value = false
    }
  }

  const fetchJoinRequests = async (scope: 'inbox' | 'outbox') => {
    loading.value = true
    error.value = null
    try {
      const requests = await getJoinRequests(scope, 'pending')
      if (scope === 'inbox') {
        joinRequestsInbox.value = requests
      } else {
        joinRequestsOutbox.value = requests
      }
      return requests
    } catch (err: any) {
      error.value = getErrorMessage(err, '获取加群申请失败')
      throw err
    } finally {
      loading.value = false
    }
  }

  const approveJoinRequestById = async (requestId: string) => {
    loading.value = true
    error.value = null
    try {
      await approveJoinRequest(requestId)
      await Promise.all([fetchJoinRequests('inbox'), fetchJoinRequests('outbox'), fetchGroups()])
    } catch (err: any) {
      error.value = getErrorMessage(err, '同意申请失败')
      throw err
    } finally {
      loading.value = false
    }
  }

  const rejectJoinRequestById = async (requestId: string) => {
    loading.value = true
    error.value = null
    try {
      await rejectJoinRequest(requestId)
      await Promise.all([fetchJoinRequests('inbox'), fetchJoinRequests('outbox')])
    } catch (err: any) {
      error.value = getErrorMessage(err, '拒绝申请失败')
      throw err
    } finally {
      loading.value = false
    }
  }

  // 方法：清除错误
  const clearError = () => {
    error.value = null
  }

  return {
    // 状态
    groups,
    selectedGroupId,
    groupMembers,
    joinRequestsInbox,
    joinRequestsOutbox,
    loading,
    error,

    // 计算属性
    selectedGroup,
    groupsCount,
    selectedGroupMembers,
    pendingInboxCount,
    pendingOutboxCount,

    // 方法
    initializeSelectedGroup,
    fetchGroups,
    addGroup,
    removeGroupById,
    updateGroupInfo,
    fetchGroupMembers,
    selectGroup,
    joinGroupByNumber,
    joinGroupById,
    addBotToGroup,
    removeBotFromGroup,
    fetchJoinRequests,
    approveJoinRequestById,
    rejectJoinRequestById,
    clearError,
  }
})
