/**
 * API 层封装
 * 配置 axios 实例、拦截器、错误处理
 */

import axios, { type AxiosInstance, type AxiosRequestConfig, type AxiosError } from 'axios'
import { STORAGE_KEYS } from '@/constants/storageKeys'
import type { ApiResponse } from '@/types'
import { getErrorMessage } from '@/utils/error'

// 获取 API 基础 URL
const API_BASE_URL = import.meta.env.VITE_API_BASE_URL || 'http://localhost:8080/api/v1'

class ApiClient {
  private instance: AxiosInstance
  private token: string | null = null

  constructor() {
    this.instance = axios.create({
      baseURL: API_BASE_URL,
      headers: {
        'Content-Type': 'application/json',
      },
      timeout: 30000,
    })

    // 初始化 token
    this.token = this.getStoredToken()

    // 请求拦截器
    this.instance.interceptors.request.use(
      (config) => {
        // 每次请求时动态获取最新的 token。若调用方已显式传 Authorization，则优先保留。
        if (config.headers?.Authorization) {
          return config
        }

        const currentToken = localStorage.getItem(STORAGE_KEYS.TOKEN)
        if (currentToken) {
          config.headers.Authorization = `Bearer ${currentToken}`
        }
        return config
      },
      (error) => {
        return Promise.reject(error)
      }
    )

    // 响应拦截器
    this.instance.interceptors.response.use(
      (response) => {
        const data = response.data as ApiResponse

        // 仅在后端返回了标准 code 字段时，才按业务码判定成功/失败。
        // 兼容当前后端直接返回业务数据（不带 code/data 包装）的接口。
        if (typeof data.code === 'number') {
          if (data.code !== 0 && data.code !== 200) {
            const error = new Error(data.message || '请求失败') as any
            error.code = data.code
            error.data = data
            return Promise.reject(error)
          }

          return data.data || data
        }

        return data
      },
      (error: AxiosError<ApiResponse>) => {
        // 处理 HTTP 错误
        const status = error.response?.status
        const responseData = error.response?.data as ApiResponse & { error?: string } | undefined
        const code = (responseData as any)?.code as string | undefined
        const message = getErrorMessage(
          {
            code,
            status,
            message: responseData?.message || responseData?.error || error.message,
          },
          '请求失败'
        )

        if (this.shouldForceLogout(status, code, error.config?.url)) {
          this.clearAuthState()
          if (window.location.pathname !== '/login') {
            window.location.assign('/login')
          }
        }

        const appError = new Error(message) as any
        appError.code = code
        appError.status = status
        appError.originalError = error

        return Promise.reject(appError)
      }
    )
  }

  /**
   * 设置认证 token
   */
  setToken(token: string) {
    this.token = token
    localStorage.setItem(STORAGE_KEYS.TOKEN, token)
  }

  /**
   * 获取存储的 token
   */
  private getStoredToken(): string | null {
    return localStorage.getItem(STORAGE_KEYS.TOKEN)
  }

  /**
   * 清除 token
   */
  clearToken() {
    this.token = null
    localStorage.removeItem(STORAGE_KEYS.TOKEN)
  }

  private clearAuthState() {
    this.clearToken()
    localStorage.removeItem(STORAGE_KEYS.USER)
    localStorage.removeItem(STORAGE_KEYS.SELECTED_GROUP_ID)
    localStorage.removeItem(STORAGE_KEYS.CURRENT_GROUP_ID)
  }

  private shouldForceLogout(
    status: number | undefined,
    code: string | undefined,
    requestUrl: string | undefined
  ): boolean {
    const normalizedUrl = (requestUrl || '').toLowerCase()
    const isAuthEndpoint = normalizedUrl.includes('/auth/login') || normalizedUrl.includes('/auth/register')
    if (isAuthEndpoint) {
      return false
    }

    const normalizedCode = (code || '').toLowerCase()
    const userAuthCodes = ['invalid_user_token', 'user_token_required', 'invalid_token', 'unauthorized']
    if (userAuthCodes.includes(normalizedCode)) {
      return true
    }

    const botAuthCodes = ['invalid_bot_token', 'bot_token_required']
    if (botAuthCodes.includes(normalizedCode)) {
      return false
    }

    // 后端若仅返回 401 但未携带业务 code，则按路径判断是否属于用户身份接口。
    if (status === 401) {
      return this.isUserAuthScope(normalizedUrl)
    }

    return false
  }

  private isUserAuthScope(normalizedUrl: string): boolean {
    if (
      normalizedUrl.includes('/users/') ||
      normalizedUrl.includes('/bots') ||
      normalizedUrl === '/groups' ||
      normalizedUrl.startsWith('/groups?') ||
      normalizedUrl.includes('/groups/join') ||
      (normalizedUrl.includes('/groups/') && !normalizedUrl.includes('/messages'))
    ) {
      return true
    }
    return false
  }

  /**
   * 获取当前 token
   */
  getToken(): string | null {
    return localStorage.getItem(STORAGE_KEYS.TOKEN)
  }

  /**
   * GET 请求
   */
  get<T = any>(url: string, config?: AxiosRequestConfig): Promise<T> {
    return this.instance.get<any, T>(url, config)
  }

  /**
   * POST 请求
   */
  post<T = any>(url: string, data?: any, config?: AxiosRequestConfig): Promise<T> {
    return this.instance.post<any, T>(url, data, config)
  }

  /**
   * PUT 请求
   */
  put<T = any>(url: string, data?: any, config?: AxiosRequestConfig): Promise<T> {
    return this.instance.put<any, T>(url, data, config)
  }

  /**
   * DELETE 请求
   */
  delete<T = any>(url: string, config?: AxiosRequestConfig): Promise<T> {
    return this.instance.delete<any, T>(url, config)
  }

  /**
   * PATCH 请求
   */
  patch<T = any>(url: string, data?: any, config?: AxiosRequestConfig): Promise<T> {
    return this.instance.patch<any, T>(url, data, config)
  }
}

// 导出单例
export const apiClient = new ApiClient()

export function unwrapCollectionResponse<T>(
  response: T[] | { bots?: T[]; groups?: T[]; messages?: T[]; members?: T[]; requests?: T[] }
): T[] {
  if (Array.isArray(response)) {
    return response
  }

  if (Array.isArray(response.bots)) {
    return response.bots
  }

  if (Array.isArray(response.groups)) {
    return response.groups
  }

  if (Array.isArray(response.messages)) {
    return response.messages
  }

  if (Array.isArray(response.members)) {
    return response.members
  }

  if (Array.isArray(response.requests)) {
    return response.requests
  }

  return []
}

export default apiClient
