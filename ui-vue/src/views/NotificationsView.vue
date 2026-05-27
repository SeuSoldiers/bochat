<template>
  <main class="notifications-main">
        <section class="notifications-card dashboard-surface">
          <header class="card-header">
            <h2>通知中心</h2>
            <span class="meta">{{ notificationStore.stats.pending_count }}/{{ notificationStore.stats.unread_count }}</span>
          </header>

          <div v-if="notificationStore.error" class="error-box">{{ notificationStore.error }}</div>

          <div class="section">
            <p class="section-title">待审批</p>
            <div v-if="notificationStore.pendingItems.length > 0" class="list">
              <article
                v-for="item in notificationStore.pendingItems"
                :key="item.notification_id"
                class="item pending"
              >
                <div class="item-main">
                  <p class="item-title">{{ item.title }}</p>
                  <p class="item-content">{{ item.content }}</p>
                </div>
                <div class="item-actions">
                  <button class="btn btn-primary" :disabled="notificationStore.loading" @click="approve(item.notification_id)">
                    同意
                  </button>
                  <button class="btn btn-secondary" :disabled="notificationStore.loading" @click="reject(item.notification_id)">
                    拒绝
                  </button>
                </div>
              </article>
            </div>
            <p v-else class="empty-tip">暂无待审批项</p>
          </div>

          <div class="section">
            <p class="section-title">提醒</p>
            <div v-if="notificationStore.reminderItems.length > 0" class="list">
              <article
                v-for="item in notificationStore.reminderItems"
                :key="item.notification_id"
                class="item"
                :class="{ unread: !item.is_read }"
              >
                <div class="item-main">
                  <p class="item-title">{{ item.title }}</p>
                  <template v-if="item.kind === 'file_scan_alert' && fileAlertDetail(item)">
                    <div class="file-alert-detail">
                      <span class="file-alert-field">时间：{{ fileAlertDetail(item)!.time }}</span>
                      <span class="file-alert-field">文件ID：{{ fileAlertDetail(item)!.fileId }}</span>
                      <span v-if="fileAlertDetail(item)!.filename" class="file-alert-field">文件名：{{ fileAlertDetail(item)!.filename }}</span>
                      <span v-if="fileAlertDetail(item)!.groupName" class="file-alert-field">群组：{{ fileAlertDetail(item)!.groupName }}</span>
                      <span v-if="fileAlertDetail(item)!.botName" class="file-alert-field">Bot：{{ fileAlertDetail(item)!.botName }}<span v-if="fileAlertDetail(item)!.botId"> ({{ fileAlertDetail(item)!.botId }})</span></span>
                      <span v-if="fileAlertDetail(item)!.riskLevel" class="file-alert-field">风险等级：{{ fileAlertDetail(item)!.riskLevel }}</span>
                    </div>
                  </template>
                  <p v-else class="item-content">{{ item.content }}</p>
                </div>
                <button
                  v-if="!item.is_read"
                  class="btn btn-ghost"
                  :disabled="notificationStore.loading"
                  @click="markRead(item.notification_id)"
                >
                  已读
                </button>
              </article>
            </div>
            <p v-else class="empty-tip">暂无提醒</p>
          </div>
        </section>
    </main>
</template>

<script setup lang="ts">
import { onMounted } from 'vue'
import { useNotificationStore } from '@/stores/notifications'
import type { NotificationItem } from '@/types'

const notificationStore = useNotificationStore()

const formatTime = (iso: string) => {
  const d = new Date(iso)
  return d.toLocaleString('zh-CN', { month: '2-digit', day: '2-digit', hour: '2-digit', minute: '2-digit', second: '2-digit' })
}

const fileAlertDetail = (item: NotificationItem) => {
  if (item.kind !== 'file_scan_alert' || !item.action_payload) return null
  const p = item.action_payload
  const alertType = p.alert_type as string | undefined
  const fileId = p.file_id as string | undefined
  const filename = p.filename as string | undefined
  const riskLevel = p.risk_level as string | undefined
  const time = formatTime(item.created_at)

  if (alertType === 'group') {
    const groupName = p.group_name as string | undefined
    const senderBotName = p.sender_bot_name as string | undefined
    const senderBotId = p.sender_bot_id as string | undefined
    return { alertType, time, fileId, filename, riskLevel, groupName, botName: senderBotName, botId: senderBotId }
  }

  if (alertType === 'bot') {
    const uploaderBotName = p.uploader_bot_name as string | undefined
    const uploaderBotId = p.uploader_bot_id as string | undefined
    return { alertType, time, fileId, filename, riskLevel, botName: uploaderBotName, botId: uploaderBotId }
  }

  return { alertType, time, fileId, filename, riskLevel }
}

const approve = async (notificationId: string) => {
  await notificationStore.handleApprove(notificationId)
}

const reject = async (notificationId: string) => {
  await notificationStore.handleReject(notificationId)
}

const markRead = async (notificationId: string) => {
  await notificationStore.handleMarkRead(notificationId)
}

onMounted(async () => {
  await notificationStore.fetchNotifications({ status: 'all', limit: 100 })
})
</script>

<style scoped>
.notifications-main {
  width: 100%;
}

.notifications-card {
  padding: 16px;
  display: flex;
  flex-direction: column;
  gap: 14px;
  min-height: 100%;
}

.card-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.card-header h2 {
  margin: 0;
  font-size: 20px;
  color: #1f1f1f;
}

.meta {
  border-radius: 999px;
  background: #e4e4e4;
  color: #4a4a4a;
  padding: 2px 10px;
  font-size: 12px;
}

.section {
  border: 1px solid #d6d6d6;
  border-radius: 8px;
  padding: 10px;
  background: #f1f1f1;
}

.section-title {
  margin: 0 0 8px;
  color: #4f4f4f;
  font-size: 12px;
}

.list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.item {
  border: 1px solid #d0d0d0;
  border-radius: 8px;
  padding: 10px;
  background: #f7f7f7;
  display: flex;
  gap: 10px;
  justify-content: space-between;
  align-items: flex-start;
}

.item.unread {
  border-color: #909090;
}

.item-main {
  min-width: 0;
}

.item-title {
  margin: 0;
  font-size: 13px;
  color: #2d2d2d;
}

.item-content {
  margin: 4px 0 0;
  font-size: 12px;
  color: #626262;
  line-height: 1.4;
  white-space: pre-wrap;
}

.file-alert-detail {
  margin-top: 6px;
  display: flex;
  flex-direction: column;
  gap: 3px;
}

.file-alert-field {
  font-size: 12px;
  color: #626262;
  line-height: 1.4;
}

.item-actions {
  display: flex;
  gap: 8px;
}

.btn {
  height: 30px;
  border-radius: 6px;
  border: 1px solid transparent;
  background: #ececec;
  color: #303030;
  padding: 0 10px;
  font-size: 12px;
  cursor: pointer;
}

.btn-primary {
  background: #2f2f2f;
  color: #f4f4f4;
}

.btn-secondary {
  border-color: #cdcdcd;
  background: #efefef;
}

.btn-ghost {
  border-color: #d2d2d2;
  background: #f2f2f2;
}

.btn:disabled {
  opacity: 0.65;
  cursor: not-allowed;
}

.empty-tip {
  margin: 0;
  color: #8a8a8a;
  font-size: 12px;
}

.error-box {
  padding: 8px 10px;
  border-radius: 8px;
  border: 1px solid #efc7c1;
  background: #fff0ee;
  color: #a6453e;
  font-size: 12px;
}
</style>

