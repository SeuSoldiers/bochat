<template>
  <aside class="side-nav">
    <div class="brand">
      <span class="brand-mark">✶</span>
      <h1>BoChat</h1>
    </div>

    <nav class="main-menu">
      <router-link
        to="/home"
        class="menu-item"
        :class="{ active: $route.path === '/home' }"
      >
        <span class="item-icon" aria-hidden="true">
          <svg viewBox="0 0 24 24" fill="none">
            <rect x="4.5" y="5.5" width="15" height="14" rx="3" />
            <path d="M9 10.5h6M12 8v5" />
          </svg>
        </span>
        <span>总览</span>
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
    </nav>

    <div class="side-deco" aria-hidden="true"></div>

    <div class="status-card">
      <p class="status-label">系统状态</p>
      <p class="status-title">运行正常</p>
      <p class="status-desc">所有系统运行良好</p>
      <div class="status-mark" aria-hidden="true">
        <svg viewBox="0 0 24 24" fill="none">
          <path d="M12 2 20.5 5.1v6.1c0 5.3-3.8 9.6-8.5 10.8C7.3 20.8 3.5 16.5 3.5 11.2V5.1L12 2Z" />
          <path d="m8.3 12.2 2.3 2.3 5-5" />
        </svg>
      </div>
    </div>

    <div ref="profileWrapRef" class="profile-wrap">
      <div class="user-card">
        <span class="avatar">{{ userInitial }}</span>
        <div class="user-meta">
          <p class="user-name">{{ authStore.userName || 'BoChat Admin' }}</p>
          <p class="user-role">管理员</p>
        </div>
        <button
          type="button"
          class="arrow-btn"
          :aria-expanded="showProfileActions ? 'true' : 'false'"
          @click="showProfileActions = !showProfileActions"
        >
          <svg viewBox="0 0 24 24" fill="none" :class="{ open: showProfileActions }">
            <path d="m7 14 5-5 5 5" />
          </svg>
        </button>
      </div>

      <div v-if="showProfileActions" class="actions-popover dashboard-surface">
        <router-link to="/profile" class="popover-action" @click="showProfileActions = false">
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
import { computed, onBeforeUnmount, onMounted, ref } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'

const router = useRouter()
const authStore = useAuthStore()
const showProfileActions = ref(false)
const profileWrapRef = ref<HTMLElement | null>(null)

const userInitial = computed(() => {
  const source = authStore.userName || 'B'
  return source.trim().slice(0, 1).toUpperCase()
})

const onGlobalPointerDown = (event: MouseEvent) => {
  if (!showProfileActions.value || !profileWrapRef.value) {
    return
  }
  if (!profileWrapRef.value.contains(event.target as Node)) {
    showProfileActions.value = false
  }
}

const handleLogout = async () => {
  if (confirm('确定要登出吗？')) {
    try {
      showProfileActions.value = false
      await authStore.handleLogout()
      await router.push('/login')
    } catch (error) {
      console.error('Logout failed:', error)
    }
  }
}

onMounted(() => {
  document.addEventListener('mousedown', onGlobalPointerDown)
})

onBeforeUnmount(() => {
  document.removeEventListener('mousedown', onGlobalPointerDown)
})
</script>

<style scoped>
.side-nav {
  position: relative;
  z-index: 5;
  width: 306px;
  flex: 0 0 306px;
  height: 100%;
  min-height: 0;
  border-radius: 0;
  padding: 22px 18px;
  display: flex;
  flex-direction: column;
  gap: 20px;
  color: #ebf4ef;
  background:
    radial-gradient(circle at 20% 0%, rgba(36, 103, 79, 0.35) 0, rgba(36, 103, 79, 0) 42%),
    radial-gradient(circle at 90% 96%, rgba(166, 215, 46, 0.16) 0, rgba(166, 215, 46, 0) 40%),
    linear-gradient(180deg, #023c2e 0%, #032f24 46%, #022319 100%);
  box-shadow: inset 0 0 0 1px rgba(255, 255, 255, 0.08), 0 20px 36px rgba(5, 22, 16, 0.3);
}

.brand {
  display: flex;
  align-items: center;
  gap: 11px;
  padding: 2px 8px 16px;
}

.brand-mark {
  width: 38px;
  height: 38px;
  border-radius: 50%;
  display: grid;
  place-items: center;
  color: #173329;
  background: #a6d72e;
  font-size: 19px;
  box-shadow: 0 10px 20px rgba(8, 33, 25, 0.32);
}

.brand h1 {
  margin: 0;
  font-size: 24px;
  line-height: 1;
  font-weight: 700;
  color: #f2f8f4;
}

.main-menu {
  display: flex;
  flex-direction: column;
  gap: 14px;
}

.menu-item {
  display: flex;
  align-items: center;
  gap: 12px;
  color: #d7e9e0;
  text-decoration: none;
  font-size: 18px;
  font-weight: 700;
  padding: 14px 14px;
  border-radius: 14px;
  border: 1px solid transparent;
  transition: var(--transition-base);
}

.item-icon {
  width: 38px;
  height: 38px;
  display: grid;
  place-items: center;
  border-radius: 10px;
  color: #0f2e24;
  background: rgba(166, 215, 46, 0.9);
}

.item-icon svg {
  width: 20px;
  height: 20px;
  stroke: currentColor;
  stroke-width: 2;
  stroke-linecap: round;
  stroke-linejoin: round;
}

.item-icon.ghost {
  color: #e4eee8;
  background: transparent;
}

.menu-item:hover {
  border-color: rgba(255, 255, 255, 0.2);
  background: rgba(255, 255, 255, 0.08);
}

.menu-item.active {
  color: #f2fbf5;
  border-color: rgba(197, 228, 127, 0.32);
  background: rgba(9, 62, 47, 0.75);
  box-shadow: inset 0 0 0 1px rgba(255, 255, 255, 0.05);
}

.side-deco {
  width: 148px;
  height: 86px;
  margin: 20px 0 6px;
  background: linear-gradient(150deg, rgba(16, 79, 60, 0.48) 0%, rgba(7, 45, 34, 0.42) 100%);
  clip-path: polygon(0 0, 100% 0, 100% 72%, 78% 100%, 0 100%);
  border-radius: 4px;
}

.status-card {
  margin-top: 8px;
  position: relative;
  border-radius: 18px;
  padding: 18px 16px;
  border: 1px solid rgba(255, 255, 255, 0.14);
  background: linear-gradient(135deg, rgba(12, 79, 59, 0.65) 0%, rgba(17, 62, 48, 0.45) 100%);
}

.status-label {
  margin: 0;
  font-size: 20px;
  color: #a9e1be;
  display: flex;
  align-items: center;
  gap: 8px;
}

.status-label::before {
  content: '';
  width: 12px;
  height: 12px;
  border-radius: 50%;
  background: #21c372;
  box-shadow: 0 0 0 6px rgba(33, 195, 114, 0.14);
}

.status-title {
  margin: 4px 0 2px;
  font-size: 34px;
  font-weight: 700;
}

.status-desc {
  margin: 0;
  font-size: 14px;
  color: rgba(230, 244, 235, 0.85);
}

.status-mark {
  position: absolute;
  right: 16px;
  top: 50%;
  transform: translateY(-50%);
  width: 106px;
  height: 106px;
  display: grid;
  place-items: center;
  color: rgba(207, 229, 215, 0.82);
  background: rgba(120, 164, 143, 0.2);
  border-radius: 20px;
}

.status-mark svg {
  width: 78px;
  height: 78px;
  stroke: currentColor;
  stroke-width: 1.6;
  stroke-linecap: round;
  stroke-linejoin: round;
}

.profile-wrap {
  position: relative;
  margin-top: auto;
}

.user-card {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px 12px;
  border-radius: 16px;
  border: 1px solid rgba(255, 255, 255, 0.14);
  background: rgba(4, 34, 26, 0.48);
}

.avatar {
  width: 50px;
  height: 50px;
  border-radius: 50%;
  display: grid;
  place-items: center;
  background: #a6d72e;
  color: #173329;
  font-size: 26px;
  font-weight: 700;
}

.user-meta {
  min-width: 0;
  flex: 1;
}

.user-name {
  margin: 0;
  font-size: 17px;
  color: #f2f8f4;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.user-role {
  margin: 2px 0 0;
  font-size: 13px;
  color: #b6c9bf;
}

.arrow-btn {
  border: none;
  width: 36px;
  height: 36px;
  display: grid;
  place-items: center;
  border-radius: 10px;
  color: #eef6f1;
  background: transparent;
}

.arrow-btn svg {
  width: 18px;
  height: 18px;
  stroke: currentColor;
  stroke-width: 2;
  stroke-linecap: round;
  stroke-linejoin: round;
  transition: transform 0.2s ease;
}

.arrow-btn svg.open {
  transform: rotate(180deg);
}

.arrow-btn:hover {
  background: rgba(255, 255, 255, 0.08);
}

.actions-popover {
  position: absolute;
  left: 12px;
  bottom: calc(100% + 10px);
  width: 196px;
  padding: 8px;
  border-radius: 18px;
  border: 1px solid rgba(255, 255, 255, 0.14);
  background:
    radial-gradient(circle at 0 100%, rgba(166, 215, 46, 0.18) 0, rgba(166, 215, 46, 0) 65%),
    linear-gradient(150deg, rgba(3, 55, 42, 0.94) 0%, rgba(3, 46, 35, 0.96) 100%);
  box-shadow: 0 18px 30px rgba(6, 21, 16, 0.34);
  backdrop-filter: blur(10px);
}

.popover-action {
  width: 100%;
  border: none;
  background: transparent;
  color: #dcede5;
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
  background: rgba(255, 255, 255, 0.1);
  color: #f4fbf7;
}

.popover-action.danger {
  color: #f3d6d4;
}

.popover-action.danger:hover {
  background: rgba(226, 92, 76, 0.16);
}

@media (max-width: 1200px) {
  .side-nav {
    width: 240px;
    flex-basis: 240px;
  }

  .brand h1 {
    font-size: 22px;
  }

  .menu-item {
    font-size: 19px;
  }

  .status-title {
    font-size: 18px;
  }

  .status-label {
    font-size: 14px;
  }

  .status-desc {
    font-size: 13px;
  }

  .user-name {
    font-size: 16px;
  }

  .user-role {
    font-size: 13px;
  }

  .status-mark {
    width: 68px;
    height: 68px;
    font-size: 22px;
  }

  .side-deco {
    width: 118px;
    height: 64px;
  }
}

@media (max-width: 768px) {
  .side-nav {
    width: 100%;
    flex-basis: auto;
    padding: 14px;
    gap: 12px;
    border-radius: 18px;
    height: auto;
  }

  .main-menu {
    flex-direction: row;
  }

  .menu-item {
    flex: 1;
    justify-content: center;
    font-size: 15px;
    padding: 10px;
  }

  .status-card {
    margin-top: 0;
  }

  .actions-popover {
    left: 0;
    right: 0;
    width: auto;
  }
}

.popover-action,
.arrow-btn,
.menu-item {
  transition: var(--transition-base);
}
</style>
