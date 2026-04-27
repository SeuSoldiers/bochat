import { computed, ref } from 'vue'
import { defineStore } from 'pinia'
import {
  approveNotification,
  getNotifications,
  markNotificationRead,
  rejectNotification,
  type NotificationListQuery,
} from '@/services/notification'
import type { NotificationItem, NotificationStats } from '@/types'
import { getErrorMessage } from '@/utils/error'

export const useNotificationStore = defineStore('notifications', () => {
  const items = ref<NotificationItem[]>([])
  const stats = ref<NotificationStats>({
    unread_count: 0,
    pending_count: 0,
  })
  const loading = ref(false)
  const error = ref<string | null>(null)

  const pendingItems = computed(() => items.value.filter((item) => item.requires_action && !item.is_resolved))
  const reminderItems = computed(() =>
    items.value.filter((item) => !item.requires_action || item.kind === 'bot_removed_from_group')
  )

  const fetchNotifications = async (query: NotificationListQuery = { status: 'all', limit: 100 }) => {
    loading.value = true
    error.value = null
    try {
      const response = await getNotifications(query)
      items.value = response.notifications || []
      stats.value = response.stats || { unread_count: 0, pending_count: 0 }
      return items.value
    } catch (err: any) {
      error.value = getErrorMessage(err, '获取通知失败')
      throw err
    } finally {
      loading.value = false
    }
  }

  const refreshStats = async () => {
    try {
      const response = await getNotifications({ status: 'unread', limit: 1, offset: 0 })
      stats.value = response.stats || { unread_count: 0, pending_count: 0 }
      return stats.value
    } catch (err: any) {
      error.value = getErrorMessage(err, '刷新通知统计失败')
      throw err
    }
  }

  const handleApprove = async (notificationId: string, note?: string) => {
    loading.value = true
    error.value = null
    try {
      await approveNotification(notificationId, note)
      await fetchNotifications({ status: 'all', limit: 100 })
    } catch (err: any) {
      error.value = getErrorMessage(err, '同意申请失败')
      throw err
    } finally {
      loading.value = false
    }
  }

  const handleReject = async (notificationId: string, note?: string) => {
    loading.value = true
    error.value = null
    try {
      await rejectNotification(notificationId, note)
      await fetchNotifications({ status: 'all', limit: 100 })
    } catch (err: any) {
      error.value = getErrorMessage(err, '拒绝申请失败')
      throw err
    } finally {
      loading.value = false
    }
  }

  const handleMarkRead = async (notificationId: string) => {
    loading.value = true
    error.value = null
    try {
      await markNotificationRead(notificationId)
      const target = items.value.find((item) => item.notification_id === notificationId)
      if (target) {
        target.is_read = true
      }
      await refreshStats()
    } catch (err: any) {
      error.value = getErrorMessage(err, '标记已读失败')
      throw err
    } finally {
      loading.value = false
    }
  }

  return {
    items,
    stats,
    loading,
    error,
    pendingItems,
    reminderItems,
    fetchNotifications,
    refreshStats,
    handleApprove,
    handleReject,
    handleMarkRead,
  }
})

