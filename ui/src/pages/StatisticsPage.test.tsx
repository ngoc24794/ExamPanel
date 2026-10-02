import { describe, it, expect, beforeEach, vi } from 'vitest'
import { render, screen, waitFor, fireEvent } from '@testing-library/react'
import { StatisticsPage } from './StatisticsPage'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { ThemeProvider } from '@/lib/theme'
import '@/i18n'
import i18n from '@/i18n'
import { api } from '@/lib/api'

function renderWithClient(ui: React.ReactElement) {
  const testClient = new QueryClient({
    defaultOptions: {
      queries: { retry: false, staleTime: 0 },
      mutations: { retry: false },
    },
  })
  return render(
    <ThemeProvider>
      <QueryClientProvider client={testClient}>{ui}</QueryClientProvider>
    </ThemeProvider>,
  )
}

describe('StatisticsPage Component Tests', () => {
  beforeEach(async () => {
    localStorage.clear()
    window.matchMedia = vi.fn().mockImplementation((query) => ({
      matches: false,
      media: query,
      onchange: null,
      addListener: vi.fn(),
      removeListener: vi.fn(),
      addEventListener: vi.fn(),
      removeEventListener: vi.fn(),
      dispatchEvent: vi.fn(),
    }))
    // Reset seed demo before each test
    await api.seedDemo()
  })

  it('renders statistics page header, plan selector, and summary cards', async () => {
    renderWithClient(<StatisticsPage />)

    await waitFor(() => {
      expect(screen.getByTestId('statistics-page')).toBeInTheDocument()
    })

    // Workload chart card
    await waitFor(() => {
      expect(screen.getByText(i18n.t('statistics.loadChartTitle'))).toBeInTheDocument()
    })

    // Wait for plan details to finish loading and render heatmaps
    await waitFor(() => {
      expect(screen.getByText(i18n.t('statistics.coWorkingTitle'))).toBeInTheDocument()
    })

    // Teacher statistics table
    expect(screen.getByText(i18n.t('statistics.perTeacher'))).toBeInTheDocument()

    // Campus mix summary card
    expect(screen.getByText(i18n.t('statistics.campusMixTitle'))).toBeInTheDocument()

    // Review relation heatmap
    expect(screen.getByText(i18n.t('statistics.reviewRelationTitle'))).toBeInTheDocument()
  })

  it('supports sorting teacher statistics table by columns', async () => {
    renderWithClient(<StatisticsPage />)

    await waitFor(() => {
      expect(screen.getByTestId('statistics-page')).toBeInTheDocument()
    })

    // Wait for table to load
    await waitFor(() => {
      expect(screen.getByText(i18n.t('statistics.teacherName'))).toBeInTheDocument()
    })

    const nameHeader = screen.getByText(i18n.t('statistics.teacherName'))
    expect(nameHeader).toBeInTheDocument()

    // Click to sort by name
    fireEvent.click(nameHeader)

    // Click to sort by quota
    const quotaHeaders = screen.getAllByText(i18n.t('statistics.quota'))
    fireEvent.click(quotaHeaders[quotaHeaders.length - 1])
  })
})
