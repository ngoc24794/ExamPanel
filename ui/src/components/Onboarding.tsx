import * as React from 'react'
import { useTranslation } from 'react-i18next'
import { useNavigate } from 'react-router-dom'
import {
  CheckCircle2,
  Circle,
  ArrowRight,
  Sparkles,
  Building2,
  Users,
  BookOpen,
  Award,
} from 'lucide-react'
import {
  Card,
  CardHeader,
  CardTitle,
  CardDescription,
  CardContent,
} from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { useSeedDemo } from '@/lib/query/hooks'

interface OnboardingProps {
  campusesCount: number
  teachersCount: number
  subjectsCount?: number
  competenciesCount?: number
}

export const Onboarding: React.FC<OnboardingProps> = ({
  campusesCount,
  teachersCount,
  subjectsCount = 0,
  competenciesCount = 0,
}) => {
  const { t } = useTranslation()
  const navigate = useNavigate()
  const seedDemoMutation = useSeedDemo()

  const hasCampuses = campusesCount > 0
  const hasTeachers = teachersCount > 0
  const hasSubjects = subjectsCount > 0
  const hasCompetencies = competenciesCount > 0

  const steps = [
    {
      id: 1,
      title: t('onboarding.step1'),
      description: t('onboarding.step1Desc'),
      done: hasCampuses,
      action: () => navigate('/campuses'),
      actionLabel: t('onboarding.goToCampuses'),
      icon: <Building2 className="h-5 w-5" />,
    },
    {
      id: 2,
      title: t('onboarding.step2'),
      description: t('onboarding.step2Desc'),
      done: hasTeachers,
      action: () => navigate('/teachers'),
      actionLabel: t('onboarding.goToTeachers'),
      icon: <Users className="h-5 w-5" />,
    },
    {
      id: 3,
      title: t('onboarding.stepSubjects'),
      description: t('onboarding.stepSubjectsDesc'),
      done: hasSubjects,
      action: () => navigate('/subjects'),
      actionLabel: t('onboarding.goToSubjects'),
      icon: <BookOpen className="h-5 w-5" />,
    },
    {
      id: 4,
      title: t('onboarding.stepCompetencies'),
      description: t('onboarding.stepCompetenciesDesc'),
      done: hasCompetencies,
      action: () => navigate('/competencies'),
      actionLabel: t('onboarding.goToCompetencies'),
      icon: <Award className="h-5 w-5" />,
    },
    {
      id: 5,
      title: t('onboarding.step3'),
      description: t('onboarding.step3Desc'),
      done: hasCampuses && hasTeachers && hasSubjects,
      action: () => navigate('/rules'),
      actionLabel: t('onboarding.goToRules'),
      icon: <Sparkles className="h-5 w-5" />,
    },
    {
      id: 6,
      title: t('onboarding.step4'),
      description: t('onboarding.step4Desc'),
      done: false,
      action: () => navigate('/assignments'),
      actionLabel: t('onboarding.goToAssignments'),
      icon: <Sparkles className="h-5 w-5" />,
    },
  ]

  return (
    <Card className="border-primary/20 shadow-md">
      <CardHeader className="pb-3">
        <div className="flex items-center justify-between">
          <div>
            <CardTitle className="text-xl font-bold flex items-center gap-2">
              <Sparkles className="h-5 w-5 text-primary" />
              {t('onboarding.title')}
            </CardTitle>
            <CardDescription className="mt-1">{t('onboarding.subtitle')}</CardDescription>
          </div>
          {import.meta.env.DEV && (
            <Button
              variant="outline"
              size="sm"
              onClick={() => seedDemoMutation.mutate()}
              disabled={seedDemoMutation.isPending}
              className="gap-2 border-primary/40 text-primary hover:bg-primary/10"
            >
              <Sparkles className="h-4 w-4" />
              {t('onboarding.loadDemoBtn')}
            </Button>
          )}
        </div>
      </CardHeader>
      <CardContent>
        <div className="grid gap-3 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-6">
          {steps.map((step) => (
            <div
              key={step.id}
              className={`p-4 rounded-lg border transition-all ${
                step.done
                  ? 'bg-muted/40 border-border text-muted-foreground'
                  : 'bg-card border-border/80 shadow-sm'
              }`}
            >
              <div className="flex items-start justify-between">
                <span className="font-semibold text-sm flex items-center gap-2 text-foreground">
                  {step.done ? (
                    <CheckCircle2 className="h-4 w-4 text-emerald-500 shrink-0" />
                  ) : (
                    <Circle className="h-4 w-4 text-muted-foreground shrink-0" />
                  )}
                  {step.title}
                </span>
              </div>
              <p className="text-xs text-muted-foreground mt-2 min-h-[32px]">
                {step.description}
              </p>
              {step.action && !step.done && (
                <Button
                  size="sm"
                  variant="secondary"
                  className="mt-3 w-full text-xs gap-1.5 h-8"
                  onClick={step.action}
                >
                  {step.actionLabel}
                  <ArrowRight className="h-3.5 w-3.5" />
                </Button>
              )}
            </div>
          ))}
        </div>
      </CardContent>
    </Card>
  )
}
