export const morandiColors = {
  primary: '#2f2f2f',
  primaryDark: '#111111',
  success: '#4f4f4f',
  warning: '#6a6a6a',
  error: '#C94D45',
  info: '#5a5a5a',

  bg: '#f2f2f2',
  bgLight: '#f5f5f5',
  bgDark: '#ebebeb',
  panel: '#f5f5f5',
  sidebar: '#f5f5f5',
  sidebarHover: '#efefef',

  text: '#141414',
  textLight: '#5f5f5f',
  textLighter: '#8f8f8f',
  textOnDark: '#f2f2f2',

  border: '#d0d0d0',
  borderLight: '#e4e4e4',
}

/**
 * Naive UI 主题覆盖配置
 */
export const naiveUITheme = {
  common: {
    primaryColor: morandiColors.primary,
    successColor: morandiColors.success,
    warningColor: morandiColors.warning,
    errorColor: morandiColors.error,
    infoColor: morandiColors.info,

    baseColor: morandiColors.bg,
    bodyColor: morandiColors.bgLight,
    cardColor: morandiColors.panel,
    modalColor: morandiColors.panel,

    textColorBase: morandiColors.text,
    textColor1: morandiColors.text,
    textColor2: morandiColors.textLight,
    textColor3: morandiColors.textLighter,

    borderColor: morandiColors.border,

    fontFamily:
      '"ui-sans-serif", "-apple-system", "system-ui", "Segoe UI", "Helvetica", "Apple Color Emoji", "Arial", "sans-serif", "Segoe UI Emoji", "Segoe UI Symbol"',
    fontSize: '14px',
    fontSizeSmall: '12px',
    fontSizeLarge: '16px',
  },
  Button: {
    colorPrimary: morandiColors.primary,
    textColorPrimary: '#f5f5f5',
    colorPrimaryHover: '#434343',
    colorPrimaryPressed: '#262626',
  },
  Input: {
    borderRadius: '10px',
    borderColor: morandiColors.border,
    colorTarget: morandiColors.bgLight,
  },
  Modal: {
    borderRadius: '10px',
    boxShadow: '0 10px 18px rgba(0, 0, 0, 0.08)',
  },
}

/**
 * CSS 变量定义（用于全局样式）
 */
export const generateCSSVariables = () => {
  const vars: Record<string, string> = {}

  Object.entries(morandiColors).forEach(([key, value]) => {
    vars[`--color-${key}`] = value
  })

  vars['--spacing-xs'] = '4px'
  vars['--spacing-sm'] = '8px'
  vars['--spacing-md'] = '12px'
  vars['--spacing-lg'] = '18px'
  vars['--spacing-xl'] = '24px'
  vars['--spacing-2xl'] = '36px'

  vars['--radius-sm'] = '6px'
  vars['--radius-md'] = '8px'
  vars['--radius-lg'] = '10px'
  vars['--radius-xl'] = '12px'

  vars['--shadow-sm'] = '0 4px 10px rgba(0, 0, 0, 0.05)'
  vars['--shadow-md'] = '0 8px 16px rgba(0, 0, 0, 0.08)'
  vars['--shadow-lg'] = '0 12px 24px rgba(0, 0, 0, 0.1)'
  vars['--shadow-inset'] = 'none'
  vars['--surface-outline'] = 'none'

  vars['--transition-base'] = 'all 0.22s cubic-bezier(0.2, 0.8, 0.2, 1)'

  return vars
}
