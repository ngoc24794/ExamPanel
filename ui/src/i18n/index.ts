import i18n from 'i18next'
import { initReactI18next } from 'react-i18next'
import viTranslation from './locales/vi.json'
import enTranslation from './locales/en.json'

const resources = {
  vi: { translation: viTranslation },
  en: { translation: enTranslation },
}

const savedLang =
  typeof window !== 'undefined'
    ? localStorage.getItem('exampanel_language') || 'vi'
    : 'vi'

i18n.use(initReactI18next).init({
  resources,
  lng: savedLang,
  fallbackLng: 'en',
  interpolation: {
    escapeValue: false, // React already does escaping
  },
})

export const setLanguage = (lang: 'vi' | 'en') => {
  i18n.changeLanguage(lang)
  if (typeof window !== 'undefined') {
    localStorage.setItem('exampanel_language', lang)
    document.documentElement.lang = lang
  }
}

export default i18n
