import * as React from 'react'
import { useTranslation } from 'react-i18next'
import { Sun, Moon, Laptop } from 'lucide-react'
import { useTheme } from './useTheme'
import { Button } from '@/components/ui/button'
import type { ThemeMode } from '@/lib/api'

export function ThemeToggle() {
  const { theme, setTheme } = useTheme()
  const { t } = useTranslation()
  const [open, setOpen] = React.useState(false)
  const dropdownRef = React.useRef<HTMLDivElement>(null)

  React.useEffect(() => {
    function handleClickOutside(event: MouseEvent) {
      if (dropdownRef.current && !dropdownRef.current.contains(event.target as Node)) {
        setOpen(false)
      }
    }
    document.addEventListener('mousedown', handleClickOutside)
    return () => document.removeEventListener('mousedown', handleClickOutside)
  }, [])

  const modes: { key: ThemeMode; label: string; icon: React.ReactNode }[] = [
    { key: 'light', label: t('theme.light'), icon: <Sun className="h-4 w-4" /> },
    { key: 'dark', label: t('theme.dark'), icon: <Moon className="h-4 w-4" /> },
    {
      key: 'system',
      label: t('theme.system'),
      icon: <Laptop className="h-4 w-4" />,
    },
  ]

  const currentIcon =
    theme === 'dark' ? (
      <Moon className="h-4 w-4" />
    ) : theme === 'light' ? (
      <Sun className="h-4 w-4" />
    ) : (
      <Laptop className="h-4 w-4" />
    )

  return (
    <div className="relative inline-block text-left" ref={dropdownRef}>
      <Button
        variant="outline"
        size="sm"
        onClick={() => setOpen(!open)}
        className="flex items-center gap-1.5"
        aria-label={t('theme.toggle')}
      >
        {currentIcon}
        <span className="hidden sm:inline-block text-xs font-normal capitalize">
          {t(`theme.${theme}`)}
        </span>
      </Button>

      {open && (
        <div className="absolute right-0 mt-2 w-36 rounded-md border bg-popover text-popover-foreground shadow-md z-50 py-1">
          {modes.map((mode) => (
            <button
              key={mode.key}
              onClick={() => {
                setTheme(mode.key)
                setOpen(false)
              }}
              className={`w-full flex items-center gap-2 px-3 py-1.5 text-xs text-left hover:bg-accent hover:text-accent-foreground ${
                theme === mode.key ? 'font-semibold text-primary' : ''
              }`}
            >
              {mode.icon}
              <span>{mode.label}</span>
            </button>
          ))}
        </div>
      )}
    </div>
  )
}
