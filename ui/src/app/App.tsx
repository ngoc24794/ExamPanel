import * as React from 'react'
import {
  HashRouter,
  Routes,
  Route,
  NavLink,
  Navigate,
  useLocation,
} from 'react-router-dom'
import { QueryClientProvider } from '@tanstack/react-query'
import { useTranslation } from 'react-i18next'
import {
  LayoutDashboard,
  Users,
  Building2,
  Calendar,
  CalendarX,
  Sliders,
  ClipboardCheck,
  BarChart3,
  Settings,
  Terminal,
  GraduationCap,
  ChevronLeft,
  ChevronRight,
  AlertTriangle,
  BookOpen,
} from 'lucide-react'
import { useQueryClient } from '@tanstack/react-query'
import { queryClient, getErrorMessage } from '@/lib/query/query-client'
import { Toaster } from '@/components/ui/sonner'
import { Button } from '@/components/ui/button'
import { ThemeToggle } from '@/lib/theme/ThemeToggle'
import { LanguageSwitcher } from '@/components/LanguageSwitcher'
import { SchoolYearSelector } from '@/components/SchoolYearSelector'
import { FeasibilityIndicator } from '@/components/FeasibilityIndicator'
import { useSchoolYears, useAppInfo } from '@/lib/query/hooks'
import { api } from '@/lib/api'
import { toast } from 'sonner'
import { OverviewPage } from '@/pages/OverviewPage'
import { CampusesPage } from '@/pages/CampusesPage'
import { SubjectsPage } from '@/pages/SubjectsPage'
import { TeachersPage } from '@/pages/TeachersPage'
import { ExamsPage } from '@/pages/ExamsPage'
import { UnavailabilityPage } from '@/pages/UnavailabilityPage'
import { RulesPage } from '@/pages/RulesPage'
import { SettingsPage } from '@/pages/SettingsPage'
import { DevPage } from '@/pages/DevPage'
import { AssignmentsPage } from '@/pages/AssignmentsPage'
import { StatisticsPage } from '@/pages/StatisticsPage'
import { PrintPlanPage } from '@/pages/print/PrintPlanPage'
import { PrintNoticesPage } from '@/pages/print/PrintNoticesPage'

interface NavEntry {
  path: string
  labelKey: string
  icon: React.ReactNode
  devOnly?: boolean
}

const NAV_ENTRIES: NavEntry[] = [
  {
    path: '/',
    labelKey: 'nav.overview',
    icon: <LayoutDashboard className="h-4 w-4" />,
  },
  {
    path: '/teachers',
    labelKey: 'nav.teachers',
    icon: <Users className="h-4 w-4" />,
  },
  {
    path: '/campuses',
    labelKey: 'nav.campuses',
    icon: <Building2 className="h-4 w-4" />,
  },
  {
    path: '/subjects',
    labelKey: 'nav.subjects',
    icon: <BookOpen className="h-4 w-4" />,
  },
  {
    path: '/exams',
    labelKey: 'nav.exams',
    icon: <Calendar className="h-4 w-4" />,
  },
  {
    path: '/unavailability',
    labelKey: 'nav.unavailability',
    icon: <CalendarX className="h-4 w-4" />,
  },
  {
    path: '/rules',
    labelKey: 'nav.rules',
    icon: <Sliders className="h-4 w-4" />,
  },
  {
    path: '/assignments',
    labelKey: 'nav.assignments',
    icon: <ClipboardCheck className="h-4 w-4" />,
  },
  {
    path: '/statistics',
    labelKey: 'nav.stats',
    icon: <BarChart3 className="h-4 w-4" />,
  },
  {
    path: '/settings',
    labelKey: 'nav.settings',
    icon: <Settings className="h-4 w-4" />,
  },
  {
    path: '/dev',
    labelKey: 'nav.dev',
    icon: <Terminal className="h-4 w-4" />,
    devOnly: true,
  },
]

function AppLayout() {
  const { t } = useTranslation()
  const location = useLocation()
  const [collapsed, setCollapsed] = React.useState(false)
  const qc = useQueryClient()
  const { data: schoolYears = [] } = useSchoolYears()
  const { data: appInfo } = useAppInfo()
  const currentYear = schoolYears.find((y) => y.is_current) || schoolYears[0]

  const handleExitTrial = async () => {
    try {
      await api.exitTrialMode()
      await qc.invalidateQueries()
      window.dispatchEvent(new CustomEvent('exampanel:restore'))
      toast.success(t('trial.exited_toast'))
    } catch (err) {
      toast.error(getErrorMessage(err))
    }
  }

  const activeNav =
    NAV_ENTRIES.find((item) =>
      item.path === '/'
        ? location.pathname === '/'
        : location.pathname.startsWith(item.path),
    ) || NAV_ENTRIES[0]

  const isPrintRoute = location.pathname.startsWith('/print/')

  if (isPrintRoute) {
    return (
      <div className="min-h-screen bg-background text-foreground">
        <Routes>
          <Route path="/print/plan/:id" element={<PrintPlanPage />} />
          <Route path="/print/notices/:id" element={<PrintNoticesPage />} />
          <Route path="*" element={<Navigate to="/assignments" replace />} />
        </Routes>
      </div>
    )
  }

  return (
    <div className="flex h-screen w-screen overflow-hidden bg-background text-foreground min-w-[1024px] min-h-[700px]">
      {/* Collapsible Sidebar */}
      <aside
        className={`flex flex-col border-r border-border bg-card transition-all duration-200 select-none flex-shrink-0 ${
          collapsed ? 'w-16' : 'w-60'
        }`}
      >
        {/* Brand Header */}
        <div className="h-14 border-b border-border px-4 flex items-center justify-between">
          <div className="flex items-center gap-2.5 overflow-hidden">
            <div className="h-8 w-8 rounded-lg bg-primary flex items-center justify-center text-primary-foreground shrink-0 shadow-sm">
              <GraduationCap className="h-5 w-5" />
            </div>
            {!collapsed && (
              <div className="flex flex-col overflow-hidden">
                <span className="font-bold text-sm tracking-tight text-foreground truncate">
                  ExamPanel
                </span>
                <span className="text-[10px] text-muted-foreground truncate">
                  Phân công ra đề
                </span>
              </div>
            )}
          </div>
          <Button
            variant="ghost"
            size="icon"
            className="h-7 w-7 text-muted-foreground hover:text-foreground shrink-0"
            onClick={() => setCollapsed(!collapsed)}
            aria-label="Toggle sidebar"
          >
            {collapsed ? (
              <ChevronRight className="h-4 w-4" />
            ) : (
              <ChevronLeft className="h-4 w-4" />
            )}
          </Button>
        </div>

        {/* Navigation Items */}
        <nav className="flex-1 overflow-y-auto p-2 space-y-1">
          {NAV_ENTRIES.map((item) => {
            if (item.devOnly && !import.meta.env.DEV) return null

            const isActive =
              item.path === '/'
                ? location.pathname === '/'
                : location.pathname.startsWith(item.path)

            return (
              <NavLink
                key={item.path}
                to={item.path}
                className={`flex items-center gap-3 px-3 py-2 rounded-md text-xs font-medium transition-colors ${
                  isActive
                    ? 'bg-primary text-primary-foreground shadow-xs font-semibold'
                    : 'text-muted-foreground hover:bg-muted/80 hover:text-foreground'
                }`}
                title={collapsed ? t(item.labelKey) : undefined}
              >
                <span className="shrink-0">{item.icon}</span>
                {!collapsed && <span className="truncate">{t(item.labelKey)}</span>}
              </NavLink>
            )
          })}
        </nav>

        {/* Sidebar Footer */}
        <div className="p-3 border-t border-border/80 text-[11px] text-muted-foreground">
          {!collapsed ? (
            <div className="flex items-center justify-between">
              <span>ExamPanel</span>
              <span className="font-mono">v0.1.0</span>
            </div>
          ) : (
            <div className="text-center font-mono text-[10px]">v0.1</div>
          )}
        </div>
      </aside>

      {/* Main Container */}
      <div className="flex-1 flex flex-col overflow-hidden">
        {/* Top Header */}
        <header className="h-14 border-b border-border bg-card px-6 flex items-center justify-between flex-shrink-0">
          <div className="flex items-center gap-3">
            <h2 className="text-base font-semibold tracking-tight text-foreground">
              {t(activeNav.labelKey)}
            </h2>
          </div>

          <div className="flex items-center gap-3">
            <FeasibilityIndicator
              schoolYearId={currentYear?.id}
              schoolYearName={currentYear?.name}
            />
            <div className="h-4 w-px bg-border mx-0.5" />
            <SchoolYearSelector />
            <div className="h-4 w-px bg-border mx-0.5" />
            <LanguageSwitcher />
            <ThemeToggle />
          </div>
        </header>

        {/* Trial Mode Persistent Banner */}
        {appInfo?.in_trial_mode && (
          <div
            className="bg-amber-500/15 border-b border-amber-500/30 px-6 py-2 flex items-center justify-between text-xs text-amber-900 dark:text-amber-200 flex-shrink-0"
            data-testid="trial-mode-banner"
          >
            <div className="flex items-center gap-2">
              <AlertTriangle className="h-4 w-4 text-amber-600 dark:text-amber-400 shrink-0" />
              <span className="font-medium">{t('trial.banner_message')}</span>
            </div>
            <Button
              variant="outline"
              size="sm"
              className="h-7 text-xs border-amber-500/40 hover:bg-amber-500/20 text-foreground"
              onClick={handleExitTrial}
              data-testid="exit-trial-btn"
            >
              {t('trial.exit_button')}
            </Button>
          </div>
        )}

        {/* Content Viewport */}
        <main className="flex-1 overflow-y-auto p-6 bg-muted/20">
          <div className="max-w-6xl mx-auto w-full">
            <Routes>
              <Route path="/" element={<OverviewPage />} />
              <Route path="/teachers" element={<TeachersPage />} />
              <Route path="/campuses" element={<CampusesPage />} />
              <Route path="/subjects" element={<SubjectsPage />} />
              <Route path="/exams" element={<ExamsPage />} />
              <Route path="/unavailability" element={<UnavailabilityPage />} />
              <Route path="/rules" element={<RulesPage />} />
              <Route path="/assignments" element={<AssignmentsPage />} />
              <Route path="/statistics" element={<StatisticsPage />} />
              <Route path="/stats" element={<Navigate to="/statistics" replace />} />
              <Route path="/settings" element={<SettingsPage />} />
              {import.meta.env.DEV && <Route path="/dev" element={<DevPage />} />}
              <Route path="*" element={<Navigate to="/" replace />} />
            </Routes>
          </div>
        </main>
      </div>

      <Toaster />
    </div>
  )
}

export function App() {
  return (
    <QueryClientProvider client={queryClient}>
      <HashRouter>
        <AppLayout />
      </HashRouter>
    </QueryClientProvider>
  )
}
