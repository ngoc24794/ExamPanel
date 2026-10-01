import * as React from 'react'
import { api, type ThemeMode } from '@/lib/api'
import { ThemeContext } from './context'

interface ThemeProviderProps {
  children: React.ReactNode
  defaultTheme?: ThemeMode
}

export function ThemeProvider({ children, defaultTheme = 'system' }: ThemeProviderProps) {
  const [theme, setThemeState] = React.useState<ThemeMode>(defaultTheme)
  const [systemDark, setSystemDark] = React.useState<boolean>(() => {
    if (typeof window === 'undefined') return false
    return window.matchMedia('(prefers-color-scheme: dark)').matches
  })

  // Load persisted theme on mount
  React.useEffect(() => {
    let mounted = true
    api.getTheme().then((saved) => {
      if (mounted && saved) {
        setThemeState(saved)
      }
    })
    return () => {
      mounted = false
    }
  }, [])

  // Listen to system dark mode changes
  React.useEffect(() => {
    if (typeof window === 'undefined') return
    const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)')
    const handleChange = (e: MediaQueryListEvent) => {
      setSystemDark(e.matches)
    }
    mediaQuery.addEventListener('change', handleChange)
    return () => mediaQuery.removeEventListener('change', handleChange)
  }, [])

  const resolvedTheme: 'light' | 'dark' =
    theme === 'system' ? (systemDark ? 'dark' : 'light') : theme

  // Apply dark class to <html>
  React.useEffect(() => {
    const root = document.documentElement
    root.classList.remove('light', 'dark')
    root.classList.add(resolvedTheme)
  }, [resolvedTheme])

  const setTheme = React.useCallback((newTheme: ThemeMode) => {
    setThemeState(newTheme)
    api.setTheme(newTheme).catch((err) => {
      console.error('Failed to persist theme:', err)
    })
  }, [])

  return (
    <ThemeContext.Provider value={{ theme, resolvedTheme, setTheme }}>
      {children}
    </ThemeContext.Provider>
  )
}
