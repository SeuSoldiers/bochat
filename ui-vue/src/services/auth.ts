/**
 * 认证服务
 */

import { apiClient } from './api'
import type { User } from '@/types'

/**
 * 用户注册
 */
export async function register(name: string, phone: string, idNumber: string) {
  return apiClient.post<User>('/auth/register', {
    name,
    phone,
    id_number: idNumber,
  })
}

/**
 * 用户登录
 */
export async function login(phone: string, idNumber: string) {
  const response = await apiClient.post<User & { bot_token: string }>('/auth/login', {
    phone,
    id_number: idNumber,
  })

  // 保存 token
  if (response.bot_token) {
    apiClient.setToken(response.bot_token)
    localStorage.setItem('user', JSON.stringify(response))
  }

  return response
}

/**
 * 登出
 */
export async function logout() {
  apiClient.clearToken()
  localStorage.removeItem('user')
}
