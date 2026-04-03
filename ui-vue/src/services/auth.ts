/**
 * 认证服务
 */

import { apiClient } from './api'
import type { User } from '@/types'

export interface AuthCredentialsPayload {
  account?: string
  password?: string
}

export interface RegisterPayload extends AuthCredentialsPayload {
  name?: string
}

export type LoginResponse = User & { token: string }

/**
 * 用户注册
 */
export async function register(payload: RegisterPayload) {
  return apiClient.post<LoginResponse>('/auth/register', payload)
}

/**
 * 用户登录
 */
export async function login(payload: AuthCredentialsPayload) {
  const response = await apiClient.post<LoginResponse>('/auth/login', payload)

  // 保存 token
  if (response.token) {
    apiClient.setToken(response.token)
    localStorage.setItem('user', JSON.stringify(response))
  }

  return response
}

export async function getCurrentUser() {
  return apiClient.get<User>('/users/me')
}

export async function updateCurrentUser(payload: {
  name?: string
  phone?: string
  avatar_url?: string
  password?: string
}) {
  return apiClient.put<User>('/users/me', payload)
}

/**
 * 登出
 */
export async function logout() {
  apiClient.clearToken()
  localStorage.removeItem('user')
}
