import { describe, it, expect, beforeEach, vi } from 'vitest'
import { render, screen, fireEvent, waitFor } from '@testing-library/react'
import { ImportPlanModal } from './ImportPlanModal'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { ThemeProvider } from '@/lib/theme'
import '@/i18n'
import { api } from '@/lib/api'

function renderModal(props: {
  open: boolean
  onOpenChange: (open: boolean) => void
  schoolYearId: number
  onSuccess: (newPlanId: number) => void
}) {
  const testClient = new QueryClient({
    defaultOptions: {
      queries: { retry: false, staleTime: 0 },
      mutations: { retry: false },
    },
  })
  return render(
    <ThemeProvider>
      <QueryClientProvider client={testClient}>
        <ImportPlanModal {...props} />
      </QueryClientProvider>
    </ThemeProvider>,
  )
}

describe('ImportPlanModal Component Tests (Part D)', () => {
  beforeEach(async () => {
    localStorage.clear()
    await api.seedDemo()
  })

  it('renders modal in input step with Excel and TSV tabs', async () => {
    const onOpenChange = vi.fn()
    const onSuccess = vi.fn()

    renderModal({
      open: true,
      onOpenChange,
      schoolYearId: 1,
      onSuccess,
    })

    expect(screen.getByTestId('import-plan-modal')).toBeInTheDocument()
    expect(screen.getByTestId('tab-source-excel')).toBeInTheDocument()
    expect(screen.getByTestId('tab-source-tsv')).toBeInTheDocument()
    expect(screen.getByTestId('btn-plan-import-preview')).toBeInTheDocument()
  })

  it('switches to TSV tab, enters table data, runs preview and displays results', async () => {
    const onOpenChange = vi.fn()
    const onSuccess = vi.fn()

    renderModal({
      open: true,
      onOpenChange,
      schoolYearId: 1,
      onSuccess,
    })

    // Click TSV tab
    fireEvent.click(screen.getByTestId('tab-source-tsv'))
    const textarea = screen.getByTestId('textarea-plan-tsv')
    expect(textarea).toBeInTheDocument()

    // Enter simple TSV sample
    fireEvent.change(textarea, {
      target: { value: 'Kì thi/khối\t\tKhối 10\t\tKhối 11\t\tKhối 12\nGK1\tĐề\tC Hiền\tT Nghĩa\tC Lài\tT Nghĩa\tT Phúc\tT Nghĩa' },
    })

    // Click Preview
    const previewBtn = screen.getByTestId('btn-plan-import-preview')
    fireEvent.click(previewBtn)

    // Wait for preview step
    await waitFor(() => {
      expect(screen.getByTestId('tab-preview-totals')).toBeInTheDocument()
    })

    // Check teacher totals tab and action buttons
    expect(screen.getByTestId('btn-plan-import-back')).toBeInTheDocument()
    expect(screen.getByTestId('btn-plan-import-apply')).toBeInTheDocument()

    // Click Apply
    fireEvent.click(screen.getByTestId('btn-plan-import-apply'))

    await waitFor(() => {
      expect(onSuccess).toHaveBeenCalled()
    })
  })
})
