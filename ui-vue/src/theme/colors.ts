export const morandiColors = {
  primary: '#A6D72E',
  primaryDark: '#0E3C2F',
  success: '#2E8A5C',
  warning: '#F1A33D',
  error: '#C94D45',
  info: '#2E6E5D',

  bg: '#ECEDEC',
  bgLight: '#F7F8F6',
  bgDark: '#E2E5DF',
  panel: '#FFFFFF',
  sidebar: '#012F23',
  sidebarHover: '#0A4735',

  text: '#17221E',
  textLight: '#5E6B66',
  textLighter: '#96A09A',
  textOnDark: '#EAF0EC',

  border: '#D8DFD8',
  borderLight: '#E8ECE8',
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

    fontFamily: '"Space Grotesk", "Manrope", "Noto Sans SC", sans-serif',
    fontSize: '14px',
    fontSizeSmall: '12px',
    fontSizeLarge: '16px',
  },
  Button: {
    colorPrimary: morandiColors.primary,
    textColorPrimary: '#132019',
    colorPrimaryHover: '#B9E349',
    colorPrimaryPressed: '#98C52C',
  },
  Input: {
    borderRadius: '10px',
    borderColor: morandiColors.border,
    colorTarget: morandiColors.bgLight,
  },
  Modal: {
    borderRadius: '16px',
    boxShadow: '0 24px 48px rgba(9, 30, 22, 0.16)',
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

  vars['--radius-sm'] = '8px'
  vars['--radius-md'] = '12px'
  vars['--radius-lg'] = '18px'
  vars['--radius-xl'] = '24px'

  vars['--shadow-sm'] = '0 6px 14px rgba(7, 27, 19, 0.06)'
  vars['--shadow-md'] = '0 12px 30px rgba(7, 27, 19, 0.1)'
  vars['--shadow-lg'] = '0 28px 48px rgba(7, 27, 19, 0.18)'
  vars['--shadow-inset'] = 'inset 0 1px 0 rgba(255, 255, 255, 0.55)'
  vars['--surface-outline'] = '0 0 0 1px rgba(255, 255, 255, 0.66), inset 0 0 0 1px #dce4db'

  vars['--transition-base'] = 'all 0.22s cubic-bezier(0.2, 0.8, 0.2, 1)'

  return vars
}
