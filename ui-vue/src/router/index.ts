/**
 * Vue Router 配置
 */

import { createRouter, createWebHistory, type RouteRecordRaw } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import { useBotUserAuthStore } from '@/stores/botUserAuth'

const LoginView = () => import('@/views/LoginView.vue')
const BotLoginView = () => import('@/views/BotLoginView.vue')
const BotChatView = () => import('@/views/BotChatView.vue')
const DashboardView = () => import('@/views/DashboardView.vue')
const BotsView = () => import('@/views/BotsView.vue')
const GroupsView = () => import('@/views/GroupsView.vue')
const ChatView = () => import('@/views/ChatView.vue')
const ProfileView = () => import('@/views/ProfileView.vue')
const AuditView = () => import('@/views/AuditView.vue')
const NotificationsView = () => import('@/views/NotificationsView.vue')
const AppShell = () => import('@/components/Layout/AppShell.vue')

export const routes: RouteRecordRaw[] = [
  {
    path: '/login',
    name: 'login',
    component: LoginView,
    meta: {
      requiresAuth: false,
    },
  },
  {
    path: '/bot/login',
    name: 'bot-login',
    component: BotLoginView,
    meta: {
      requiresAuth: false,
    },
  },
  {
    path: '/bot/chat',
    name: 'bot-chat',
    component: BotChatView,
    meta: {
      requiresAuth: false,
      requiresBotAuth: true,
    },
  },
  {
    path: '/bot',
    redirect: '/bot/chat',
  },
  {
    path: '/',
    component: AppShell,
    meta: {
      requiresAuth: true,
    },
    children: [
      {
        path: '',
        name: 'dashboard',
        component: DashboardView,
      },
      {
        path: 'bots',
        name: 'bots',
        component: BotsView,
      },
      {
        path: 'groups',
        name: 'groups',
        component: GroupsView,
      },
      {
        path: 'chat',
        name: 'chat',
        component: ChatView,
      },
      {
        path: 'profile',
        name: 'profile',
        component: ProfileView,
      },
      {
        path: 'notifications',
        name: 'notifications',
        component: NotificationsView,
      },
      {
        path: 'audit',
        name: 'audit',
        component: AuditView,
        meta: {
          requiresSuperAdmin: true,
        },
      },
    ],
  },
  {
    path: '/home',
    redirect: '/',
  },
  {
    path: '/:pathMatch(.*)*',
    redirect: '/',
  },
]

const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes,
})

router.beforeEach((to, _from, next) => {
  const authStore = useAuthStore()
  const botAuthStore = useBotUserAuthStore()
  const requiresAuth = to.meta.requiresAuth !== false
  const requiresBotAuth =
    to.meta.requiresBotAuth === true ||
    to.matched.some((record) => record.meta.requiresBotAuth === true)

  // Check parent route meta for requiresSuperAdmin (nested routes)
  const requiresSuperAdmin =
    to.meta.requiresSuperAdmin === true ||
    to.matched.some((record) => record.meta.requiresSuperAdmin === true)

  if (!authStore.user || !authStore.token) {
    authStore.initializeAuth()
  }

  if (!botAuthStore.bot || !botAuthStore.token) {
    botAuthStore.initializeAuth()
  }

  const isAuthenticated = authStore.isAuthenticated
  const isBotAuthenticated = botAuthStore.isAuthenticated

  if (requiresBotAuth && !isBotAuthenticated) {
    next({
      path: '/bot/login',
      query: to.fullPath === '/bot/login' ? undefined : { redirect: to.fullPath },
    })
  } else if (to.path === '/bot/login' && isBotAuthenticated) {
    next('/bot/chat')
  } else if (requiresAuth && !isAuthenticated) {
    next({
      path: '/login',
      query: to.fullPath === '/login' ? undefined : { redirect: to.fullPath },
    })
  } else if (requiresSuperAdmin && !authStore.isSuperAdmin) {
    next('/')
  } else if (to.path === '/login' && isAuthenticated) {
    next('/')
  } else {
    next()
  }
})

export default router
