import * as React from 'react'
import { useTranslation } from 'react-i18next'
import { useNavigate } from 'react-router-dom'
import {
  AlertOctagon,
  AlertTriangle,
  ArrowRight,
  CheckCircle2,
  Database,
  Grid,
  Lock as LockIcon,
  Scale,
} from 'lucide-react'
import {
  Sheet,
  SheetContent,
  SheetHeader,
  SheetTitle,
  SheetDescription,
} from '@/components/ui/sheet'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import type { Diagnostic, FeasibilityReport, Violation } from '@/lib/api'

interface FeasibilitySheetProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  report: FeasibilityReport | undefined
  schoolYearName?: string
}

type CategoryKey = 'data' | 'panels' | 'locks' | 'capacity'

function categorizeDiagnostic(code: string): CategoryKey {
  if (
    code.startsWith('no_') ||
    code.startsWith('duplicate_') ||
    code.startsWith('unknown_') ||
    code === 'single_campus' ||
    code === 'load_weight_out_of_range' ||
    code === 'negative_rule_weight'
  ) {
    return 'data'
  }
  if (code.includes('lock') || code.includes('pin')) {
    return 'locks'
  }
  if (code.includes('capacity') || code.includes('quota') || code.includes('flow')) {
    return 'capacity'
  }
  return 'panels'
}

function getFixTarget(violation: Diagnostic | Violation): { path: string; labelKey: string } {
  const code = violation.code
  if (
    code === 'no_campuses' ||
    code === 'single_campus' ||
    code.startsWith('duplicate_campus')
  ) {
    return { path: '/campuses', labelKey: 'nav.campuses' }
  }
  if (
    code === 'no_grades' ||
    code === 'no_exams' ||
    code.startsWith('duplicate_grade') ||
    code.startsWith('duplicate_exam')
  ) {
    return { path: '/exams', labelKey: 'nav.exams' }
  }
  if (code.includes('lock') || code.includes('pin')) {
    return { path: '/rules?tab=locks', labelKey: 'rules.tabLocks' }
  }
  if (code.includes('unavailability') || code.includes('tight_panel_roster')) {
    return { path: '/unavailability', labelKey: 'nav.unavailability' }
  }
  if (code.includes('capacity') || code.includes('quota')) {
    return { path: '/rules?tab=quotas', labelKey: 'rules.tabQuotas' }
  }
  return { path: '/teachers', labelKey: 'nav.teachers' }
}

const CATEGORY_ICONS: Record<CategoryKey, React.ReactNode> = {
  data: <Database className="h-4 w-4" />,
  panels: <Grid className="h-4 w-4" />,
  locks: <LockIcon className="h-4 w-4" />,
  capacity: <Scale className="h-4 w-4" />,
}

export const FeasibilitySheet: React.FC<FeasibilitySheetProps> = ({
  open,
  onOpenChange,
  report,
  schoolYearName,
}) => {
  const { t } = useTranslation()
  const navigate = useNavigate()

  const errors = report?.errors ?? []
  const warnings = report?.warnings ?? []
  const isFeasible = report?.is_feasible ?? true

  const categories: CategoryKey[] = ['data', 'panels', 'locks', 'capacity']

  const handleGoToFix = (path: string) => {
    onOpenChange(false)
    navigate(path)
  }

  return (
    <Sheet open={open} onOpenChange={onOpenChange}>
      <SheetContent
        side="right"
        className="w-full sm:max-w-lg overflow-y-auto flex flex-col p-6 gap-6 bg-card text-card-foreground border-border"
        data-testid="feasibility-sheet"
      >
        <SheetHeader className="space-y-2 text-left">
          <div className="flex items-center justify-between">
            <SheetTitle className="text-lg font-bold text-foreground">
              {t('feasibility.sheetTitle')}
            </SheetTitle>
            <Badge
              variant={
                isFeasible
                  ? warnings.length > 0
                    ? 'secondary'
                    : 'default'
                  : 'destructive'
              }
              className="text-xs uppercase tracking-wider"
              data-testid="feasibility-status-badge"
            >
              {isFeasible
                ? warnings.length > 0
                  ? t('feasibility.statusWarnings')
                  : t('feasibility.statusFeasible')
                : t('feasibility.statusErrors')}
            </Badge>
          </div>
          <SheetDescription className="text-xs text-muted-foreground">
            {t('feasibility.sheetDescription')} {schoolYearName && `(${schoolYearName})`}
          </SheetDescription>
        </SheetHeader>

        {/* Global Summary */}
        <div className="flex items-center gap-3 p-3 rounded-lg border border-border bg-muted/30">
          {errors.length > 0 ? (
            <AlertOctagon className="h-6 w-6 text-destructive shrink-0" />
          ) : warnings.length > 0 ? (
            <AlertTriangle className="h-6 w-6 text-amber-500 shrink-0" />
          ) : (
            <CheckCircle2 className="h-6 w-6 text-emerald-500 shrink-0" />
          )}
          <div className="flex-1 text-xs">
            <div className="font-semibold text-foreground">
              {errors.length > 0
                ? t('feasibility.errorCount', { count: errors.length })
                : warnings.length > 0
                  ? t('feasibility.warningCount', { count: warnings.length })
                  : t('feasibility.allGood')}
            </div>
            {warnings.length > 0 && errors.length > 0 && (
              <div className="text-muted-foreground">
                {t('feasibility.warningCount', { count: warnings.length })}
              </div>
            )}
          </div>
        </div>

        {/* Grouped Diagnostics */}
        <div className="flex-1 space-y-6">
          {categories.map((catKey) => {
            const catErrors = errors.filter(
              (e) => categorizeDiagnostic(e.code) === catKey,
            )
            const catWarnings = warnings.filter(
              (w) => categorizeDiagnostic(w.code) === catKey,
            )
            const totalItems = catErrors.length + catWarnings.length

            if (totalItems === 0) return null

            const categoryTitleKey =
              catKey === 'data'
                ? 'feasibility.categoryData'
                : catKey === 'panels'
                  ? 'feasibility.categoryPanels'
                  : catKey === 'locks'
                    ? 'feasibility.categoryLocks'
                    : 'feasibility.categoryCapacity'

            return (
              <div
                key={catKey}
                className="space-y-3"
                data-testid={`feasibility-cat-${catKey}`}
              >
                <div className="flex items-center gap-2 text-xs font-semibold uppercase tracking-wider text-muted-foreground border-b border-border pb-1">
                  {CATEGORY_ICONS[catKey]}
                  <span>{t(categoryTitleKey)}</span>
                  <Badge variant="outline" className="ml-auto text-[10px]">
                    {totalItems}
                  </Badge>
                </div>

                <div className="space-y-2">
                  {catErrors.map((err, idx) => {
                    const target = getFixTarget(err)
                    const message = t(`diagnostics.${err.code}`, {
                      ...err.params,
                      defaultValue: `[${err.code}] ${JSON.stringify(err.params)}`,
                    })

                    return (
                      <div
                        key={`err-${idx}`}
                        className="p-3 rounded-md border border-destructive/30 bg-destructive/5 space-y-2"
                        data-testid="feasibility-error-item"
                      >
                        <div className="flex items-start gap-2">
                          <AlertOctagon className="h-4 w-4 text-destructive shrink-0 mt-0.5" />
                          <span className="text-xs text-foreground font-medium flex-1">
                            {message}
                          </span>
                        </div>
                        <div className="flex justify-end pt-1">
                          <Button
                            variant="ghost"
                            size="sm"
                            className="h-7 text-xs text-destructive hover:bg-destructive/10 gap-1 px-2"
                            onClick={() => handleGoToFix(target.path)}
                            data-testid="go-to-fix-btn"
                          >
                            <span>{t('feasibility.goToFix')}</span>
                            <ArrowRight className="h-3 w-3" />
                          </Button>
                        </div>
                      </div>
                    )
                  })}

                  {catWarnings.map((warn, idx) => {
                    const target = getFixTarget(warn)
                    const message = t(`diagnostics.${warn.code}`, {
                      ...warn.params,
                      defaultValue: `[${warn.code}] ${JSON.stringify(warn.params)}`,
                    })

                    return (
                      <div
                        key={`warn-${idx}`}
                        className="p-3 rounded-md border border-amber-500/30 bg-amber-500/5 space-y-2"
                        data-testid="feasibility-warning-item"
                      >
                        <div className="flex items-start gap-2">
                          <AlertTriangle className="h-4 w-4 text-amber-500 shrink-0 mt-0.5" />
                          <span className="text-xs text-foreground font-medium flex-1">
                            {message}
                          </span>
                        </div>
                        <div className="flex justify-end pt-1">
                          <Button
                            variant="ghost"
                            size="sm"
                            className="h-7 text-xs text-amber-600 dark:text-amber-400 hover:bg-amber-500/10 gap-1 px-2"
                            onClick={() => handleGoToFix(target.path)}
                            data-testid="go-to-fix-btn"
                          >
                            <span>{t('feasibility.goToFix')}</span>
                            <ArrowRight className="h-3 w-3" />
                          </Button>
                        </div>
                      </div>
                    )
                  })}
                </div>
              </div>
            )
          })}

          {errors.length === 0 && warnings.length === 0 && (
            <div className="p-8 text-center space-y-2 text-muted-foreground">
              <CheckCircle2 className="h-10 w-10 text-emerald-500 mx-auto" />
              <p className="text-sm font-medium text-foreground">
                {t('feasibility.allGood')}
              </p>
            </div>
          )}
        </div>
      </SheetContent>
    </Sheet>
  )
}
