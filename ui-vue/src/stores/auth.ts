/**
 * 认证状态管理（Pinia）
 */

import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { login, logout, register } from '@/services/auth'
import { STORAGE_KEYS } from '@/constants/storageKeys'
import type { User } from '@/types'

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

  // 方法：注册
  const handleRegister = async (phone: string, idNumber: string) => {
    loading.value = true
    error.value = null

    try {
      // 先注册用户
      await register(phone, idNumber)

      // 注册成功后自动登录
      const response = await login(phone, idNumber)
      user.value = response
      token.value = response.bot_token

      // 保存到 localStorage
      localStorage.setItem(STORAGE_KEYS.TOKEN, response.bot_token)
      localStorage.setItem(STORAGE_KEYS.USER, JSON.stringify(response))

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
      user.value = response
      token.value = response.bot_token

      // 保存到 localStorage
      localStorage.setItem(STORAGE_KEYS.TOKEN, response.bot_token)
      localStorage.setItem(STORAGE_KEYS.USER, JSON.stringify(response))

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

    // 方法
    initializeAuth,
    handleRegister,
    handleLogin,
    handleLogout,
    clearError,
  }
})
