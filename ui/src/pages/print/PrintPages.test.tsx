import { render, screen, fireEvent } from '@testing-library/react'
import { describe, it, expect, vi, beforeEach } from 'vitest'
import { MemoryRouter, Routes, Route } from 'react-router-dom'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { PrintPlanPage } from './PrintPlanPage'
import { PrintNoticesPage } from './PrintNoticesPage'
import '@/i18n'
import { api } from '@/lib/api'

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
    expect(
      screen.getByText(/BẢNG PHÂN CÔNG RA ĐỀ VÀ PHẢN BIỆN ĐỀ KIỂM TRA/i),
    ).toBeInTheDocument()
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
    expect(
      screen.getAllByText(/THÔNG BÁO PHÂN CÔNG RA ĐỀ VÀ PHẢN BIỆN ĐỀ KIỂM TRA/i).length,
    ).toBeGreaterThan(0)
    expect(screen.getAllByText(/Kính gửi Thầy\/Cô:/i).length).toBeGreaterThan(0)
    expect(
      screen.getAllByText(/Danh sách nhiệm vụ được phân công:/i).length,
    ).toBeGreaterThan(0)
  })

  // RA-030: window.print() is a no-op in WebKitGTK; printing must go through the API (native print)
  it.each([
    ['plan', '/print/plan/1', '/print/plan/:id', PrintPlanPage],
    ['notices', '/print/notices/1', '/print/notices/:id', PrintNoticesPage],
  ])(
    'the %s print button uses api.printPage, not window.print',
    async (_n, url, route, Page) => {
      const printPage = vi.spyOn(api, 'printPage').mockResolvedValue(undefined)
      const windowPrint = vi.fn()
      vi.stubGlobal('print', windowPrint)
      try {
        render(
          <QueryClientProvider client={queryClient}>
            <MemoryRouter initialEntries={[url]}>
              <Routes>
                <Route path={route} element={<Page />} />
              </Routes>
            </MemoryRouter>
          </QueryClientProvider>,
        )
        fireEvent.click(await screen.findByTestId('print-btn'))
        expect(printPage).toHaveBeenCalledTimes(1)
        expect(windowPrint).not.toHaveBeenCalled()
      } finally {
        vi.unstubAllGlobals()
      }
    },
  )

  // RA-029: nothing invented when Settings has no organisation info
  it('prints no invented organisation text when settings are empty', async () => {
    vi.spyOn(api, 'getSettings').mockResolvedValue({
      theme: 'light',
      language: 'vi',
      current_school_year_id: 1,
      school_name: null,
      department_name: null,
      signer_title: null,
      signer_name: null,
      place_name: null,
    } as never)
    render(
      <QueryClientProvider client={queryClient}>
        <MemoryRouter initialEntries={['/print/plan/1']}>
          <Routes>
            <Route path="/print/plan/:id" element={<PrintPlanPage />} />
          </Routes>
        </MemoryRouter>
      </QueryClientProvider>,
    )
    expect(await screen.findByTestId('org-missing-banner')).toBeInTheDocument()
    const text = document.body.textContent ?? ''
    for (const invented of [
      /TRƯỜNG THPT CHUYÊN/,
      /TỔ TOÁN - TIN/,
      /Hà Nội/,
      /Nguyễn Văn A(?!n)/,
    ]) {
      expect(text).not.toMatch(invented)
    }
  })
})
