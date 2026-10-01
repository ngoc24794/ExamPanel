import * as React from 'react'
import { useTranslation } from 'react-i18next'
import { Languages } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { setLanguage } from '@/i18n'

export function LanguageSwitcher() {
  const { i18n, t } = useTranslation()
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

  const currentLang = i18n.language?.startsWith('en') ? 'en' : 'vi'

  return (
    <div className="relative inline-block text-left" ref={dropdownRef}>
      <Button
        variant="outline"
        size="sm"
        onClick={() => setOpen(!open)}
        className="flex items-center gap-1.5"
        aria-label={t('language.select')}
      >
        <Languages className="h-4 w-4" />
        <span className="hidden sm:inline-block text-xs uppercase font-medium">
          {currentLang}
        </span>
      </Button>

      {open && (
        <div className="absolute right-0 mt-2 w-32 rounded-md border bg-popover text-popover-foreground shadow-md z-50 py-1">
          <button
            onClick={() => {
              setLanguage('vi')
              setOpen(false)
            }}
            className={`w-full px-3 py-1.5 text-xs text-left hover:bg-accent hover:text-accent-foreground ${
              currentLang === 'vi' ? 'font-semibold text-primary' : ''
            }`}
          >
            🇻🇳 {t('language.vi')}
          </button>
          <button
            onClick={() => {
              setLanguage('en')
              setOpen(false)
            }}
            className={`w-full px-3 py-1.5 text-xs text-left hover:bg-accent hover:text-accent-foreground ${
              currentLang === 'en' ? 'font-semibold text-primary' : ''
            }`}
          >
            🇬🇧 {t('language.en')}
          </button>
        </div>
      )}
    </div>
  )
}
