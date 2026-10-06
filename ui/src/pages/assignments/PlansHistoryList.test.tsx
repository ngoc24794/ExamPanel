import { describe, it, expect, afterEach, vi } from 'vitest'
import { render, screen } from '@testing-library/react'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import '@/i18n'
import type { PlanSummary } from '@/lib/api'
import { toIsoUtc, parseDbDate } from '@/lib/dates'
import { PlansHistoryList } from './PlansHistoryList'

const RealDate = Date

/** WebKit-like Date: only ISO 8601 strings parse; SQLite's `YYYY-MM-DD HH:MM:SS` does not. */
function installStrictDate() {
  class StrictDate extends RealDate {
    constructor(...args: unknown[]) {
      if (args.length === 1 && typeof args[0] === 'string') {
        const iso = /^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(\.\d+)?(Z|[+-]\d{2}:?\d{2})?$/
        super(iso.test(args[0]) ? args[0] : Number.NaN)
      } else {
        super(...(args as [number]))
      }
    }
  }
  vi.stubGlobal('Date', StrictDate)
}

const plan: PlanSummary = {
  id: 1,
  name: 'Phương án #1',
  score: 176.36,
  created_at: '2026-10-05 12:34:56', // exactly what SQLite datetime('now') stores
  is_final: false,
  source: 'optimizer',
  is_stale: false,
}

describe('plan creation date (RA-024)', () => {
  afterEach(() => vi.unstubAllGlobals())

  it('normalises SQLite timestamps to ISO 8601 UTC', () => {
    expect(toIsoUtc('2026-10-05 12:34:56')).toBe('2026-10-05T12:34:56Z')
    expect(toIsoUtc('2026-10-05T12:34:56Z')).toBe('2026-10-05T12:34:56Z')
    expect(toIsoUtc('not a date')).toBe('not a date')
    expect(parseDbDate('garbage')).toBeNull()
  })

  it('history list shows a real date, never "Invalid Date", on a strict (WebKit-like) engine', () => {
    installStrictDate()
    render(
      <QueryClientProvider client={new QueryClient()}>
        <PlansHistoryList
          plans={[plan]}
          activePlanId={1}
          schoolYearId={1}
          onSelectPlan={() => {}}
        />
      </QueryClientProvider>,
    )
    expect(screen.queryByText(/Invalid Date/)).not.toBeInTheDocument()
    expect(screen.getByText(/2026/)).toBeInTheDocument()
  })
})
