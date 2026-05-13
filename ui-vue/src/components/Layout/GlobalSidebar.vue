<template>
  <aside class="global-sidebar">
    <div class="sidebar-brand">
      <router-link to="/" class="brand-link">
        <h1>BoChat</h1>
      </router-link>
    </div>

    <nav class="sidebar-nav">
      <router-link
        v-for="item in navItems"
        :key="item.path"
        :to="item.path"
        class="nav-item"
        :class="{ active: isActive(item) }"
      >
        <span class="nav-icon">
          <component :is="item.icon" class="icon-svg" />
        </span>
        <span class="nav-label">{{ item.label }}</span>
      </router-link>
    </nav>

    <div class="sidebar-user">
      <button
        type="button"
        class="user-btn"
        :class="{ open: showUserMenu }"
        @click="showUserMenu = !showUserMenu"
        @blur="closeUserMenu"
      >
        <span class="user-avatar">{{ authStore.userName?.charAt(0) ?? '?' }}</span>
        <div class="user-info">
          <span class="user-name">{{ authStore.userName ?? '用户' }}</span>
          <span class="user-role">{{ authStore.isSuperAdmin ? '超级管理员' : '管理员' }}</span>
        </div>
        <ChevronUp class="user-arrow" :class="{ rotated: showUserMenu }" />
      </button>

      <div v-if="showUserMenu" class="user-menu">
        <router-link to="/profile" class="user-menu-item" @click="showUserMenu = false">
          <User class="icon-xs" />
          个人资料
        </router-link>
        <button type="button" class="user-menu-item logout" @click="handleLogout">
          <LogOut class="icon-xs" />
          退出登录
        </button>
      </div>
    </div>
  </aside>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import {
  LayoutDashboard,
  Bot,
  Users,
  MessageSquare,
  ScrollText,
  ChevronUp,
  User,
  LogOut,
} from 'lucide-vue-next'
import { useAuthStore } from '@/stores/auth'

const route = useRoute()
const router = useRouter()
const authStore = useAuthStore()

const showUserMenu = ref(false)

const navItems = computed(() => {
  const items = [
    { path: '/', label: '工作台', icon: LayoutDashboard },
    { path: '/bots', label: 'Bot 管理', icon: Bot },
    { path: '/groups', label: '群组管理', icon: Users },
    { path: '/chat', label: '实时会话', icon: MessageSquare },
  ]
  if (authStore.isSuperAdmin) {
    items.push({ path: '/audit', label: '审计日志', icon: ScrollText })
  }
  return items
})

const isActive = (item: { path: string }) => {
  if (item.path === '/') {
    return route.path === '/'
  }
  return route.path.startsWith(item.path)
}

const handleLogout = () => {
  showUserMenu.value = false
  authStore.handleLogout()
  router.push('/login')
}

const closeUserMenu = () => {
  setTimeout(() => {
    showUserMenu.value = false
  }, 200)
}

</script>

<style scoped>
.global-sidebar {
  width: 240px;
  flex: 0 0 240px;
  height: 100vh;
  display: flex;
  flex-direction: column;
  background: #f5f5f5;
  border-right: 1px solid #d0d0d0;
  overflow-y: auto;
  z-index: 10;
}

.sidebar-brand {
  padding: 20px 20px 12px;
  flex-shrink: 0;
}

.brand-link {
  text-decoration: none;
}

.brand-link h1 {
  margin: 0;
  font-size: 22px;
  font-weight: 700;
  color: #1a1a1a;
  letter-spacing: -0.3px;
}

.sidebar-nav {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 4px;
  padding: 8px 12px;
  overflow-y: auto;
}

.nav-item {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 10px 12px;
  border-radius: 8px;
  text-decoration: none;
  color: #4a4a4a;
  font-size: 14px;
  font-weight: 600;
  transition: all 0.15s ease;
  position: relative;
}

.nav-item:hover {
  background: #ebebeb;
  color: #1f1f1f;
}

.nav-item.active {
  background: #2f2f2f;
  color: #f3f3f3;
}

.nav-icon {
  width: 22px;
  height: 22px;
  display: grid;
  place-items: center;
  flex-shrink: 0;
}

.icon-svg {
  width: 18px;
  height: 18px;
  stroke: currentColor;
  stroke-width: 2;
  stroke-linecap: round;
  stroke-linejoin: round;
  fill: none;
}

.nav-label {
  flex: 1;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.sidebar-user {
  flex-shrink: 0;
  padding: 12px;
  border-top: 1px solid #e0e0e0;
  position: relative;
}

.user-btn {
  width: 100%;
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 8px;
  border: 1px solid transparent;
  border-radius: 8px;
  background: transparent;
  cursor: pointer;
  transition: all 0.15s ease;
}

.user-btn:hover,
.user-btn.open {
  background: #ebebeb;
  border-color: #d0d0d0;
}

.user-avatar {
  width: 36px;
  height: 36px;
  border-radius: 50%;
  background: #2f2f2f;
  color: #f3f3f3;
  display: grid;
  place-items: center;
  font-size: 15px;
  font-weight: 700;
  flex-shrink: 0;
}

.user-info {
  flex: 1;
  min-width: 0;
  text-align: left;
}

.user-name {
  display: block;
  font-size: 13px;
  font-weight: 700;
  color: #1f1f1f;
}

.user-role {
  display: block;
  font-size: 11px;
  color: #6f6f6f;
  margin-top: 1px;
}

.user-arrow {
  width: 14px;
  height: 14px;
  stroke: #6f6f6f;
  stroke-width: 2;
  flex-shrink: 0;
  transition: transform 0.2s ease;
}

.user-arrow.rotated {
  transform: rotate(180deg);
}

.user-menu {
  position: absolute;
  bottom: calc(100% + 4px);
  left: 12px;
  right: 12px;
  background: #fff;
  border: 1px solid #d0d0d0;
  border-radius: 8px;
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.1);
  overflow: hidden;
  z-index: 20;
}

.user-menu-item {
  display: flex;
  align-items: center;
  gap: 8px;
  width: 100%;
  padding: 10px 14px;
  border: none;
  background: transparent;
  color: #1f1f1f;
  font-size: 13px;
  font-weight: 600;
  text-decoration: none;
  cursor: pointer;
  transition: background 0.15s ease;
}

.user-menu-item:hover {
  background: #f0f0f0;
}

.user-menu-item.logout {
  color: #c4453c;
  border-top: 1px solid #f0f0f0;
}

.icon-xs {
  width: 14px;
  height: 14px;
  stroke: currentColor;
  stroke-width: 2;
}

@media (max-width: 768px) {
  .global-sidebar {
    width: 100%;
    height: auto;
    flex: 0 0 auto;
    flex-direction: row;
    align-items: center;
    padding: 8px 12px;
    gap: 4px;
    border-right: none;
    border-bottom: 1px solid #d0d0d0;
    overflow-x: auto;
    overflow-y: hidden;
  }

  .sidebar-brand {
    display: none;
  }

  .sidebar-nav {
    flex-direction: row;
    gap: 2px;
    padding: 0;
    overflow-y: visible;
    flex: 1;
  }

  .nav-item {
    padding: 8px 10px;
    font-size: 12px;
    gap: 6px;
  }

  .nav-label {
    display: none;
  }

  .sidebar-user {
    padding: 0;
    border-top: none;
  }

  .user-info,
  .user-arrow {
    display: none;
  }

  .user-btn {
    padding: 6px;
    width: auto;
  }

  .user-menu {
    left: auto;
    right: 0;
    width: 160px;
  }
}
</style>
