import { render, screen } from '@testing-library/react'
import { describe, it, expect, vi, beforeEach } from 'vitest'
import { MemoryRouter, Routes, Route } from 'react-router-dom'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { PrintPlanPage } from './PrintPlanPage'
import { PrintNoticesPage } from './PrintNoticesPage'
import '@/i18n'

describe('Print Pages Component Tests', () => {
  let queryClient: QueryClient

  beforeEach(() => {
    queryClient = new QueryClient({
      defaultOptions: {
        queries: { retry: false },
      },
    })
    vi.clearAllMocks()
  })

  it('renders PrintPlanPage with school header, matrix table, and print button', async () => {
    render(
      <QueryClientProvider client={queryClient}>
        <MemoryRouter initialEntries={['/print/plan/1']}>
          <Routes>
            <Route path="/print/plan/:id" element={<PrintPlanPage />} />
          </Routes>
        </MemoryRouter>
      </QueryClientProvider>,
    )

    expect(await screen.findByTestId('print-btn')).toBeInTheDocument()
    expect(screen.getByText(/BẢNG PHÂN CÔNG RA ĐỀ VÀ PHẢN BIỆN ĐỀ KIỂM TRA/i)).toBeInTheDocument()
    expect(screen.getByText('Kì thi/khối')).toBeInTheDocument()
    expect(screen.getByText('Tổng lượt n.vụ')).toBeInTheDocument()
    expect(screen.getAllByText('P.Biện').length).toBeGreaterThan(0)
  })

  it('renders PrintNoticesPage with teacher assignment notices', async () => {
    render(
      <QueryClientProvider client={queryClient}>
        <MemoryRouter initialEntries={['/print/notices/1']}>
          <Routes>
            <Route path="/print/notices/:id" element={<PrintNoticesPage />} />
          </Routes>
        </MemoryRouter>
      </QueryClientProvider>,
    )

    expect(await screen.findByTestId('print-btn')).toBeInTheDocument()
    expect(screen.getAllByText(/THÔNG BÁO PHÂN CÔNG RA ĐỀ VÀ PHẢN BIỆN ĐỀ KIỂM TRA/i).length).toBeGreaterThan(0)
    expect(screen.getAllByText(/Kính gửi Thầy\/Cô:/i).length).toBeGreaterThan(0)
    expect(screen.getAllByText(/Danh sách nhiệm vụ được phân công:/i).length).toBeGreaterThan(0)
  })
})
