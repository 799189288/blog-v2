import { createI18n } from 'vue-i18n'
import en from './locales/en'
import zh from './locales/zh'

export type Locale = 'en' | 'zh'

const STORAGE_KEY = 'admin.locale'

function detectLocale(): Locale {
  const saved = localStorage.getItem(STORAGE_KEY)
  if (saved === 'en' || saved === 'zh') return saved
  const nav = navigator.language?.toLowerCase() ?? ''
  return nav.startsWith('zh') ? 'zh' : 'en'
}

const i18n = createI18n({
  legacy: false,
  locale: detectLocale(),
  fallbackLocale: 'en',
  messages: { en, zh },
})

export function setLocale(locale: Locale) {
  i18n.global.locale.value = locale
  localStorage.setItem(STORAGE_KEY, locale)
  document.documentElement.lang = locale === 'zh' ? 'zh-CN' : 'en'
}

document.documentElement.lang = i18n.global.locale.value === 'zh' ? 'zh-CN' : 'en'

export default i18n
