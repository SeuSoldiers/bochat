/**
 * 认证服务
 */

import { apiClient } from './api'
import type { User } from '@/types'

/**
 * 用户注册
 */
export async function register(phone: string, idNumber: string) {
  const response = await apiClient.post<User>('/auth/register', {
    phone,
    id_number: idNumber,
  })

  // 注册后需要登录获取 token
  // 注册成功会返回用户信息，但不直接返回 token
  // 需要调用登录接口来获取 token
  return response
}

/**
 * 用户登录
 */
export async function login(phone: string, idNumber: string) {
  const response = await apiClient.post<User>('/auth/login', {
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
  try {
    await apiClient.post('/auth/logout', {})
  } finally {
    apiClient.clearToken()
    localStorage.removeItem('user')
  }
}

/**
 * 获取当前用户信息
 */
export async function getCurrentUser() {
  return apiClient.get<User>('/auth/me')
}
