import * as React from 'react'
import type { ThemeMode } from '@/lib/api'

export interface ThemeContextType {
  theme: ThemeMode
  resolvedTheme: 'light' | 'dark'
  setTheme: (theme: ThemeMode) => void
}

export const ThemeContext = React.createContext<ThemeContextType | undefined>(undefined)
