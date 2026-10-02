import * as React from 'react'
import { useTranslation } from 'react-i18next'
import {
  Card,
  CardHeader,
  CardTitle,
  CardDescription,
  CardContent,
} from '@/components/ui/card'
import { Info } from 'lucide-react'

interface PlaceholderPageProps {
  titleKey: string
  descKey: string
}

export const PlaceholderPage: React.FC<PlaceholderPageProps> = ({
  titleKey,
  descKey,
}) => {
  const { t } = useTranslation()

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-2xl font-bold tracking-tight text-foreground">
          {t(titleKey)}
        </h1>
        <p className="text-sm text-muted-foreground mt-1">{t(descKey)}</p>
      </div>

      <Card className="border-dashed bg-card/50">
        <CardHeader>
          <CardTitle className="text-base font-semibold flex items-center gap-2 text-foreground">
            <Info className="h-5 w-5 text-primary" />
            Phase 1 Skeleton
          </CardTitle>
          <CardDescription>{t('common.placeholderNotice')}</CardDescription>
        </CardHeader>
        <CardContent>
          <p className="text-xs text-muted-foreground">
            Các tính năng chuyên sâu của mô-đun này sẽ được phát triển trong các giai đoạn
            tiếp theo theo kiến trúc dự án.
          </p>
        </CardContent>
      </Card>
    </div>
  )
}
