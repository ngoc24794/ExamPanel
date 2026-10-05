import { describe, it, expect, beforeEach, vi } from 'vitest'
import { render, screen, waitFor } from '@testing-library/react'
import { OverviewPage } from './OverviewPage'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { ThemeProvider } from '@/lib/theme'
import { HashRouter } from 'react-router-dom'
import '@/i18n'
import { api } from '@/lib/api'

function renderWithClient(ui: React.ReactElement) {
  const testClient = new QueryClient({
    defaultOptions: {
      queries: { retry: false, staleTime: 0 },
      mutations: { retry: false },
    },
  })
  return render(
    <HashRouter>
      <ThemeProvider>
        <QueryClientProvider client={testClient}>{ui}</QueryClientProvider>
      </ThemeProvider>
    </HashRouter>,
  )
}

describe('OverviewPage & Onboarding', () => {
  beforeEach(async () => {
    localStorage.clear()
    await api.seedDemo()
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
  })

  it('renders overview stats cards and ping response', async () => {
    renderWithClient(<OverviewPage />)

    await waitFor(() => {
      expect(
        screen.getByText(/Bảng điều khiển tổng quan|System Overview/i),
      ).toBeInTheDocument()
      expect(screen.getByText(/pong from Mock Engine/i)).toBeInTheDocument()
    })

    // With demo data loaded, campuses and teachers are present
    expect(screen.getAllByText(/Phân hiệu|Campuses/i).length).toBeGreaterThan(0)
    expect(screen.getAllByText(/Giáo viên|Teachers/i).length).toBeGreaterThan(0)
  })

  it('shows onboarding checklist when campuses or teachers are missing', async () => {
    // Clear campuses in mock to simulate empty state
    const originalListCampuses = api.listCampuses
    api.listCampuses = vi.fn().mockResolvedValue([])

    renderWithClient(<OverviewPage />)

    // Should display onboarding checklist
    await waitFor(() => {
      expect(
        screen.getByText(/Bắt đầu với ExamPanel|Getting Started with ExamPanel/i),
      ).toBeInTheDocument()
    })

    // Check that subjects and competencies steps are present
    expect(
      screen.getByText(/3\. Cấu hình môn học|3\. Configure subjects/i),
    ).toBeInTheDocument()
    expect(
      screen.getByText(/4\. Phân công môn đảm nhiệm|4\. Assign subject competencies/i),
    ).toBeInTheDocument()

    // Restore
    api.listCampuses = originalListCampuses
  })

  // RA-004: the developer "Rust core ping" panel must not appear in production builds
  it('does not show the developer IPC ping panel in a production build', async () => {
    vi.stubEnv('DEV', false)
    try {
      renderWithClient(<OverviewPage />)
      await waitFor(() => {
        expect(
          screen.getByText(/Bảng điều khiển tổng quan|System Overview/i),
        ).toBeInTheDocument()
      })
      expect(screen.queryByTestId('ping-result')).not.toBeInTheDocument()
      expect(screen.queryByText(/pong from/i)).not.toBeInTheDocument()
    } finally {
      vi.unstubAllEnvs()
    }
  })
})
