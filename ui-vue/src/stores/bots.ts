/**
 * Bot 状态管理（Pinia）
 */

import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { getBotList, createBot, deleteBot, updateBot } from '@/services/bot'
import { STORAGE_KEYS } from '@/constants/storageKeys'
import type { Bot, CreateBotRequest } from '@/types'

export const useBotStore = defineStore('bots', () => {
  // 状态
  const bots = ref<Bot[]>([])
  const selectedBotId = ref<string | null>(null)
  const loading = ref(false)
  const error = ref<string | null>(null)

  // 从 localStorage 恢复选中的 Bot
  const initializeSelectedBot = () => {
    const stored = localStorage.getItem(STORAGE_KEYS.SELECTED_BOT_ID)
    if (stored) {
      selectedBotId.value = stored
    }
  }

  // 计算属性
  const selectedBot = computed(() => {
    return bots.value.find((b: Bot) => b.bot_id === selectedBotId.value) || null
  })

  const botsCount = computed(() => bots.value.length)

  // 方法：获取 Bot 列表
  const fetchBots = async () => {
    loading.value = true
    error.value = null

    try {
      bots.value = await getBotList()
      return bots.value
    } catch (err: any) {
      error.value = err.message || '获取 Bot 列表失败'
      throw err
    } finally {
      loading.value = false
    }
  }

  // 方法：创建 Bot
  const addBot = async (data: CreateBotRequest) => {
    loading.value = true
    error.value = null

    try {
      const newBot = await createBot(data)
      bots.value.push(newBot)

      // 自动选中新创建的 Bot
      selectedBotId.value = newBot.bot_id
      localStorage.setItem(STORAGE_KEYS.SELECTED_BOT_ID, newBot.bot_id)

      return newBot
    } catch (err: any) {
      error.value = err.message || '创建 Bot 失败'
      throw err
    } finally {
      loading.value = false
    }
  }

  // 方法：删除 Bot
  const removeBotById = async (botId: string) => {
    loading.value = true
    error.value = null

    try {
      await deleteBot(botId)

      // 从列表中删除
      bots.value = bots.value.filter((b: Bot) => b.bot_id !== botId)

      // 如果删除的是当前选中的 Bot，清除选中
      if (selectedBotId.value === botId) {
        selectedBotId.value = bots.value.length > 0 ? bots.value[0].bot_id : null
        if (selectedBotId.value) {
          localStorage.setItem(STORAGE_KEYS.SELECTED_BOT_ID, selectedBotId.value)
        } else {
          localStorage.removeItem(STORAGE_KEYS.SELECTED_BOT_ID)
        }
      }
    } catch (err: any) {
      error.value = err.message || '删除 Bot 失败'
      throw err
    } finally {
      loading.value = false
    }
  }

  // 方法：更新 Bot
  const updateBotInfo = async (botId: string, data: Partial<CreateBotRequest>) => {
    loading.value = true
    error.value = null

    try {
      const updated = await updateBot(botId, data)

      const index = bots.value.findIndex((b: Bot) => b.bot_id === botId)
      if (index >= 0) {
        bots.value[index] = updated
      }

      return updated
    } catch (err: any) {
      error.value = err.message || '更新 Bot 失败'
      throw err
    } finally {
      loading.value = false
    }
  }

  // 方法：选择 Bot
  const selectBot = (botId: string) => {
    const bot = bots.value.find((b: Bot) => b.bot_id === botId)
    if (bot) {
      selectedBotId.value = botId
      localStorage.setItem(STORAGE_KEYS.SELECTED_BOT_ID, botId)
    }
  }

  // 方法：清除错误
  const clearError = () => {
    error.value = null
  }

  return {
    // 状态
    bots,
    selectedBotId,
    loading,
    error,

    // 计算属性
    selectedBot,
    botsCount,

    // 方法
    initializeSelectedBot,
    fetchBots,
    addBot,
    removeBotById,
    updateBotInfo,
    selectBot,
    clearError,
  }
})
