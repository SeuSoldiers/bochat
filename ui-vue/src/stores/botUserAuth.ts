import { computed, ref } from 'vue'
import { defineStore } from 'pinia'
import { STORAGE_KEYS } from '@/constants/storageKeys'
import { getCurrentBotProfile } from '@/services/botUser'
import type { Bot } from '@/types'
import { getErrorMessage } from '@/utils/error'

export const useBotUserAuthStore = defineStore('botUserAuth', () => {
  const bot = ref<Bot | null>(null)
  const token = ref<string | null>(null)
  const loading = ref(false)
  const error = ref<string | null>(null)

  const isAuthenticated = computed(() => !!token.value && !!bot.value)
  const botId = computed(() => bot.value?.bot_id || parseBotIdFromToken(token.value) || '')
  const botName = computed(() => bot.value?.name || 'Bot')

  const initializeAuth = () => {
    const storedToken = localStorage.getItem(STORAGE_KEYS.BOT_USER_TOKEN)
    const storedBot = localStorage.getItem(STORAGE_KEYS.BOT_USER_PROFILE)

    if (storedToken) {
      token.value = storedToken
    }

    if (storedBot) {
      try {
        bot.value = JSON.parse(storedBot)
      } catch {
        localStorage.removeItem(STORAGE_KEYS.BOT_USER_PROFILE)
      }
    }
  }

  const handleLogin = async (botToken: string) => {
    loading.value = true
    error.value = null

    try {
      const normalizedToken = botToken.trim()
      const profile = await getCurrentBotProfile(normalizedToken)

      token.value = normalizedToken
      bot.value = profile
      localStorage.setItem(STORAGE_KEYS.BOT_USER_TOKEN, normalizedToken)
      localStorage.setItem(STORAGE_KEYS.BOT_USER_PROFILE, JSON.stringify(profile))

      return profile
    } catch (err: any) {
      error.value = getErrorMessage(err, 'Bot Token 登录失败')
      throw err
    } finally {
      loading.value = false
    }
  }

  const handleLogout = () => {
    bot.value = null
    token.value = null
    error.value = null
    localStorage.removeItem(STORAGE_KEYS.BOT_USER_TOKEN)
    localStorage.removeItem(STORAGE_KEYS.BOT_USER_PROFILE)
    localStorage.removeItem(STORAGE_KEYS.BOT_SELECTED_GROUP_ID)
  }

  return {
    bot,
    token,
    loading,
    error,
    isAuthenticated,
    botId,
    botName,
    initializeAuth,
    handleLogin,
    handleLogout,
  }
})

function parseBotIdFromToken(token: string | null): string | null {
  if (!token) return null
  const parts = token.split(':')
  if (parts.length !== 3) return null
  return parts[0] || null
}
