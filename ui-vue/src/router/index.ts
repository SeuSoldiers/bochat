/**
 * Vue Router 配置
 */

import { createRouter, createWebHistory, type RouteRecordRaw } from 'vue-router'
import { useAuthStore } from '@/stores/auth'

// 视图组件（延后加载）
const LoginView = () => import('@/views/LoginView.vue')
const HomeView = () => import('@/views/HomeView.vue')
const ChatView = () => import('@/views/ChatView.vue')
const ProfileView = () => import('@/views/ProfileView.vue')
const AuditView = () => import('@/views/AuditView.vue')
const NotificationsView = () => import('@/views/NotificationsView.vue')

export const routes: RouteRecordRaw[] = [
  {
    path: '/',
    redirect: '/home',
  },
  {
    path: '/login',
    name: 'login',
    component: LoginView,
    meta: {
      requiresAuth: false,
    },
  },
  {
    path: '/home',
    name: 'home',
    component: HomeView,
    meta: {
      requiresAuth: true,
    },
  },
  {
    path: '/chat',
    name: 'chat',
    component: ChatView,
    meta: {
      requiresAuth: true,
    },
  },
  {
    path: '/profile',
    name: 'profile',
    component: ProfileView,
    meta: {
      requiresAuth: true,
    },
  },
  {
    path: '/notifications',
    name: 'notifications',
    component: NotificationsView,
    meta: {
      requiresAuth: true,
    },
  },
  {
    path: '/audit',
    name: 'audit',
    component: AuditView,
    meta: {
      requiresAuth: true,
      requiresSuperAdmin: true,
    },
  },
  {
    path: '/:pathMatch(.*)*',
    redirect: '/home',
  },
]

const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes,
})

/**
 * 路由守卫：检查认证状态
 */
router.beforeEach((to, _from, next) => {
  const authStore = useAuthStore()
  const requiresAuth = to.meta.requiresAuth !== false
  const requiresSuperAdmin = to.meta.requiresSuperAdmin === true

  // 如果 store 中没有用户信息，从 localStorage 恢复
  if (!authStore.user || !authStore.token) {
    authStore.initializeAuth()
  }

  const isAuthenticated = authStore.isAuthenticated

  if (requiresAuth && !isAuthenticated) {
    // 需要认证但未认证，跳转到登录
    next('/login')
  } else if (requiresSuperAdmin && !authStore.isSuperAdmin) {
    next('/home')
  } else if (to.path === '/login' && isAuthenticated) {
    // 已认证但访问登录页，跳转到首页
    next('/home')
  } else {
    // 允许跳转
    next()
  }
})

export default router
