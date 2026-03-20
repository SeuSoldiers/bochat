/**
 * 认证状态管理（Pinia）
 */

import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { login, logout, register } from '@/services/auth'
import { STORAGE_KEYS } from '@/constants/storageKeys'
import type { User } from '@/types'

type LoginResponse = User & { bot_token: string }

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
  const userId = computed(() => user.value?.id || '')
  const userPhone = computed(() => user.value?.phone || '')
  const userName = computed(() => user.value?.name || '')

  // 方法：注册
  const handleRegister = async (name: string, phone: string, idNumber: string) => {
    loading.value = true
    error.value = null

    try {
      await register(name, phone, idNumber)

      const response = await login(phone, idNumber)
      user.value = extractUser(response)
      token.value = response.bot_token

      localStorage.setItem(STORAGE_KEYS.TOKEN, response.bot_token)
      localStorage.setItem(STORAGE_KEYS.USER, JSON.stringify(user.value))

      return response
    } catch (err: any) {
      error.value = err.message || '注册失败'
      throw err
    } finally {
      loading.value = false
    }
  }

  // 方法：登录
  const handleLogin = async (phone: string, idNumber: string) => {
    loading.value = true
    error.value = null

    try {
      const response = await login(phone, idNumber)
      user.value = extractUser(response)
      token.value = response.bot_token

      localStorage.setItem(STORAGE_KEYS.TOKEN, response.bot_token)
      localStorage.setItem(STORAGE_KEYS.USER, JSON.stringify(user.value))

      return response
    } catch (err: any) {
      error.value = err.message || '登录失败'
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
      error.value = err.message || '登出失败'
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
    handleLogout,
    clearError,
  }
})

function extractUser(response: LoginResponse): User {
  return {
    id: response.id,
    name: response.name,
    phone: response.phone,
    id_number: response.id_number,
    created_at: response.created_at,
  }
}
