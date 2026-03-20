/**
 * 本地存储键值常量统一管理
 */

export const STORAGE_KEYS = {
  // 认证相关
  TOKEN: 'bot_token',
  USER: 'user',

  // Bot 相关
  SELECTED_BOT_ID: 'selectedBotId',

  // 群组相关
  SELECTED_GROUP_ID: 'selectedGroupId',
  CURRENT_GROUP_ID: 'currentGroupId',
} as const
