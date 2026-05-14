/**
 * 本地存储键值常量统一管理
 */

export const STORAGE_KEYS = {
  // 认证相关
  TOKEN: 'bot_token',
  USER: 'user',
  BOT_USER_TOKEN: 'bot_user_token',
  BOT_USER_PROFILE: 'bot_user_profile',

  // 群组相关
  SELECTED_GROUP_ID: 'selectedGroupId',
  CURRENT_GROUP_ID: 'currentGroupId',
  BOT_SELECTED_GROUP_ID: 'botSelectedGroupId',
} as const
