import * as React from 'react'
import { useTranslation } from 'react-i18next'
import {
  Activity,
  RefreshCw,
  Building2,
  Users,
  ShieldCheck,
  ShieldAlert,
} from 'lucide-react'
import {
  Card,
  CardHeader,
  CardTitle,
  CardDescription,
  CardContent,
} from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import { Onboarding } from '@/components/Onboarding'
import { api } from '@/lib/api'
import {
  useAppInfo,
  useCampuses,
  useSchoolYears,
  useTeachers,
  useFeasibility,
  useSubjects,
  useCompetencies,
} from '@/lib/query/hooks'

export const OverviewPage: React.FC = () => {
  const { t } = useTranslation()
  const { data: appInfo } = useAppInfo()
  const { data: campuses = [] } = useCampuses()
  const { data: schoolYears = [] } = useSchoolYears()
  const currentYear = schoolYears.find((y) => y.is_current) || schoolYears[0]
  const { data: teachersWithGrades = [] } = useTeachers(currentYear?.id)
  const { data: subjects = [] } = useSubjects(currentYear?.id)
  const { data: competencies = [] } = useCompetencies(currentYear?.id)
  const { data: feasibility } = useFeasibility(currentYear?.id)

  const [pingResult, setPingResult] = React.useState<string>('')
  const [loadingPing, setLoadingPing] = React.useState<boolean>(true)

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
  }, [fetchPing])

  const showOnboarding =
    campuses.length === 0 || teachersWithGrades.length === 0 || subjects.length === 0

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-2xl font-bold tracking-tight text-foreground">
          {t('overview.title')}
        </h1>
        <p className="text-sm text-muted-foreground mt-1">{t('overview.description')}</p>
      </div>

      {/* Onboarding Checklist if no campuses or teachers or subjects */}
      {showOnboarding && (
        <Onboarding
          campusesCount={campuses.length}
          teachersCount={teachersWithGrades.length}
          subjectsCount={subjects.length}
          competenciesCount={competencies.length}
        />
      )}

      {/* Summary KPI Cards */}
      <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
        <Card className="bg-card">
          <CardHeader className="flex flex-row items-center justify-between pb-2">
            <CardTitle className="text-sm font-medium text-foreground">
              {t('campuses.title')}
            </CardTitle>
            <Building2 className="h-4 w-4 text-primary" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold text-foreground">{campuses.length}</div>
            <p className="text-xs text-muted-foreground mt-1">
              {t('campuses.description')}
            </p>
          </CardContent>
        </Card>

        <Card className="bg-card">
          <CardHeader className="flex flex-row items-center justify-between pb-2">
            <CardTitle className="text-sm font-medium text-foreground">
              {t('teachers.title')}
            </CardTitle>
            <Users className="h-4 w-4 text-primary" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold text-foreground">
              {teachersWithGrades.length}
            </div>
            <p className="text-xs text-muted-foreground mt-1">
              {currentYear ? t('common.schoolYearName', { name: currentYear.name }) : ''}
            </p>
          </CardContent>
        </Card>

        <Card className="bg-card">
          <CardHeader className="flex flex-row items-center justify-between pb-2">
            <CardTitle className="text-sm font-medium text-foreground">
              {t('dev.checkFeasibility')}
            </CardTitle>
            {feasibility?.report.is_feasible ? (
              <ShieldCheck className="h-4 w-4 text-emerald-500" />
            ) : (
              <ShieldAlert className="h-4 w-4 text-destructive" />
            )}
          </CardHeader>
          <CardContent>
            <div className="text-lg font-bold text-foreground">
              {feasibility?.report.is_feasible ? (
                <span className="text-emerald-600 dark:text-emerald-400">
                  {t('dev.feasibilityFeasible')}
                </span>
              ) : (
                <span className="text-destructive">{t('dev.feasibilityInfeasible')}</span>
              )}
            </div>
            <p className="text-xs text-muted-foreground mt-1">
              {t('overview.issuesSummary', { errors: feasibility?.report.errors.length ?? 0, warnings: feasibility?.report.warnings.length ?? 0 })}
            </p>
          </CardContent>
        </Card>
      </div>

      {/* Connection & Architecture Cards */}
      <div className="grid gap-6 md:grid-cols-2">
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium text-foreground">
              {t('overview.coreConnectionCard')}
            </CardTitle>
            <Activity className="h-4 w-4 text-primary" />
          </CardHeader>
          <CardContent className="space-y-4 pt-2">
            <p className="text-xs text-muted-foreground">
              {t('overview.coreDescription')}
            </p>

            <div className="p-3 rounded-lg bg-muted border font-mono text-xs flex items-center justify-between">
              <span className="text-foreground break-all" data-testid="ping-result">
                {loadingPing ? t('app.loading') : pingResult}
              </span>
              <Button
                variant="ghost"
                size="icon"
                className="h-7 w-7 flex-shrink-0 ml-2 text-muted-foreground hover:text-foreground"
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
                {appInfo?.mode === 'tauri' ? t('app.modeTauri') : t('app.modeMock')}
              </Badge>
              <span className="text-emerald-500 font-medium ml-auto">
                ● {t('app.ready')}
              </span>
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm font-medium text-foreground">
              {t('overview.scaleCard')}
            </CardTitle>
            <CardDescription className="text-xs text-muted-foreground">
              {t('overview.scaleInfo')}
            </CardDescription>
          </CardHeader>
          <CardContent className="space-y-2 text-xs text-muted-foreground pt-2">
            <div className="flex justify-between py-1 border-b border-border/50">
              <span>{t('common.version')}:</span>
              <span className="font-mono text-foreground font-semibold">
                {appInfo?.version || '0.1.0'}
              </span>
            </div>
            <div className="flex justify-between py-1 border-b border-border/50">
              <span>{t('teachers.campus')}:</span>
              <span className="font-medium text-foreground">
                {t('overview.campusesCount', { count: campuses.length })}
              </span>
            </div>
            <div className="flex justify-between py-1">
              <span>{t('nav.teachers')}:</span>
              <span className="font-medium text-foreground">
                {t('overview.teachersCount', { count: teachersWithGrades.length })}
              </span>
            </div>
          </CardContent>
        </Card>
      </div>
    </div>
  )
}
