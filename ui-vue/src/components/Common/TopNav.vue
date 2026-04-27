<template>
  <aside class="side-nav">
    <div class="brand">
      <h1>BoChat</h1>
      <div class="brand-status" role="status" aria-live="polite">
        <span class="status-dot" aria-hidden="true"></span>
      </div>
    </div>

    <nav class="main-menu">
      <router-link
        :to="{ path: '/home', query: { tab: 'bots' } }"
        class="menu-item"
        :class="{ active: isHomeTab('bots') }"
      >
        <span class="item-icon" aria-hidden="true">
          <svg viewBox="0 0 24 24" fill="none">
            <rect x="4.5" y="5.5" width="15" height="14" rx="3" />
            <path d="M9 10.5h6M12 8v5" />
          </svg>
        </span>
        <span class="menu-label">控制面板</span>
        <span class="menu-meta">{{ activeBotsCount }}/{{ botStore.bots.length }}</span>
      </router-link>
      <router-link
        :to="{ path: '/home', query: { tab: 'groups' } }"
        class="menu-item"
        :class="{ active: isHomeTab('groups') }"
      >
        <span class="item-icon" aria-hidden="true">
          <svg viewBox="0 0 24 24" fill="none">
            <rect x="4.5" y="6.5" width="15" height="11" rx="2.8" />
            <path d="M9 10.5h6M9 13.5h4" />
          </svg>
        </span>
        <span class="menu-label">群组空间</span>
        <span class="menu-meta">{{ groupStore.groups.length }}</span>
      </router-link>
      <router-link
        to="/chat"
        class="menu-item"
        :class="{ active: $route.path === '/chat' }"
      >
        <span class="item-icon ghost" aria-hidden="true">
          <svg viewBox="0 0 24 24" fill="none">
            <path d="M7 17.8l-2.8 1.7.8-3.7A7.7 7.7 0 1 1 12 20a8 8 0 0 1-5-1.7Z" />
            <path d="M8.8 12.3h.1m2.8 0h.1m2.8 0h.1" />
          </svg>
        </span>
        <span>实时聊天</span>
      </router-link>
      <router-link
        to="/notifications"
        class="menu-item"
        :class="{ active: $route.path === '/notifications' }"
      >
        <span class="item-icon ghost" aria-hidden="true">
          <svg viewBox="0 0 24 24" fill="none">
            <path d="M12 5.5a5.5 5.5 0 0 1 5.5 5.5v3.3l1.3 2.2H5.2l1.3-2.2V11A5.5 5.5 0 0 1 12 5.5Z" />
            <path d="M9.8 18.3a2.2 2.2 0 0 0 4.4 0" />
          </svg>
        </span>
        <span class="menu-label">通知中心</span>
        <span class="menu-meta">{{ notificationStore.stats.pending_count }}</span>
      </router-link>
      <router-link
        v-if="authStore.isSuperAdmin"
        to="/audit"
        class="menu-item"
        :class="{ active: $route.path === '/audit' }"
      >
        <span class="item-icon ghost" aria-hidden="true">
          <svg viewBox="0 0 24 24" fill="none">
            <path d="M6.5 5.5h11v13h-11z" />
            <path d="M9 9.5h6M9 12.5h6M9 15.5h4" />
          </svg>
        </span>
        <span>审计日志</span>
      </router-link>
    </nav>

    <div ref="profileWrapRef" class="profile-wrap">
      <div class="user-card">
        <span class="avatar">{{ userInitial }}</span>
        <div class="user-meta">
          <p class="user-name">{{ authStore.userName || 'BoChat Admin' }}</p>
          <p class="user-role">{{ authStore.isSuperAdmin ? '超级管理员' : '管理员' }}</p>
        </div>
        <button type="button" class="arrow-btn" aria-haspopup="menu" aria-label="打开用户菜单">
          <svg viewBox="0 0 24 24" fill="none">
            <path d="m9 6 6 6-6 6" />
          </svg>
        </button>
      </div>

      <div class="actions-popover dashboard-surface" role="menu">
        <router-link to="/profile" class="popover-action">
          个人资料
        </router-link>
        <button class="popover-action danger" @click="handleLogout">
          退出登录
        </button>
      </div>
    </div>
  </aside>
</template>

<script setup lang="ts">
import { computed, onMounted } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import { useBotStore } from '@/stores/bots'
import { useGroupStore } from '@/stores/groups'
import { useNotificationStore } from '@/stores/notifications'

const router = useRouter()
const route = useRoute()
const authStore = useAuthStore()
const botStore = useBotStore()
const groupStore = useGroupStore()
const notificationStore = useNotificationStore()
const disabledStatuses = new Set(['disabled', 'stopped', 'paused', 'inactive'])

const userInitial = computed(() => {
  const source = authStore.userName || 'B'
  return source.trim().slice(0, 1).toUpperCase()
})

const inactiveBotsCount = computed(
  () => botStore.bots.filter((bot) => disabledStatuses.has(bot.status?.toLowerCase())).length
)
const activeBotsCount = computed(() => Math.max(0, botStore.bots.length - inactiveBotsCount.value))
const currentHomeTab = computed<'bots' | 'groups'>(() =>
  route.query.tab === 'groups' ? 'groups' : 'bots'
)

const isHomeTab = (tab: 'bots' | 'groups') => route.path === '/home' && currentHomeTab.value === tab

const handleLogout = async () => {
  if (confirm('确定要登出吗？')) {
    try {
      await authStore.handleLogout()
      await router.push('/login')
    } catch (error) {
      console.error('Logout failed:', error)
    }
  }
}

onMounted(() => {
  notificationStore.refreshStats().catch((error) => {
    console.error('Failed to fetch notification stats:', error)
  })
})
</script>

<style scoped>
.side-nav {
  position: relative;
  z-index: 5;
  width: 248px;
  flex: 0 0 248px;
  height: 100%;
  min-height: 0;
  border-radius: 10px;
  padding: 22px 18px;
  display: flex;
  flex-direction: column;
  gap: 20px;
  color: #1f1f1f;
  background: #f5f5f5;
  border: 1px solid #d0d0d0;
  box-shadow: 0 4px 10px rgba(0, 0, 0, 0.05);
}

.brand {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
  padding: 2px 8px 16px;
}

.brand h1 {
  margin: 0;
  font-size: 24px;
  line-height: 1;
  font-weight: 700;
  color: #1a1a1a;
}

.brand-status {
  display: inline-flex;
  align-items: center;
  margin-left: auto;
}

.status-dot {
  width: 10px;
  height: 10px;
  border-radius: 50%;
  background: #2f8f4e;
  box-shadow: 0 0 0 3px rgba(47, 143, 78, 0.2);
}

.main-menu {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.menu-item {
  --meta-bg: #e7e7e7;
  --meta-color: #6a6a6a;
  display: flex;
  align-items: center;
  gap: 10px;
  color: #2d2d2d;
  text-decoration: none;
  font-size: 14px;
  font-weight: 700;
  min-height: 46px;
  padding: 11px 10px;
  border-radius: 8px;
  border: 1px solid transparent;
  transition: var(--transition-base);
}

.menu-label {
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.menu-meta {
  margin-left: auto;
  font-size: 12px;
  font-weight: 600;
  color: var(--meta-color);
  padding: 2px 8px;
  border-radius: 999px;
  background: var(--meta-bg);
  line-height: 1.2;
}

.item-icon {
  width: 26px;
  height: 26px;
  display: grid;
  place-items: center;
  border-radius: 6px;
  color: #4a4a4a;
  background: transparent;
}

.item-icon svg {
  width: 18px;
  height: 18px;
  stroke: currentColor;
  stroke-width: 2;
  stroke-linecap: round;
  stroke-linejoin: round;
}

.item-icon.ghost {
  color: #4a4a4a;
  background: transparent;
}

.menu-item:hover {
  --meta-bg: #dddddd;
  border-color: #c8c8c8;
  background: #efefef;
}

.menu-item.active {
  --meta-bg: #5a5a5a;
  --meta-color: #f3f3f3;
  color: #f3f3f3;
  border-color: #2f2f2f;
  background: #2f2f2f;
  box-shadow: none;
}

.menu-item.active .item-icon {
  color: #f3f3f3;
}

.menu-item.active .menu-meta {
  color: var(--meta-color);
  background: var(--meta-bg);
}

.profile-wrap {
  position: relative;
  margin-top: auto;
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.user-card {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px 12px;
  border-radius: 8px;
  border: 1px solid #d0d0d0;
  background: #f5f5f5;
}

.avatar {
  width: 40px;
  height: 40px;
  border-radius: 50%;
  display: grid;
  place-items: center;
  background: #ebebeb;
  color: #2f2f2f;
  font-size: 20px;
  font-weight: 700;
}

.user-meta {
  min-width: 0;
  flex: 1;
}

.user-name {
  margin: 0;
  font-size: 17px;
  color: #1f1f1f;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.user-role {
  margin: 2px 0 0;
  font-size: 13px;
  color: #6c6c6c;
}

.arrow-btn {
  border: none;
  width: 36px;
  height: 36px;
  display: grid;
  place-items: center;
  border-radius: 10px;
  color: #2f2f2f;
  background: transparent;
}

.arrow-btn svg {
  width: 18px;
  height: 18px;
  stroke: currentColor;
  stroke-width: 2;
  stroke-linecap: round;
  stroke-linejoin: round;
}

.arrow-btn:hover {
  background: #ebebeb;
}

.actions-popover {
  position: absolute;
  left: 100%;
  top: 50%;
  transform: translateY(-50%);
  width: 196px;
  padding: 8px;
  border-radius: 8px;
  border: 1px solid #d0d0d0;
  background: #f5f5f5;
  box-shadow: 0 8px 16px rgba(0, 0, 0, 0.08);
  backdrop-filter: none;
  opacity: 0;
  visibility: hidden;
  pointer-events: none;
  transition: var(--transition-base);
}

.profile-wrap:hover .actions-popover,
.profile-wrap:focus-within .actions-popover {
  opacity: 1;
  visibility: visible;
  pointer-events: auto;
}

.popover-action {
  width: 100%;
  border: none;
  background: transparent;
  color: #2f2f2f;
  text-decoration: none;
  border-radius: 12px;
  padding: 10px 12px;
  text-align: left;
  font-size: 13px;
  font-weight: 700;
  display: block;
  transition: var(--transition-base);
}

.popover-action:hover {
  background: #ebebeb;
  color: #1f1f1f;
}

.popover-action.danger {
  color: #7a2e2a;
}

.popover-action.danger:hover {
  background: #efefef;
}

@media (max-width: 1200px) {
  .side-nav {
    width: 208px;
    flex-basis: 208px;
  }

  .brand h1 {
    font-size: 22px;
  }

  .menu-item {
    font-size: 15px;
  }

  .user-name {
    font-size: 16px;
  }

  .user-role {
    font-size: 13px;
  }

}

@media (max-width: 768px) {
  .side-nav {
    width: 100%;
    flex-basis: auto;
    padding: 14px;
    gap: 12px;
    border-radius: 8px;
    height: auto;
  }

  .main-menu {
    flex-direction: row;
  }

  .menu-item {
    flex: 1;
    justify-content: center;
    font-size: 13px;
    padding: 10px;
  }

  .menu-meta {
    display: none;
  }

  .actions-popover {
    left: 0;
    top: calc(100% + 8px);
    right: 0;
    transform: none;
    width: auto;
  }
}

.popover-action,
.arrow-btn,
.menu-item {
  transition: var(--transition-base);
}
</style>
