/**
 * 认证状态管理（Pinia）
 */

import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import {
  getCurrentUser,
  login,
  logout,
  register,
  updateCurrentUser,
  type AuthIdentityPayload,
  type LoginResponse,
  type RegisterPayload,
} from '@/services/auth'
import { STORAGE_KEYS } from '@/constants/storageKeys'
import type { User } from '@/types'
import { getErrorMessage } from '@/utils/error'

export const useAuthStore = defineStore('auth', () => {
  // 状态
  const user = ref<User | null>(null)
  const token = ref<string | null>(null)
  const loading = ref(false)
  const error = ref<string | null>(null)

  // 初始化：从 localStorage 恢复状态
  const initializeAuth = () => {
    const storedToken = localStorage.getItem(STORAGE_KEYS.TOKEN)
    const storedUser = localStorage.getItem(STORAGE_KEYS.USER)

    if (storedToken) {
      token.value = storedToken
    }

    if (storedUser) {
      try {
        user.value = JSON.parse(storedUser)
      } catch (e) {
        console.error('Failed to parse stored user:', e)
      }
    }
  }

  // 计算属性
  const isAuthenticated = computed(() => !!token.value && !!user.value)
  const userId = computed(() => parseUserIdFromToken(token.value) || '')
  const userPhone = computed(() => user.value?.phone || '')
  const userName = computed(() => user.value?.name || '')

  // 方法：注册
  const handleRegister = async (payload: RegisterPayload) => {
    loading.value = true
    error.value = null

    try {
      const response = await register(payload)
      user.value = extractUser(response)
      token.value = response.token

      localStorage.setItem(STORAGE_KEYS.TOKEN, response.token)
      localStorage.setItem(STORAGE_KEYS.USER, JSON.stringify(user.value))

      return response
    } catch (err: any) {
      error.value = getErrorMessage(err, '注册失败')
      throw err
    } finally {
      loading.value = false
    }
  }

  // 方法：登录
  const handleLogin = async (payload: AuthIdentityPayload) => {
    loading.value = true
    error.value = null

    try {
      const response = await login(payload)
      user.value = extractUser(response)
      token.value = response.token

      localStorage.setItem(STORAGE_KEYS.TOKEN, response.token)
      localStorage.setItem(STORAGE_KEYS.USER, JSON.stringify(user.value))

      return response
    } catch (err: any) {
      error.value = getErrorMessage(err, '登录失败')
      throw err
    } finally {
      loading.value = false
    }
  }

  const refreshCurrentUser = async () => {
    if (!token.value) {
      return null
    }

    try {
      const profile = await getCurrentUser()
      user.value = {
        name: profile.name,
        phone: profile.phone,
        avatar_url: profile.avatar_url,
        created_at: profile.created_at,
        updated_at: profile.updated_at,
      }
      localStorage.setItem(STORAGE_KEYS.USER, JSON.stringify(user.value))
      return user.value
    } catch (err: any) {
      error.value = getErrorMessage(err, '获取用户信息失败')
      throw err
    }
  }

  const updateProfile = async (payload: {
    name?: string
    phone?: string
    avatar_url?: string
  }) => {
    loading.value = true
    error.value = null

    try {
      const updated = await updateCurrentUser(payload)
      user.value = {
        name: updated.name,
        phone: updated.phone,
        avatar_url: updated.avatar_url,
        created_at: updated.created_at,
        updated_at: updated.updated_at,
      }
      localStorage.setItem(STORAGE_KEYS.USER, JSON.stringify(user.value))
      return user.value
    } catch (err: any) {
      error.value = getErrorMessage(err, '更新用户信息失败')
      throw err
    } finally {
      loading.value = false
    }
  }

  // 方法：登出
  const handleLogout = async () => {
    loading.value = true
    error.value = null

    try {
      await logout()
      user.value = null
      token.value = null

      // 清理 localStorage
      localStorage.removeItem(STORAGE_KEYS.TOKEN)
      localStorage.removeItem(STORAGE_KEYS.USER)
    } catch (err: any) {
      error.value = getErrorMessage(err, '登出失败')
      throw err
    } finally {
      loading.value = false
    }
  }

  // 方法：清除错误
  const clearError = () => {
    error.value = null
  }

  return {
    // 状态
    user,
    token,
    loading,
    error,

    // 计算属性
    isAuthenticated,
    userId,
    userPhone,
    userName,

    // 方法
    initializeAuth,
    handleRegister,
    handleLogin,
    refreshCurrentUser,
    updateProfile,
    handleLogout,
    clearError,
  }
})

function extractUser(response: LoginResponse): User {
  return {
    name: response.name,
    phone: response.phone,
  }
}

function parseUserIdFromToken(token: string | null): string | null {
  if (!token) {
    return null
  }

  const parts = token.split(':')
  if (parts.length !== 4 || parts[0] !== 'u') {
    return null
  }

  return parts[1] || null
}
