<template>
  <div class="dashboard">
    <header class="dash-header">
      <div>
        <h1>你好，{{ authStore.userName ?? '用户' }}</h1>
        <p class="dash-date">{{ today }}</p>
      </div>
    </header>

    <div class="stats-row">
      <div class="stat-card">
        <div class="stat-icon bot-icon">
          <Bot />
        </div>
        <div class="stat-body">
          <span class="stat-value">{{ botStore.bots.length }}</span>
          <span class="stat-label">Bot</span>
        </div>
      </div>
      <div class="stat-card">
        <div class="stat-icon group-icon">
          <Users />
        </div>
        <div class="stat-body">
          <span class="stat-value">{{ groupStore.groups.length }}</span>
          <span class="stat-label">群组</span>
        </div>
      </div>
      <div class="stat-card">
        <div class="stat-icon notify-icon">
          <Bell />
        </div>
        <div class="stat-body">
          <span class="stat-value">{{ notificationStore.stats.pending_count }}</span>
          <span class="stat-label">待处理</span>
        </div>
      </div>
    </div>

    <section class="quick-actions">
      <h2>快捷操作</h2>
      <div class="actions-row">
        <button type="button" class="action-card" @click="router.push('/bots')">
          <Plus class="action-icon" />
          <span class="action-title">创建 Bot</span>
          <span class="action-desc">创建一个新的聊天机器人</span>
        </button>
        <button type="button" class="action-card" @click="router.push('/groups')">
          <Users class="action-icon" />
          <span class="action-title">管理群组</span>
          <span class="action-desc">创建或加入一个群组</span>
        </button>
        <button type="button" class="action-card" @click="router.push('/chat')">
          <MessageSquare class="action-icon" />
          <span class="action-title">进入会话</span>
          <span class="action-desc">选择一个群组开始聊天</span>
        </button>
      </div>
    </section>

    <section v-if="needsOnboarding" class="onboarding">
      <h2>新手指引</h2>
      <div class="onboarding-steps">
        <div class="step" :class="{ done: botStore.bots.length > 0 }">
          <span class="step-num">1</span>
          <div class="step-body">
            <span class="step-title">创建 Bot</span>
            <span class="step-desc">Bot 是你在群聊中的发言人，每个 Bot 有独立的身份和 token</span>
          </div>
          <button
            v-if="botStore.bots.length === 0"
            type="button"
            class="step-action"
            @click="router.push('/bots')"
          >
            去创建
          </button>
          <CheckCircle v-else class="step-done" />
        </div>
        <div class="step" :class="{ done: groupStore.groups.length > 0 }">
          <span class="step-num">2</span>
          <div class="step-body">
            <span class="step-title">加入群组</span>
            <span class="step-desc">创建或加入一个群组，Bot 需要群组才能发言</span>
          </div>
          <button
            v-if="groupStore.groups.length === 0"
            type="button"
            class="step-action"
            @click="router.push('/groups')"
          >
            去创建
          </button>
          <CheckCircle v-else class="step-done" />
        </div>
        <div class="step" :class="{ done: botStore.bots.length > 0 && groupStore.groups.length > 0 }">
          <span class="step-num">3</span>
          <div class="step-body">
            <span class="step-title">开始聊天</span>
            <span class="step-desc">选择 Bot 和群组，开始实时对话</span>
          </div>
          <button
            v-if="botStore.bots.length > 0 && groupStore.groups.length > 0"
            type="button"
            class="step-action primary"
            @click="router.push('/chat')"
          >
            开始聊天
          </button>
          <span v-else class="step-wait">等待前置步骤完成</span>
        </div>
      </div>
    </section>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { Bot, Users, Bell, Plus, MessageSquare, CheckCircle } from 'lucide-vue-next'
import { useAuthStore } from '@/stores/auth'
import { useBotStore } from '@/stores/bots'
import { useGroupStore } from '@/stores/groups'
import { useNotificationStore } from '@/stores/notifications'

const router = useRouter()
const authStore = useAuthStore()
const botStore = useBotStore()
const groupStore = useGroupStore()
const notificationStore = useNotificationStore()

const today = computed(() => {
  return new Date().toLocaleDateString('zh-CN', {
    year: 'numeric',
    month: 'long',
    day: 'numeric',
    weekday: 'long',
  })
})

const needsOnboarding = computed(() => {
  return botStore.bots.length === 0 || groupStore.groups.length === 0
})

onMounted(async () => {
  await Promise.all([
    botStore.fetchBots(),
    groupStore.fetchGroups(),
    notificationStore.refreshStats(),
  ])
})
</script>

<style scoped>
.dashboard {
  max-width: 800px;
}

.dash-header {
  margin-bottom: 24px;
}

.dash-header h1 {
  margin: 0;
  font-size: 22px;
  font-weight: 700;
  color: #1a1a1a;
}

.dash-date {
  margin: 4px 0 0;
  font-size: 13px;
  color: #6f6f6f;
}

.stats-row {
  display: flex;
  gap: 14px;
  margin-bottom: 28px;
}

.stat-card {
  flex: 1;
  min-width: 0;
  display: flex;
  align-items: center;
  gap: 14px;
  padding: 16px 18px;
  background: #ffffff;
  border: 1px solid #d0d0d0;
  border-radius: 8px;
}

.stat-icon {
  width: 44px;
  height: 44px;
  border-radius: 8px;
  display: grid;
  place-items: center;
  flex-shrink: 0;
}

.stat-icon svg {
  width: 22px;
  height: 22px;
}

.bot-icon {
  background: #e8f0fe;
  color: #2f6fbf;
}

.group-icon {
  background: #e6f4ea;
  color: #2f8f4e;
}

.notify-icon {
  background: #fff8e6;
  color: #a57d11;
}

.stat-body {
  display: flex;
  flex-direction: column;
}

.stat-value {
  font-size: 24px;
  font-weight: 700;
  color: #1f1f1f;
  line-height: 1;
}

.stat-label {
  font-size: 12px;
  color: #6f6f6f;
  margin-top: 4px;
}

.quick-actions h2,
.onboarding h2 {
  margin: 0 0 12px;
  font-size: 15px;
  font-weight: 700;
  color: #1f1f1f;
}

.actions-row {
  display: flex;
  gap: 12px;
  margin-bottom: 28px;
}

.action-card {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
  padding: 20px 16px;
  background: #ffffff;
  border: 1px solid #d0d0d0;
  border-radius: 8px;
  cursor: pointer;
  transition: all 0.15s ease;
  text-align: center;
}

.action-card:hover {
  border-color: #2f2f2f;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.08);
}

.action-icon {
  width: 28px;
  height: 28px;
  stroke: #2f2f2f;
  stroke-width: 2;
}

.action-title {
  font-size: 14px;
  font-weight: 700;
  color: #1f1f1f;
}

.action-desc {
  font-size: 12px;
  color: #6f6f6f;
}

.onboarding {
  background: #ffffff;
  border: 1px solid #d0d0d0;
  border-radius: 8px;
  padding: 20px;
}

.onboarding-steps {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.step {
  display: flex;
  align-items: center;
  gap: 14px;
  padding: 14px 16px;
  background: #f5f5f5;
  border: 1px solid #e0e0e0;
  border-radius: 8px;
}

.step.done {
  background: #f0faf3;
  border-color: #c8e6d0;
}

.step-num {
  width: 32px;
  height: 32px;
  border-radius: 50%;
  background: #d0d0d0;
  color: #6f6f6f;
  display: grid;
  place-items: center;
  font-size: 14px;
  font-weight: 700;
  flex-shrink: 0;
}

.step.done .step-num {
  background: #2f8f4e;
  color: #fff;
}

.step-body {
  flex: 1;
  min-width: 0;
}

.step-title {
  display: block;
  font-size: 14px;
  font-weight: 700;
  color: #1f1f1f;
}

.step-desc {
  display: block;
  font-size: 12px;
  color: #6f6f6f;
  margin-top: 2px;
}

.step-action {
  padding: 6px 14px;
  border: 1px solid #2f2f2f;
  border-radius: 6px;
  background: #2f2f2f;
  color: #fff;
  font-size: 12px;
  font-weight: 600;
  cursor: pointer;
  white-space: nowrap;
  flex-shrink: 0;
}

.step-action:hover {
  background: #3a3a3a;
}

.step-action.primary {
  background: #2f8f4e;
  border-color: #2f8f4e;
}

.step-done {
  width: 22px;
  height: 22px;
  stroke: #2f8f4e;
  stroke-width: 2;
  flex-shrink: 0;
}

.step-wait {
  font-size: 12px;
  color: #9f9f9f;
  flex-shrink: 0;
}

@media (max-width: 768px) {
  .stats-row {
    flex-direction: column;
    gap: 10px;
  }

  .actions-row {
    flex-direction: column;
  }

  .step {
    flex-wrap: wrap;
  }

  .step-body {
    min-width: 200px;
  }
}
</style>
