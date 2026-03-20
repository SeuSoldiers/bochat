/**
 * API 层封装
 * 配置 axios 实例、拦截器、错误处理
 */

import axios, { type AxiosInstance, type AxiosRequestConfig, type AxiosError } from 'axios'
import { STORAGE_KEYS } from '@/constants/storageKeys'
import type { ApiResponse } from '@/types'

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
        // 每次请求时动态获取最新的 token
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

        // 检查业务逻辑错误
        if (data.code !== 0 && data.code !== 200) {
          const error = new Error(data.message || '请求失败') as any
          error.code = data.code
          error.data = data
          return Promise.reject(error)
        }

        return data.data || data
      },
      (error: AxiosError<ApiResponse>) => {
        // 处理 HTTP 错误
        const status = error.response?.status
        const message = error.response?.data?.message || error.message

        // 处理 401 未授权错误
        if (status === 401) {
          this.clearToken()
          window.location.href = '/login'
        }

        const appError = new Error(message) as any
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

  /**
   * 获取当前 token
   */
  getToken(): string | null {
    return this.token
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

export default apiClient
