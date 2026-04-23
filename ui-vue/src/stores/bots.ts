/**
 * Bot 状态管理（Pinia）
 */

import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { getBotList, createBot, deleteBot, updateBot } from '@/services/bot'
import type { Bot, CreateBotRequest, UpdateBotRequest } from '@/types'
import { getErrorMessage } from '@/utils/error'

export const useBotStore = defineStore('bots', () => {
  // 状态
  const bots = ref<Bot[]>([])
  const loading = ref(false)
  const error = ref<string | null>(null)

  const botsCount = computed(() => bots.value.length)

  // 方法：获取 Bot 列表
  const fetchBots = async () => {
    loading.value = true
    error.value = null

    try {
      bots.value = await getBotList()
      return bots.value
    } catch (err: any) {
      error.value = getErrorMessage(err, '获取机器人列表失败')
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
      bots.value.unshift(newBot)

      return newBot
    } catch (err: any) {
      error.value = getErrorMessage(err, '创建机器人失败')
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

      bots.value = bots.value.filter((b: Bot) => b.bot_id !== botId)
    } catch (err: any) {
      error.value = getErrorMessage(err, '删除机器人失败')
      throw err
    } finally {
      loading.value = false
    }
  }

  const updateBotInfo = async (botId: string, data: UpdateBotRequest) => {
    loading.value = true
    error.value = null

    try {
      const updatedBot = await updateBot(botId, data)
      const index = bots.value.findIndex((bot) => bot.bot_id === botId)
      if (index >= 0) {
        bots.value[index] = updatedBot
      }
      return updatedBot
    } catch (err: any) {
      error.value = getErrorMessage(err, '更新机器人失败')
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
    bots,
    loading,
    error,

    // 计算属性
    botsCount,

    // 方法
    fetchBots,
    addBot,
    removeBotById,
    updateBotInfo,
    clearError,
  }
})
