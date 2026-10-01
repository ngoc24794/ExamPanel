import * as React from 'react'
import { useTranslation } from 'react-i18next'
import {
  LayoutDashboard,
  Users,
  Calendar,
  Sliders,
  ClipboardCheck,
  BarChart3,
  Settings,
  GraduationCap,
  Activity,
  CalendarDays,
  RefreshCw,
} from 'lucide-react'
import { api, type AppInfo } from '@/lib/api'
import { ThemeToggle } from '@/lib/theme/ThemeToggle'
import { LanguageSwitcher } from '@/components/LanguageSwitcher'
import {
  Card,
  CardHeader,
  CardTitle,
  CardDescription,
  CardContent,
} from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'

export type NavItem =
  'overview' | 'teachers' | 'exams' | 'rules' | 'assignments' | 'stats' | 'settings'

export function App() {
  const { t } = useTranslation()
  const [activeTab, setActiveTab] = React.useState<NavItem>('overview')
  const [pingResult, setPingResult] = React.useState<string>('')
  const [loadingPing, setLoadingPing] = React.useState<boolean>(true)
  const [appInfo, setAppInfo] = React.useState<AppInfo | null>(null)

  const fetchPing = React.useCallback(async () => {
    setLoadingPing(true)
    try {
      const res = await api.ping()
      setPingResult(res)
    } catch (err) {
      setPingResult(`Error: ${String(err)}`)
    } finally {
      setLoadingPing(false)
    }
  }, [])

  React.useEffect(() => {
    fetchPing()
    api.getAppInfo().then(setAppInfo).catch(console.error)
  }, [fetchPing])

  const navItems: { key: NavItem; label: string; icon: React.ReactNode }[] = [
    {
      key: 'overview',
      label: t('nav.overview'),
      icon: <LayoutDashboard className="h-4 w-4" />,
    },
    {
      key: 'teachers',
      label: t('nav.teachers'),
      icon: <Users className="h-4 w-4" />,
    },
    {
      key: 'exams',
      label: t('nav.exams'),
      icon: <Calendar className="h-4 w-4" />,
    },
    {
      key: 'rules',
      label: t('nav.rules'),
      icon: <Sliders className="h-4 w-4" />,
    },
    {
      key: 'assignments',
      label: t('nav.assignments'),
      icon: <ClipboardCheck className="h-4 w-4" />,
    },
    {
      key: 'stats',
      label: t('nav.stats'),
      icon: <BarChart3 className="h-4 w-4" />,
    },
    {
      key: 'settings',
      label: t('nav.settings'),
      icon: <Settings className="h-4 w-4" />,
    },
  ]

  return (
    <div className="flex h-screen w-screen overflow-hidden bg-background text-foreground">
      {/* Sidebar */}
      <aside className="w-64 flex-shrink-0 border-r bg-card flex flex-col justify-between">
        <div>
          {/* Brand header in sidebar */}
          <div className="flex items-center gap-3 px-6 h-16 border-b">
            <div className="h-9 w-9 rounded-lg bg-primary text-primary-foreground flex items-center justify-center font-bold">
              <GraduationCap className="h-5 w-5" />
            </div>
            <div>
              <h1 className="font-bold text-base tracking-tight leading-none">
                {t('app.title')}
              </h1>
              <span className="text-[10px] text-muted-foreground uppercase font-medium">
                {appInfo ? `v${appInfo.version}` : 'v0.1.0'}
              </span>
            </div>
          </div>

          {/* Navigation Links */}
          <nav className="p-3 space-y-1">
            {navItems.map((item) => {
              const isActive = activeTab === item.key
              return (
                <button
                  key={item.key}
                  onClick={() => setActiveTab(item.key)}
                  className={`w-full flex items-center gap-3 px-3 py-2 rounded-md text-sm font-medium transition-colors ${
                    isActive
                      ? 'bg-primary text-primary-foreground shadow-sm'
                      : 'text-muted-foreground hover:bg-accent hover:text-accent-foreground'
                  }`}
                >
                  {item.icon}
                  <span>{item.label}</span>
                </button>
              )
            })}
          </nav>
        </div>

        {/* Sidebar Footer */}
        <div className="p-4 border-t text-xs text-muted-foreground space-y-2">
          <div className="flex items-center justify-between">
            <span className="flex items-center gap-1.5">
              <span className="relative flex h-2 w-2">
                <span className="animate-ping absolute inline-flex h-full w-full rounded-full bg-emerald-400 opacity-75"></span>
                <span className="relative inline-flex rounded-full h-2 w-2 bg-emerald-500"></span>
              </span>
              {t('app.ready')}
            </span>
            <Badge variant="outline" className="text-[10px] font-mono">
              {appInfo?.mode === 'tauri' ? 'Tauri' : 'Mock'}
            </Badge>
          </div>
        </div>
      </aside>

      {/* Main Content Area */}
      <div className="flex-1 flex flex-col overflow-hidden">
        {/* Top Header */}
        <header className="h-16 border-b bg-card px-6 flex items-center justify-between flex-shrink-0">
          <div className="flex items-center gap-4">
            <h2 className="text-lg font-semibold tracking-tight">
              {t(`nav.${activeTab}`)}
            </h2>
          </div>

          <div className="flex items-center gap-3">
            {/* School-year selector placeholder */}
            <div className="flex items-center gap-2 border rounded-md px-3 py-1.5 bg-background text-xs">
              <CalendarDays className="h-3.5 w-3.5 text-muted-foreground" />
              <span className="text-muted-foreground font-medium">
                {t('app.schoolYear')}
              </span>
              <span className="font-semibold">{t('app.schoolYearDefault')}</span>
            </div>

            {/* Language Switcher */}
            <LanguageSwitcher />

            {/* Theme Toggle */}
            <ThemeToggle />
          </div>
        </header>

        {/* Content Body */}
        <main className="flex-1 overflow-y-auto p-6 bg-muted/20">
          {activeTab === 'overview' && (
            <div className="max-w-5xl mx-auto space-y-6">
              <div>
                <h3 className="text-2xl font-bold tracking-tight">
                  {t('overview.title')}
                </h3>
                <p className="text-sm text-muted-foreground mt-1">
                  {t('overview.description')}
                </p>
              </div>

              <div className="grid gap-6 md:grid-cols-2">
                {/* Core Engine Status Card */}
                <Card>
                  <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                    <CardTitle className="text-sm font-medium">
                      {t('overview.coreConnectionCard')}
                    </CardTitle>
                    <Activity className="h-4 w-4 text-primary" />
                  </CardHeader>
                  <CardContent className="space-y-4 pt-2">
                    <p className="text-xs text-muted-foreground">
                      {t('overview.coreDescription')}
                    </p>

                    <div className="p-3 rounded-lg bg-muted border font-mono text-xs flex items-center justify-between">
                      <span
                        className="text-foreground break-all"
                        data-testid="ping-result"
                      >
                        {loadingPing ? t('app.loading') : pingResult}
                      </span>
                      <Button
                        variant="ghost"
                        size="icon"
                        className="h-7 w-7 flex-shrink-0 ml-2"
                        onClick={fetchPing}
                        disabled={loadingPing}
                        aria-label="Refresh ping"
                      >
                        <RefreshCw
                          className={`h-3.5 w-3.5 ${loadingPing ? 'animate-spin' : ''}`}
                        />
                      </Button>
                    </div>

                    <div className="flex items-center gap-2 text-xs text-muted-foreground">
                      <span>{t('common.status')}:</span>
                      <Badge variant="default" className="text-[10px]">
                        {appInfo?.mode === 'tauri'
                          ? t('app.modeTauri')
                          : t('app.modeMock')}
                      </Badge>
                    </div>
                  </CardContent>
                </Card>

                {/* Reference Scale Card */}
                <Card>
                  <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                    <CardTitle className="text-sm font-medium">
                      {t('overview.scaleCard')}
                    </CardTitle>
                    <GraduationCap className="h-4 w-4 text-primary" />
                  </CardHeader>
                  <CardContent className="space-y-4 pt-2">
                    <p className="text-xs text-muted-foreground leading-relaxed">
                      {t('overview.scaleInfo')}
                    </p>
                    <div className="grid grid-cols-3 gap-2 pt-2 text-center">
                      <div className="p-2 rounded bg-muted/60 border">
                        <div className="text-lg font-bold">11</div>
                        <div className="text-[10px] text-muted-foreground">
                          {t('nav.teachers')}
                        </div>
                      </div>
                      <div className="p-2 rounded bg-muted/60 border">
                        <div className="text-lg font-bold">12</div>
                        <div className="text-[10px] text-muted-foreground">
                          {t('nav.exams')} (4×3)
                        </div>
                      </div>
                      <div className="p-2 rounded bg-muted/60 border">
                        <div className="text-lg font-bold">36</div>
                        <div className="text-[10px] text-muted-foreground">
                          {t('nav.assignments')}
                        </div>
                      </div>
                    </div>
                  </CardContent>
                </Card>
              </div>
            </div>
          )}

          {activeTab !== 'overview' && (
            <div className="max-w-5xl mx-auto space-y-6">
              <div>
                <h3 className="text-2xl font-bold tracking-tight">
                  {t(`${activeTab}.title`)}
                </h3>
                <p className="text-sm text-muted-foreground mt-1">
                  {t(`${activeTab}.description`)}
                </p>
              </div>

              <Card>
                <CardHeader>
                  <CardTitle className="text-base">{t(`${activeTab}.title`)}</CardTitle>
                  <CardDescription>{t('common.placeholderNotice')}</CardDescription>
                </CardHeader>
                <CardContent>
                  <div className="border border-dashed rounded-lg p-12 text-center text-muted-foreground space-y-3">
                    <p className="text-sm font-medium">{t('common.placeholderNotice')}</p>
                    <Badge variant="outline">Phase 1 Skeleton</Badge>
                  </div>
                </CardContent>
              </Card>
            </div>
          )}
        </main>
      </div>
    </div>
  )
}
