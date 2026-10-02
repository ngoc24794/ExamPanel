import { render, screen, fireEvent, waitFor } from '@testing-library/react'
import { describe, it, expect, vi, beforeEach } from 'vitest'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { ImportWizardModal } from './ImportWizardModal'
import { mockApi } from '@/lib/api/mock'
import '@/i18n'

describe('ImportWizardModal Component Tests', () => {
  let queryClient: QueryClient

  beforeEach(() => {
    queryClient = new QueryClient({
      defaultOptions: {
        queries: { retry: false },
      },
    })
    vi.clearAllMocks()
  })

  it('renders file selection and mode cards when open', () => {
    render(
      <QueryClientProvider client={queryClient}>
        <ImportWizardModal
          open={true}
          onOpenChange={vi.fn()}
          schoolYearId={1}
          onSuccess={vi.fn()}
        />
      </QueryClientProvider>,
    )

    expect(screen.getByTestId('import-file-input')).toBeInTheDocument()
    expect(screen.getByTestId('import-browse-btn')).toBeInTheDocument()
    expect(screen.getByTestId('mode-upsert-card')).toBeInTheDocument()
    expect(screen.getByTestId('mode-sync-card')).toBeInTheDocument()
    expect(screen.getByTestId('import-run-preview-btn')).toBeInTheDocument()
  })

  it('navigates from select to preview and displays preview tables and filters', async () => {
    const previewSpy = vi.spyOn(mockApi, 'previewImport')
    const applySpy = vi.spyOn(mockApi, 'applyImport')
    const onSuccess = vi.fn()

    render(
      <QueryClientProvider client={queryClient}>
        <ImportWizardModal
          open={true}
          onOpenChange={vi.fn()}
          schoolYearId={1}
          onSuccess={onSuccess}
        />
      </QueryClientProvider>,
    )

    // Fill file path
    const input = screen.getByTestId('import-file-input')
    fireEvent.change(input, { target: { value: 'test.xlsx' } })

    // Click mode sync
    fireEvent.click(screen.getByTestId('mode-sync-card'))

    // Click run preview
    fireEvent.click(screen.getByTestId('import-run-preview-btn'))

    await waitFor(() => {
      expect(previewSpy).toHaveBeenCalledWith(1, 'test.xlsx', 'sync')
    })

    // Check preview tables and tabs are shown
    expect(await screen.findByTestId('tab-teachers')).toBeInTheDocument()
    expect(screen.getByTestId('tab-campuses')).toBeInTheDocument()
    expect(screen.getByTestId('tab-unavailabilities')).toBeInTheDocument()
    expect(screen.getByTestId('tab-deactivated')).toBeInTheDocument()

    // Status filter buttons
    expect(screen.getByTestId('filter-all')).toBeInTheDocument()
    expect(screen.getByTestId('filter-new')).toBeInTheDocument()
    expect(screen.getByTestId('filter-update')).toBeInTheDocument()

    // Switch to campuses tab
    fireEvent.click(screen.getByTestId('tab-campuses'))
    expect(await screen.findByText('CS1')).toBeInTheDocument()
    expect(screen.getByText('CS3')).toBeInTheDocument()

    // Switch to deactivated tab
    fireEvent.click(screen.getByTestId('tab-deactivated'))
    expect(await screen.findByText('Lê Văn Cũ')).toBeInTheDocument()

    // Click apply
    const applyBtn = screen.getByTestId('import-apply-btn')
    fireEvent.click(applyBtn)

    await waitFor(() => {
      expect(applySpy).toHaveBeenCalled()
      expect(onSuccess).toHaveBeenCalled()
    })
  })
})
