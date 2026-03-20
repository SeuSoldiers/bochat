<template>
  <nav class="top-nav">
    <div class="nav-container">
      <!-- Logo -->
      <div class="nav-logo">
        <h1>BoChat</h1>
      </div>

      <!-- 导航标签 -->
      <div class="nav-tabs">
        <router-link
          to="/home"
          class="nav-tab"
          :class="{ active: $route.path === '/home' }"
        >
          主页
        </router-link>
        <router-link
          to="/chat"
          class="nav-tab"
          :class="{ active: $route.path === '/chat' }"
        >
          聊天
        </router-link>
      </div>

      <!-- 用户菜单 -->
      <div class="nav-user">
        <div class="user-info">
          <span class="user-phone">{{ authStore.userName || authStore.userPhone }}</span>
          <button class="logout-btn" @click="handleLogout">
            登出
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
  background-color: white;
  border-bottom: 1px solid #d4cfc8;
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.05);
  position: sticky;
  top: 0;
  z-index: 100;
}

.nav-container {
  max-width: 1400px;
  margin: 0 auto;
  padding: 0 30px;
  display: flex;
  justify-content: space-between;
  align-items: center;
  height: 64px;
}

.nav-logo h1 {
  font-size: 20px;
  font-weight: 700;
  color: #8b9d83;
  letter-spacing: 1px;
  margin: 0;
}

.nav-tabs {
  display: flex;
  gap: 30px;
  flex: 1;
  margin-left: 40px;
}

.nav-tab {
  color: #888888;
  text-decoration: none;
  font-size: 14px;
  font-weight: 500;
  padding-bottom: 8px;
  border-bottom: 2px solid transparent;
  transition: all 0.3s ease;
  position: relative;
}

.nav-tab:hover {
  color: #8b9d83;
}

.nav-tab.active {
  color: #8b9d83;
  border-bottom-color: #8b9d83;
}

.nav-user {
  display: flex;
  align-items: center;
  gap: 20px;
}

.user-info {
  display: flex;
  align-items: center;
  gap: 12px;
}

.user-phone {
  font-size: 13px;
  color: #888888;
}

.logout-btn {
  padding: 6px 12px;
  background-color: #f5e6e6;
  color: #a88b7f;
  border: none;
  border-radius: 4px;
  font-size: 12px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.3s ease;
}

.logout-btn:hover {
  background-color: #ead4d0;
  color: #8b6b5f;
}

@media (max-width: 768px) {
  .nav-container {
    padding: 0 15px;
  }

  .nav-logo h1 {
    font-size: 18px;
  }

  .nav-tabs {
    gap: 15px;
    margin-left: 20px;
  }

  .nav-tab {
    font-size: 13px;
  }

  .user-phone {
    display: none;
  }
}
</style>
