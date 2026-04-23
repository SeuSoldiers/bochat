<template>
  <nav class="top-nav">
    <div class="nav-container dashboard-surface">
      <div class="nav-logo">
        <span class="logo-mark">✶</span>
        <h1>BoChat Console</h1>
      </div>

      <div class="nav-tabs">
        <router-link
          to="/home"
          class="nav-tab"
          :class="{ active: $route.path === '/home' }"
        >
          Overview
        </router-link>
        <router-link
          to="/chat"
          class="nav-tab"
          :class="{ active: $route.path === '/chat' }"
        >
          Live Chat
        </router-link>
        <router-link
          to="/profile"
          class="nav-tab"
          :class="{ active: $route.path === '/profile' }"
        >
          Profile
        </router-link>
      </div>

      <div class="nav-user">
        <div class="user-info">
          <span class="user-tag">Sales Admin</span>
          <span class="user-name">{{ authStore.userName || '未命名用户' }}</span>
          <button class="logout-btn" @click="handleLogout">
            Sign out
          </button>
        </div>
      </div>
    </div>
  </nav>
</template>

<script setup lang="ts">
import { useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'

const router = useRouter()
const authStore = useAuthStore()

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
</script>

<style scoped>
.top-nav {
  position: relative;
  z-index: 4;
  margin-bottom: 14px;
}

.nav-container {
  width: 100%;
  padding: 12px 18px;
  display: flex;
  justify-content: space-between;
  align-items: center;
  min-height: 76px;
  gap: 18px;
}

.nav-logo {
  display: flex;
  align-items: center;
  gap: 12px;
}

.logo-mark {
  width: 34px;
  height: 34px;
  border-radius: 50%;
  display: grid;
  place-items: center;
  color: #173329;
  background: #a6d72e;
  box-shadow: 0 8px 16px rgba(14, 60, 47, 0.24);
}

.nav-logo h1 {
  font-size: 19px;
  font-weight: 700;
  color: #0f2f25;
  margin: 0;
}

.nav-tabs {
  display: flex;
  gap: 10px;
  flex: 1;
  margin-left: 6px;
  min-width: 0;
  flex-wrap: wrap;
}

.nav-tab {
  color: #5d6a65;
  text-decoration: none;
  font-size: 13px;
  font-weight: 700;
  padding: 8px 14px;
  border: 1px solid transparent;
  border-radius: 999px;
  transition: var(--transition-base);
}

.nav-tab:hover {
  color: #102d23;
  border-color: #d4ddd4;
  background: #f7faf6;
}

.nav-tab.active {
  color: #102d23;
  border-color: #9bcf2d;
  background: #eaf6cd;
}

.nav-user {
  display: flex;
  align-items: center;
  gap: 20px;
}

.user-info {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 9px 8px 12px;
  border-radius: 999px;
  border: 1px solid #d8dfd8;
  background: #fff;
}

.user-tag {
  padding: 4px 8px;
  border-radius: 999px;
  font-size: 11px;
  font-weight: 700;
  color: #dff1e8;
  background: #0e3c2f;
}

.user-name {
  font-size: 12px;
  color: #51615b;
  max-width: 160px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.logout-btn {
  padding: 7px 12px;
  background-color: #eaf6cd;
  color: #173329;
  border: 1px solid #bcde66;
  border-radius: 999px;
  font-size: 12px;
  font-weight: 700;
  transition: var(--transition-base);
}

.logout-btn:hover {
  background-color: #dff0b2;
  transform: translateY(-1px);
}

@media (max-width: 768px) {
  .top-nav {
    margin-bottom: 10px;
  }

  .nav-container {
    padding: 14px;
    align-items: flex-start;
    flex-direction: column;
  }

  .nav-tabs {
    width: 100%;
    margin-left: 0;
  }

  .nav-user {
    width: 100%;
  }

  .user-info {
    width: 100%;
    justify-content: space-between;
  }
}
</style>
