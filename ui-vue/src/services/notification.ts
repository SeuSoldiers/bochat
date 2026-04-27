import { apiClient } from '@/services/api'
import type { NotificationItem, NotificationStats } from '@/types'

export interface NotificationListQuery {
  status?: 'all' | 'unread' | 'pending'
  kind?: string
  limit?: number
  offset?: number
}

export interface NotificationListResponse {
  notifications: NotificationItem[]
  stats: NotificationStats
  limit: number
  offset: number
}

export async function getNotifications(query: NotificationListQuery = {}) {
  return apiClient.get<NotificationListResponse>('/notifications', { params: query })
}

export async function markNotificationRead(notificationId: string) {
  return apiClient.post(`/notifications/${notificationId}/read`, {})
}

export async function approveNotification(notificationId: string, note?: string) {
  return apiClient.post(`/notifications/${notificationId}/approve`, { note })
}

export async function rejectNotification(notificationId: string, note?: string) {
  return apiClient.post(`/notifications/${notificationId}/reject`, { note })
}

