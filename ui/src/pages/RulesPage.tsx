import * as React from 'react'
import { useTranslation } from 'react-i18next'
import { useSearchParams } from 'react-router-dom'
import {
  AlertOctagon,
  AlertTriangle,
  Check,
  CheckCircle2,
  Lock as LockIcon,
  Plus,
  RotateCcw,
  Save,
  Scale,
  ShieldAlert,
  ShieldCheck,
  Sliders,
  Sparkles,
  Trash2,
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
import { Switch } from '@/components/ui/switch'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogFooter,
} from '@/components/ui/dialog'
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from '@/components/ui/alert-dialog'
import {
  useExams,
  useGrades,
  useLocks,
  useRulePresets,
  useRuleSettings,
  useSchoolYears,
  useSubjects,
  useTeachers,
  useUnavailabilities,
  useSaveRuleSettings,
  useResetRuleSettings,
  useCreateLock,
  useDeleteLock,
  useFeasibility,
} from '@/lib/query/hooks'
import {
  api,
  type Lock,
  type LockKind,
  type Role,
  type RuleSetting,
  type QuotaPreviewItem,
} from '@/lib/api'
import { toast } from 'sonner'
import { getErrorMessage } from '@/lib/query/query-client'

type TabKey = 'hard' | 'soft' | 'quotas' | 'locks'

export const RulesPage: React.FC = () => {
  const { t } = useTranslation()
  const [searchParams, setSearchParams] = useSearchParams()
  const initialTab = (searchParams.get('tab') as TabKey) || 'hard'
  const [activeTab, setActiveTab] = React.useState<TabKey>(initialTab)

  const { data: schoolYears = [] } = useSchoolYears()
  const currentYear = schoolYears.find((y) => y.is_current) || schoolYears[0]

  const { data: grades = [] } = useGrades()
  const { data: exams = [] } = useExams(currentYear?.id)
  const { data: subjects = [] } = useSubjects(currentYear?.id)
  const { data: teachersWithGrades = [] } = useTeachers(currentYear?.id)
  const { data: unavailabilities = [] } = useUnavailabilities(currentYear?.id)
  const { data: serverRuleSettings = [] } = useRuleSettings(currentYear?.id)
  const { data: rulePresets = [] } = useRulePresets()
  const { data: locks = [] } = useLocks(currentYear?.id)
  const { data: feasibility } = useFeasibility(currentYear?.id)

  const saveSettingsMutation = useSaveRuleSettings(currentYear?.id ?? 0)
  const resetSettingsMutation = useResetRuleSettings(currentYear?.id ?? 0)
  const createLockMutation = useCreateLock(currentYear?.id ?? 0)
  const deleteLockMutation = useDeleteLock(currentYear?.id ?? 0)

  // Local draft rule settings for live preview and unsaved changes tracking
  const [draftSettings, setDraftSettings] = React.useState<RuleSetting[]>([])
  const [isDirty, setIsDirty] = React.useState(false)

  // Sync draft settings with server settings on load
  React.useEffect(() => {
    if (serverRuleSettings.length > 0 && !isDirty) {
      setDraftSettings(JSON.parse(JSON.stringify(serverRuleSettings)))
    }
  }, [serverRuleSettings, isDirty])

  // Sync tab with URL search parameter
  React.useEffect(() => {
    const tabParam = searchParams.get('tab') as TabKey
    if (tabParam && tabParam !== activeTab) {
      setActiveTab(tabParam)
    }
  }, [searchParams, activeTab])

  const handleTabChange = (tab: TabKey) => {
    setActiveTab(tab)
    setSearchParams({ tab })
  }

  // Unsaved changes window listener
  React.useEffect(() => {
    const handleBeforeUnload = (e: BeforeUnloadEvent) => {
      if (isDirty) {
        e.preventDefault()
        e.returnValue = ''
      }
    }
    window.addEventListener('beforeunload', handleBeforeUnload)
    return () => window.removeEventListener('beforeunload', handleBeforeUnload)
  }, [isDirty])

  // Helper to get or update a rule setting in draft
  const getRule = (key: string): RuleSetting | undefined => {
    return draftSettings.find((s) => s.key === key)
  }

  const updateRule = (key: string, updates: Partial<RuleSetting>) => {
    setIsDirty(true)
    setDraftSettings((prev) => {
      const idx = prev.findIndex((s) => s.key === key)
      if (idx === -1) return prev
      const copy = [...prev]
      copy[idx] = { ...copy[idx], ...updates }
      return copy
    })
  }

  const updateH7Tolerance = (tolerance: number) => {
    setIsDirty(true)
    setDraftSettings((prev) => {
      const idx = prev.findIndex((s) => s.key === 'h7')
      if (idx === -1) return prev
      const copy = [...prev]
      copy[idx] = {
        ...copy[idx],
        params: { ...(copy[idx].params || {}), tolerance },
      }
      return copy
    })
  }

  // Apply Preset
  const handleApplyPreset = (presetId: string) => {
    const preset = rulePresets.find((p) => p.id === presetId)
    if (!preset) return
    setIsDirty(true)
    setDraftSettings((prev) => {
      const copy = JSON.parse(JSON.stringify(prev)) as RuleSetting[]
      for (const newSetting of preset.settings) {
        const idx = copy.findIndex((s) => s.key === newSetting.key)
        if (idx !== -1) {
          copy[idx].weight = newSetting.weight
          copy[idx].enabled = newSetting.enabled
        }
      }
      return copy
    })
    toast.success(`${t('rules.applyPreset')}: ${preset.name}`)
  }

  // Reset to Defaults Dialog
  const [resetConfirmOpen, setResetConfirmOpen] = React.useState(false)
  const handleConfirmReset = async () => {
    try {
      await resetSettingsMutation.mutateAsync()
      setIsDirty(false)
      setResetConfirmOpen(false)
      toast.success(t('rules.resetDefaults'))
    } catch (err) {
      toast.error(getErrorMessage(err))
    }
  }

  // Save Settings
  const handleSaveSettings = async () => {
    if (!currentYear) return
    try {
      await saveSettingsMutation.mutateAsync(draftSettings)
      setIsDirty(false)
    } catch (err) {
      toast.error(getErrorMessage(err))
    }
  }

  // Live Quota Preview Calculation
  const [quotaPreview, setQuotaPreview] = React.useState<QuotaPreviewItem[]>([])
  React.useEffect(() => {
    if (!currentYear || draftSettings.length === 0) return
    let active = true

    api
      .previewQuotas({
        school_year_id: currentYear.id,
        rule_settings: draftSettings,
      })
      .then((items) => {
        if (active) setQuotaPreview(items)
      })
      .catch((err) => {
        console.error('Failed to preview quotas:', err)
      })

    return () => {
      active = false
    }
  }, [currentYear, draftSettings])

  // Lock Management State
  const [lockDialogOpen, setLockDialogOpen] = React.useState(false)
  const [lockExamId, setLockExamId] = React.useState<string>('')
  const [lockGradeId, setLockGradeId] = React.useState<string>('')
  const [lockSubjectId, setLockSubjectId] = React.useState<string>('')
  const [lockTeacherId, setLockTeacherId] = React.useState<string>('')
  const [lockRole, setLockRole] = React.useState<string>('any')
  const [lockKind, setLockKind] = React.useState<LockKind>('pin')
  const [deletingLock, setDeletingLock] = React.useState<Lock | null>(null)

  const handleOpenCreateLock = () => {
    setLockExamId(exams[0]?.id.toString() || '')
    setLockGradeId(grades[0]?.id.toString() || '')
    setLockSubjectId(subjects[0]?.id.toString() || '')
    setLockTeacherId('')
    setLockRole('any')
    setLockKind('pin')
    setLockDialogOpen(true)
  }

  const handleSaveLock = async (e: React.FormEvent) => {
    e.preventDefault()
    if (!lockExamId || !lockGradeId || !lockTeacherId) return

    try {
      await createLockMutation.mutateAsync({
        exam_id: Number(lockExamId),
        grade_id: Number(lockGradeId),
        subject_id: Number(lockSubjectId) || subjects[0]?.id || 1,
        teacher_id: Number(lockTeacherId),
        role: lockRole === 'any' ? null : (lockRole as Role),
        kind: lockKind,
      })
      setLockDialogOpen(false)
    } catch (err) {
      toast.error(getErrorMessage(err))
    }
  }

  const handleConfirmDeleteLock = async () => {
    if (!deletingLock) return
    try {
      await deleteLockMutation.mutateAsync(deletingLock.id)
      setDeletingLock(null)
    } catch (err) {
      toast.error(getErrorMessage(err))
    }
  }

  // Teacher eligibility check for selected lock exam & grade
  const teacherEligibilityList = React.useMemo(() => {
    const examId = Number(lockExamId)
    const gradeId = Number(lockGradeId)
    if (!examId || !gradeId) return []

    return teachersWithGrades.map(({ teacher, grade_ids }) => {
      let isEligible = true
      let reason = ''

      if (!teacher.active || teacher.load_weight <= 0) {
        isEligible = false
        reason = t('locks.reasonInactive')
      } else if (!grade_ids.includes(gradeId)) {
        isEligible = false
        reason = t('locks.reasonNotQualified')
      } else {
        const isUnavailable = unavailabilities.some(
          (u) => u.teacher_id === teacher.id && u.exam_id === examId,
        )
        if (isUnavailable) {
          isEligible = false
          reason = t('locks.reasonUnavailable')
        }
      }

      return { teacher, isEligible, reason }
    })
  }, [teachersWithGrades, unavailabilities, lockExamId, lockGradeId, t])

  // Live lock validation helper from feasibility report
  const getLockValidationDiagnostic = (lock: Lock) => {
    const report = feasibility?.report
    if (!report) return null

    const match = report.errors.find(
      (e) =>
        e.panel &&
        e.panel.exam_id === lock.exam_id &&
        e.panel.grade_id === lock.grade_id &&
        (e.teacher === lock.teacher_id ||
          e.code.includes('excess_pinned') ||
          e.code.includes('monopoly')),
    )

    if (match) {
      return {
        isError: true,
        message: t(`diagnostics.${match.code}`, {
          ...match.params,
          defaultValue: match.code,
        }),
      }
    }
    return null
  }

  // Current H7 tolerance
  const h7Rule = getRule('h7')
  const currentTolerance =
    typeof h7Rule?.params?.tolerance === 'number'
      ? (h7Rule.params.tolerance as number)
      : 1

  return (
    <div className="space-y-6" data-testid="rules-page">
      <div className="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4">
        <div>
          <h1 className="text-2xl font-bold tracking-tight text-foreground">
            {t('rules.title')}
          </h1>
          <p className="text-sm text-muted-foreground mt-1">{t('rules.description')}</p>
        </div>

        {/* Global Action Bar */}
        <div className="flex items-center gap-2">
          {isDirty && (
            <Badge
              variant="outline"
              className="text-xs text-amber-500 border-amber-500/30"
            >
              Chưa lưu thay đổi
            </Badge>
          )}
          <Button
            variant="outline"
            size="sm"
            onClick={() => setResetConfirmOpen(true)}
            className="gap-1.5 h-8 text-xs text-muted-foreground hover:text-foreground"
            data-testid="reset-rules-btn"
          >
            <RotateCcw className="h-3.5 w-3.5" />
            <span>{t('rules.resetDefaults')}</span>
          </Button>
          <Button
            size="sm"
            onClick={handleSaveSettings}
            disabled={!isDirty || saveSettingsMutation.isPending}
            className="gap-1.5 h-8 text-xs"
            data-testid="save-rules-btn"
          >
            <Save className="h-3.5 w-3.5" />
            <span>{t('rules.saveRules')}</span>
          </Button>
        </div>
      </div>

      {/* Navigation Tabs */}
      <div className="flex items-center border-b border-border gap-2 text-xs font-medium">
        <button
          type="button"
          onClick={() => handleTabChange('hard')}
          className={`pb-2.5 px-3 border-b-2 font-semibold transition-colors flex items-center gap-2 ${
            activeTab === 'hard'
              ? 'border-primary text-primary'
              : 'border-transparent text-muted-foreground hover:text-foreground'
          }`}
          data-testid="tab-hard-rules"
        >
          <ShieldAlert className="h-4 w-4" />
          <span>{t('rules.tabHard')}</span>
        </button>

        <button
          type="button"
          onClick={() => handleTabChange('soft')}
          className={`pb-2.5 px-3 border-b-2 font-semibold transition-colors flex items-center gap-2 ${
            activeTab === 'soft'
              ? 'border-primary text-primary'
              : 'border-transparent text-muted-foreground hover:text-foreground'
          }`}
          data-testid="tab-soft-rules"
        >
          <Sliders className="h-4 w-4" />
          <span>{t('rules.tabSoft')}</span>
        </button>

        <button
          type="button"
          onClick={() => handleTabChange('quotas')}
          className={`pb-2.5 px-3 border-b-2 font-semibold transition-colors flex items-center gap-2 ${
            activeTab === 'quotas'
              ? 'border-primary text-primary'
              : 'border-transparent text-muted-foreground hover:text-foreground'
          }`}
          data-testid="tab-quotas"
        >
          <Scale className="h-4 w-4" />
          <span>{t('rules.tabQuotas')}</span>
        </button>

        <button
          type="button"
          onClick={() => handleTabChange('locks')}
          className={`pb-2.5 px-3 border-b-2 font-semibold transition-colors flex items-center gap-2 ${
            activeTab === 'locks'
              ? 'border-primary text-primary'
              : 'border-transparent text-muted-foreground hover:text-foreground'
          }`}
          data-testid="tab-locks"
        >
          <LockIcon className="h-4 w-4" />
          <span>{t('rules.tabLocks')}</span>
          {locks.length > 0 && (
            <Badge variant="secondary" className="h-4 px-1 text-[10px]">
              {locks.length}
            </Badge>
          )}
        </button>
      </div>

      {/* TAB 1: HARD RULES (H1-H7) */}
      {activeTab === 'hard' && (
        <div className="space-y-4" data-testid="hard-rules-section">
          {/* Always-on Hard Rules: H1, H2, H3, H5, H6 */}
          <div className="grid gap-4 md:grid-cols-2">
            {[
              { id: 'h1', titleKey: 'rules.h1Title', descKey: 'rules.h1Desc' },
              { id: 'h2', titleKey: 'rules.h2Title', descKey: 'rules.h2Desc' },
              { id: 'h3', titleKey: 'rules.h3Title', descKey: 'rules.h3Desc' },
              { id: 'h5', titleKey: 'rules.h5Title', descKey: 'rules.h5Desc' },
              { id: 'h6', titleKey: 'rules.h6Title', descKey: 'rules.h6Desc' },
            ].map(({ id, titleKey, descKey }) => (
              <Card key={id} className="bg-card border-border">
                <CardHeader className="p-4 pb-2 flex flex-row items-center justify-between">
                  <CardTitle className="text-sm font-semibold text-foreground flex items-center gap-2">
                    <ShieldCheck className="h-4 w-4 text-emerald-500" />
                    <span>{t(titleKey)}</span>
                  </CardTitle>
                  <Badge
                    variant="outline"
                    className="text-[10px] text-emerald-600 dark:text-emerald-400 bg-emerald-500/10 border-emerald-500/30"
                  >
                    {t('rules.alwaysOn')}
                  </Badge>
                </CardHeader>
                <CardContent className="p-4 pt-1 text-xs text-muted-foreground">
                  {t(descKey)}
                </CardContent>
              </Card>
            ))}

            {/* H4 Toggle */}
            <Card className="bg-card border-border">
              <CardHeader className="p-4 pb-2 flex flex-row items-center justify-between">
                <CardTitle className="text-sm font-semibold text-foreground flex items-center gap-2">
                  <ShieldAlert className="h-4 w-4 text-primary" />
                  <span>{t('rules.h4Title')}</span>
                </CardTitle>
                <Switch
                  checked={getRule('h4')?.enabled ?? true}
                  onCheckedChange={(checked) => updateRule('h4', { enabled: checked })}
                  data-testid="toggle-h4"
                />
              </CardHeader>
              <CardContent className="p-4 pt-1 space-y-2 text-xs">
                <p className="text-muted-foreground">{t('rules.h4Desc')}</p>
                {!(getRule('h4')?.enabled ?? true) && (
                  <div
                    className="p-2.5 rounded-md border border-amber-500/30 bg-amber-500/10 text-amber-700 dark:text-amber-400 flex items-start gap-2"
                    data-testid="h4-warning-box"
                  >
                    <AlertTriangle className="h-4 w-4 shrink-0 mt-0.5" />
                    <span>{t('rules.h4Warning')}</span>
                  </div>
                )}
              </CardContent>
            </Card>

            {/* H7 Tolerance Select */}
            <Card className="bg-card border-border md:col-span-2">
              <CardHeader className="p-4 pb-2 flex flex-row items-center justify-between">
                <CardTitle className="text-sm font-semibold text-foreground flex items-center gap-2">
                  <Scale className="h-4 w-4 text-primary" />
                  <span>{t('rules.h7Title')}</span>
                </CardTitle>
                <Badge
                  variant="outline"
                  className="text-[10px] text-emerald-600 dark:text-emerald-400 bg-emerald-500/10 border-emerald-500/30"
                >
                  {t('rules.alwaysOn')}
                </Badge>
              </CardHeader>
              <CardContent className="p-4 pt-1 space-y-3 text-xs">
                <p className="text-muted-foreground">{t('rules.h7Desc')}</p>
                <div className="flex flex-col sm:flex-row sm:items-center gap-3 pt-1">
                  <label className="font-semibold text-foreground">
                    {t('rules.h7Tolerance')}:
                  </label>
                  <Select
                    value={currentTolerance.toString()}
                    onValueChange={(val) => updateH7Tolerance(Number(val))}
                  >
                    <SelectTrigger
                      className="w-[280px] text-xs bg-background"
                      data-testid="select-h7-tolerance"
                    >
                      <SelectValue />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="0">{t('rules.h7Tolerance0')}</SelectItem>
                      <SelectItem value="1">{t('rules.h7Tolerance1')}</SelectItem>
                      <SelectItem value="2">{t('rules.h7Tolerance2')}</SelectItem>
                    </SelectContent>
                  </Select>
                </div>
              </CardContent>
            </Card>
          </div>
        </div>
      )}

      {/* TAB 2: SOFT RULES (S1-S8) & PRESETS */}
      {activeTab === 'soft' && (
        <div className="space-y-6" data-testid="soft-rules-section">
          {/* Preset Buttons */}
          <Card className="bg-card border-border">
            <CardContent className="p-4 flex flex-wrap items-center justify-between gap-3">
              <div className="flex items-center gap-2">
                <Sparkles className="h-4 w-4 text-primary" />
                <span className="text-xs font-semibold text-foreground">
                  {t('rules.presetsTitle')}:
                </span>
              </div>
              <div className="flex flex-wrap items-center gap-2">
                <Button
                  variant="outline"
                  size="sm"
                  onClick={() => handleApplyPreset('balanced')}
                  className="h-8 text-xs gap-1.5"
                  data-testid="preset-balanced-btn"
                >
                  <Check className="h-3 w-3" />
                  <span>{t('rules.presetBalanced')}</span>
                </Button>
                <Button
                  variant="outline"
                  size="sm"
                  onClick={() => handleApplyPreset('workload_fairness')}
                  className="h-8 text-xs gap-1.5"
                  data-testid="preset-workload-btn"
                >
                  <Scale className="h-3 w-3 text-primary" />
                  <span>{t('rules.presetWorkloadFairness')}</span>
                </Button>
                <Button
                  variant="outline"
                  size="sm"
                  onClick={() => handleApplyPreset('team_diversity')}
                  className="h-8 text-xs gap-1.5"
                  data-testid="preset-diversity-btn"
                >
                  <Sparkles className="h-3 w-3 text-primary" />
                  <span>{t('rules.presetTeamDiversity')}</span>
                </Button>
              </div>
            </CardContent>
          </Card>

          {/* S1-S8 Sliders and Cards */}
          <div className="grid gap-4 md:grid-cols-2">
            {[
              {
                key: 's1',
                titleKey: 'rules.s1Title',
                descKey: 'rules.s1Desc',
                exKey: 'rules.s1Example',
              },
              {
                key: 's2',
                titleKey: 'rules.s2Title',
                descKey: 'rules.s2Desc',
                exKey: 'rules.s2Example',
              },
              {
                key: 's3',
                titleKey: 'rules.s3Title',
                descKey: 'rules.s3Desc',
                exKey: 'rules.s3Example',
              },
              {
                key: 's4',
                titleKey: 'rules.s4Title',
                descKey: 'rules.s4Desc',
                exKey: 'rules.s4Example',
              },
              {
                key: 's5',
                titleKey: 'rules.s5Title',
                descKey: 'rules.s5Desc',
                exKey: 'rules.s5Example',
              },
              {
                key: 's6',
                titleKey: 'rules.s6Title',
                descKey: 'rules.s6Desc',
                exKey: 'rules.s6Example',
              },
              {
                key: 's7',
                titleKey: 'rules.s7Title',
                descKey: 'rules.s7Desc',
                exKey: 'rules.s7Example',
              },
              {
                key: 's8',
                titleKey: 'rules.s8Title',
                descKey: 'rules.s8Desc',
                exKey: 'rules.s8Example',
              },
            ].map(({ key, titleKey, descKey, exKey }) => {
              const rule = getRule(key)
              const weight = rule?.weight ?? 10.0
              const enabled = rule?.enabled ?? true

              return (
                <Card
                  key={key}
                  className="bg-card border-border"
                  data-testid={`soft-rule-card-${key}`}
                >
                  <CardHeader className="p-4 pb-2 flex flex-row items-center justify-between">
                    <CardTitle className="text-sm font-semibold text-foreground">
                      {t(titleKey)}
                    </CardTitle>
                    <div className="flex items-center gap-2">
                      <Badge
                        variant="outline"
                        className="font-mono text-xs w-12 justify-center"
                      >
                        {weight.toFixed(1)}
                      </Badge>
                      <Switch
                        checked={enabled}
                        onCheckedChange={(val) => updateRule(key, { enabled: val })}
                        data-testid={`toggle-${key}`}
                      />
                    </div>
                  </CardHeader>
                  <CardContent className="p-4 pt-1 space-y-3 text-xs">
                    <p className="text-muted-foreground">{t(descKey)}</p>
                    <p className="text-[11px] text-muted-foreground/80 italic bg-muted/40 p-2 rounded">
                      {t(exKey)}
                    </p>
                    <div className="space-y-1">
                      <div className="flex justify-between text-[11px] text-muted-foreground">
                        <span>
                          {t('rules.weight')}: {weight}
                        </span>
                        <span>[0.0 — 20.0]</span>
                      </div>
                      <input
                        type="range"
                        min="0"
                        max="20"
                        step="0.5"
                        value={weight}
                        disabled={!enabled}
                        onChange={(e) =>
                          updateRule(key, { weight: parseFloat(e.target.value) })
                        }
                        className="w-full accent-primary cursor-pointer disabled:opacity-40"
                        data-testid={`slider-${key}`}
                      />
                    </div>
                  </CardContent>
                </Card>
              )
            })}
          </div>
        </div>
      )}

      {/* TAB 3: QUOTA PREVIEW TABLE */}
      {activeTab === 'quotas' && (
        <Card className="bg-card border-border" data-testid="quotas-preview-section">
          <CardHeader className="p-4 pb-3">
            <CardTitle className="text-base font-semibold text-foreground flex items-center justify-between">
              <span>{t('quotas.title')}</span>
              <Badge variant="outline" className="font-mono text-xs">
                {t('quotas.totalTeachers', { count: quotaPreview.length })}
              </Badge>
            </CardTitle>
            <CardDescription className="text-xs text-muted-foreground">
              {t('quotas.description')} (H7 tolerance = {currentTolerance})
            </CardDescription>
          </CardHeader>
          <CardContent className="p-0">
            <div className="overflow-x-auto">
              <table
                className="w-full border-collapse text-left text-xs"
                data-testid="quota-preview-table"
              >
                <thead>
                  <tr className="border-b border-border bg-muted/40 font-semibold text-muted-foreground">
                    <th className="p-3">{t('quotas.teacher')}</th>
                    <th className="p-3">{t('quotas.campus')}</th>
                    <th className="p-3 text-center">{t('quotas.loadWeight')}</th>
                    <th className="p-3 text-center">{t('quotas.availableExams')}</th>
                    <th className="p-3 text-center">{t('quotas.quota')}</th>
                    <th className="p-3 text-center">{t('quotas.bounds')}</th>
                  </tr>
                </thead>
                <tbody className="divide-y divide-border">
                  {quotaPreview.map((item) => (
                    <tr
                      key={item.teacher_id}
                      className="hover:bg-muted/20 transition-colors"
                      data-testid={`quota-row-${item.teacher_id}`}
                    >
                      <td className="p-3 font-medium text-foreground">
                        {item.teacher_name}
                      </td>
                      <td className="p-3 text-muted-foreground">{item.campus_name}</td>
                      <td className="p-3 text-center font-mono">{item.load_weight}</td>
                      <td className="p-3 text-center font-mono">
                        {item.available_exams}
                      </td>
                      <td className="p-3 text-center font-mono font-semibold text-foreground">
                        {item.quota.toFixed(2)}
                      </td>
                      <td className="p-3 text-center font-mono font-bold text-primary">
                        [{item.lo}, {item.hi}]
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          </CardContent>
        </Card>
      )}

      {/* TAB 4: LOCKS TAB */}
      {activeTab === 'locks' && (
        <Card className="bg-card border-border" data-testid="locks-section">
          <CardHeader className="p-4 pb-3 flex flex-row items-center justify-between">
            <div>
              <CardTitle className="text-base font-semibold text-foreground flex items-center gap-2">
                <LockIcon className="h-4 w-4 text-primary" />
                <span>{t('locks.title')}</span>
              </CardTitle>
              <CardDescription className="text-xs text-muted-foreground mt-0.5">
                {t('locks.description')}
              </CardDescription>
            </div>
            <Button
              size="sm"
              onClick={handleOpenCreateLock}
              className="gap-1.5 h-8 text-xs"
              data-testid="add-lock-btn"
            >
              <Plus className="h-3.5 w-3.5" />
              <span>{t('locks.createLock')}</span>
            </Button>
          </CardHeader>
          <CardContent className="p-4 pt-1 space-y-4">
            {locks.length === 0 ? (
              <div className="p-8 text-center text-xs text-muted-foreground">
                {t('locks.empty')}
              </div>
            ) : (
              <div className="space-y-3">
                {locks.map((lock) => {
                  const exam = exams.find((e) => e.id === lock.exam_id)
                  const grade = grades.find((g) => g.id === lock.grade_id)
                  const teacher = teachersWithGrades.find(
                    (twg) => twg.teacher.id === lock.teacher_id,
                  )?.teacher
                  const validation = getLockValidationDiagnostic(lock)

                  return (
                    <div
                      key={lock.id}
                      className={`p-3 rounded-lg border flex flex-col sm:flex-row sm:items-center justify-between gap-3 transition-colors ${
                        validation
                          ? 'border-destructive/40 bg-destructive/5'
                          : 'border-border bg-muted/20 hover:bg-muted/30'
                      }`}
                      data-testid={`lock-item-${lock.id}`}
                    >
                      <div className="space-y-1">
                        <div className="flex flex-wrap items-center gap-2">
                          <Badge
                            variant={lock.kind === 'pin' ? 'default' : 'destructive'}
                            className="text-[10px] uppercase font-semibold"
                          >
                            {lock.kind === 'pin'
                              ? t('locks.kindPin')
                              : t('locks.kindForbid')}
                          </Badge>
                          <span className="font-semibold text-xs text-foreground">
                            {teacher?.full_name || `GV #${lock.teacher_id}`}
                          </span>
                          <span className="text-xs text-muted-foreground">
                            → {exam?.name || `Kỳ ${lock.exam_id}`} —{' '}
                            {grade?.name || `Khối ${lock.grade_id}`}
                          </span>
                          {lock.role && (
                            <Badge variant="outline" className="text-[10px]">
                              {lock.role === 'setter'
                                ? t('locks.roleSetter')
                                : t('locks.roleReviewer')}
                            </Badge>
                          )}
                        </div>

                        {/* Inline live validation diagnostic (Part F) */}
                        {validation ? (
                          <div
                            className="flex items-center gap-1.5 text-[11px] text-destructive font-medium pt-0.5"
                            data-testid="lock-diagnostic-error"
                          >
                            <AlertOctagon className="h-3.5 w-3.5 shrink-0" />
                            <span>{validation.message}</span>
                          </div>
                        ) : (
                          <div className="flex items-center gap-1 text-[11px] text-emerald-600 dark:text-emerald-400">
                            <CheckCircle2 className="h-3 w-3 shrink-0" />
                            <span>{t('locks.statusOk')}</span>
                          </div>
                        )}
                      </div>

                      <Button
                        variant="ghost"
                        size="sm"
                        onClick={() => setDeletingLock(lock)}
                        className="h-7 text-xs text-destructive hover:bg-destructive/10 px-2"
                        data-testid={`delete-lock-btn-${lock.id}`}
                      >
                        <Trash2 className="h-3.5 w-3.5 mr-1" />
                        <span>{t('common.delete')}</span>
                      </Button>
                    </div>
                  )
                })}
              </div>
            )}
          </CardContent>
        </Card>
      )}

      {/* Create Lock Dialog */}
      <Dialog open={lockDialogOpen} onOpenChange={setLockDialogOpen}>
        <DialogContent className="sm:max-w-md bg-card text-foreground border-border">
          <DialogHeader>
            <DialogTitle>{t('locks.createLock')}</DialogTitle>
          </DialogHeader>
          <form onSubmit={handleSaveLock} className="space-y-4 pt-2">
            <div className="grid grid-cols-3 gap-3">
              <div className="space-y-1.5">
                <label className="text-xs font-semibold text-foreground">
                  {t('locks.exam')}
                </label>
                <Select value={lockExamId} onValueChange={setLockExamId}>
                  <SelectTrigger
                    className="text-xs bg-background"
                    data-testid="lock-exam-select"
                  >
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    {exams.map((e) => (
                      <SelectItem key={e.id} value={e.id.toString()}>
                        {e.name}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>

              <div className="space-y-1.5">
                <label className="text-xs font-semibold text-foreground">
                  {t('locks.grade')}
                </label>
                <Select value={lockGradeId} onValueChange={setLockGradeId}>
                  <SelectTrigger
                    className="text-xs bg-background"
                    data-testid="lock-grade-select"
                  >
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    {grades.map((g) => (
                      <SelectItem key={g.id} value={g.id.toString()}>
                        {g.name}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>

              <div className="space-y-1.5">
                <label className="text-xs font-semibold text-foreground">
                  {t('subjects.title', { defaultValue: 'Môn học' })}
                </label>
                <Select value={lockSubjectId} onValueChange={setLockSubjectId}>
                  <SelectTrigger
                    className="text-xs bg-background"
                    data-testid="lock-subject-select"
                  >
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    {subjects.map((s) => (
                      <SelectItem key={s.id} value={s.id.toString()}>
                        {s.name} ({s.code})
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>
            </div>

            {/* Teacher Select with Eligibility Filtering */}
            <div className="space-y-1.5">
              <label className="text-xs font-semibold text-foreground">
                {t('locks.teacher')}
              </label>
              <Select value={lockTeacherId} onValueChange={setLockTeacherId}>
                <SelectTrigger
                  className="text-xs bg-background"
                  data-testid="lock-teacher-select"
                >
                  <SelectValue placeholder="Chọn giáo viên..." />
                </SelectTrigger>
                <SelectContent>
                  {teacherEligibilityList.map(({ teacher, isEligible, reason }) => (
                    <SelectItem
                      key={teacher.id}
                      value={teacher.id.toString()}
                      disabled={!isEligible}
                      className={!isEligible ? 'opacity-50 text-muted-foreground' : ''}
                    >
                      {teacher.full_name} {!isEligible && `(${reason})`}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>

            <div className="grid grid-cols-2 gap-3">
              <div className="space-y-1.5">
                <label className="text-xs font-semibold text-foreground">
                  {t('locks.kind')}
                </label>
                <Select
                  value={lockKind}
                  onValueChange={(val) => setLockKind(val as LockKind)}
                >
                  <SelectTrigger
                    className="text-xs bg-background"
                    data-testid="lock-kind-select"
                  >
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="pin">{t('locks.kindPin')}</SelectItem>
                    <SelectItem value="forbid">{t('locks.kindForbid')}</SelectItem>
                  </SelectContent>
                </Select>
              </div>

              <div className="space-y-1.5">
                <label className="text-xs font-semibold text-foreground">
                  {t('locks.role')}
                </label>
                <Select value={lockRole} onValueChange={setLockRole}>
                  <SelectTrigger
                    className="text-xs bg-background"
                    data-testid="lock-role-select"
                  >
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="any">{t('locks.roleAny')}</SelectItem>
                    <SelectItem value="setter">{t('locks.roleSetter')}</SelectItem>
                    <SelectItem value="reviewer">{t('locks.roleReviewer')}</SelectItem>
                  </SelectContent>
                </Select>
              </div>
            </div>

            <DialogFooter className="pt-2">
              <Button
                type="button"
                variant="outline"
                onClick={() => setLockDialogOpen(false)}
              >
                {t('common.cancel')}
              </Button>
              <Button
                type="submit"
                disabled={!lockTeacherId}
                data-testid="lock-submit-btn"
              >
                {t('common.save')}
              </Button>
            </DialogFooter>
          </form>
        </DialogContent>
      </Dialog>

      {/* Delete Lock Confirmation Dialog */}
      <AlertDialog
        open={!!deletingLock}
        onOpenChange={(open) => !open && setDeletingLock(null)}
      >
        <AlertDialogContent className="bg-card text-foreground border-border">
          <AlertDialogHeader>
            <AlertDialogTitle>{t('locks.deleteConfirm')}</AlertDialogTitle>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>{t('common.cancel')}</AlertDialogCancel>
            <AlertDialogAction
              onClick={handleConfirmDeleteLock}
              className="bg-destructive text-destructive-foreground hover:bg-destructive/90"
              data-testid="confirm-delete-lock-btn"
            >
              {t('common.delete')}
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>

      {/* Reset Confirmation Dialog */}
      <AlertDialog open={resetConfirmOpen} onOpenChange={setResetConfirmOpen}>
        <AlertDialogContent className="bg-card text-foreground border-border">
          <AlertDialogHeader>
            <AlertDialogTitle>{t('rules.resetConfirm')}</AlertDialogTitle>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>{t('common.cancel')}</AlertDialogCancel>
            <AlertDialogAction
              onClick={handleConfirmReset}
              className="bg-primary text-primary-foreground"
              data-testid="confirm-reset-btn"
            >
              {t('rules.resetDefaults')}
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </div>
  )
}
