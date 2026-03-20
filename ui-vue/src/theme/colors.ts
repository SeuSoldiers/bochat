/**
 * 莫兰迪色系配置
 * 低饱和度、高级感的配色方案
 */

export const morandiColors = {
  // 主色调（莫兰迪绿）
  primary: '#8B9D83',

  // 辅助色
  success: '#9B8B7E',      // 莫兰迪棕
  warning: '#C9A876',      // 莫兰迪黄
  error: '#A88B7F',        // 莫兰迪粉
  info: '#7A9BA6',         // 莫兰迪蓝

  // 背景色
  bg: '#F5F3F1',           // 米色背景
  bgLight: '#FAFAF8',      // 极浅米色
  bgDark: '#EBE6E1',       // 深米色

  // 文字色
  text: '#4A4A4A',         // 深灰文字
  textLight: '#888888',    // 浅灰文字
  textLighter: '#CCCCCC',  // 更浅灰

  // 边框色
  border: '#D4CFC8',       // 浅灰边框
  borderLight: '#E8E3DD',  // 更浅边框
}

/**
 * Naive UI 主题覆盖配置
 */
export const naiveUITheme = {
  common: {
    // 主色调
    primaryColor: morandiColors.primary,
    successColor: morandiColors.success,
    warningColor: morandiColors.warning,
    errorColor: morandiColors.error,
    infoColor: morandiColors.info,

    // 背景
    baseColor: morandiColors.bg,
    bodyColor: morandiColors.bgLight,
    cardColor: '#FFFFFF',
    modalColor: '#FFFFFF',

    // 文字
    textColorBase: morandiColors.text,
    textColor1: morandiColors.text,
    textColor2: morandiColors.textLight,
    textColor3: morandiColors.textLighter,

    // 边框
    borderColor: morandiColors.border,

    // 其他
    fontFamily: '-apple-system, BlinkMacSystemFont, "Segoe UI", "Roboto", "Oxygen", "Ubuntu", "Cantarell", sans-serif',
    fontSize: '14px',
    fontSizeSmall: '12px',
    fontSizeLarge: '16px',
  },
  Button: {
    colorPrimary: morandiColors.primary,
    textColorPrimary: '#FFFFFF',
    colorPrimaryHover: '#9CAA93',
    colorPrimaryPressed: '#7A8C72',
  },
  Input: {
    borderRadius: '6px',
    borderColor: morandiColors.border,
    colorTarget: morandiColors.bg,
  },
  Modal: {
    borderRadius: '8px',
    boxShadow: '0 2px 12px rgba(0, 0, 0, 0.08)',
  },
}

/**
 * CSS 变量定义（用于全局样式）
 */
export const generateCSSVariables = () => {
  const vars: Record<string, string> = {}

  // 颜色变量
  Object.entries(morandiColors).forEach(([key, value]) => {
    vars[`--color-${key}`] = value
  })

  // 空间变量
  vars['--spacing-xs'] = '4px'
  vars['--spacing-sm'] = '8px'
  vars['--spacing-md'] = '12px'
  vars['--spacing-lg'] = '16px'
  vars['--spacing-xl'] = '24px'
  vars['--spacing-2xl'] = '32px'

  // 圆角变量
  vars['--radius-sm'] = '4px'
  vars['--radius-md'] = '6px'
  vars['--radius-lg'] = '8px'

  // 阴影变量
  vars['--shadow-sm'] = '0 1px 2px rgba(0, 0, 0, 0.05)'
  vars['--shadow-md'] = '0 2px 4px rgba(0, 0, 0, 0.08)'
  vars['--shadow-lg'] = '0 4px 12px rgba(0, 0, 0, 0.1)'

  // 过渡
  vars['--transition-base'] = 'all 0.3s ease'

  return vars
}
