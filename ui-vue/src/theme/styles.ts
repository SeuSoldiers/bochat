/**
 * 全局样式
 * 莫兰迪色系主题应用
 */

import { generateCSSVariables } from '@/theme/colors'

export function applyGlobalStyles() {
  const cssVars = generateCSSVariables()
  const root = document.documentElement

  // 应用所有 CSS 变量
  Object.entries(cssVars).forEach(([key, value]) => {
    root.style.setProperty(key, value as string)
  })
}

// 导出一个样式对象用于 Vue 应用
export const globalStyleVars = generateCSSVariables()
