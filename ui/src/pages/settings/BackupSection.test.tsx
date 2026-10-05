import { render, screen, fireEvent, waitFor } from '@testing-library/react'
import { describe, it, expect, vi, beforeEach } from 'vitest'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { BackupSection } from './BackupSection'
import { mockApi } from '@/lib/api/mock'
import '@/i18n'

describe('BackupSection Component Tests', () => {
  let queryClient: QueryClient

  beforeEach(() => {
    queryClient = new QueryClient({
      defaultOptions: {
        queries: { retry: false },
      },
    })
    vi.clearAllMocks()
  })

  it('renders backup actions and lists automatic backups', async () => {
    render(
      <QueryClientProvider client={queryClient}>
        <BackupSection />
      </QueryClientProvider>,
    )

    expect(screen.getByTestId('backup-now-btn')).toBeInTheDocument()
    expect(screen.getByTestId('restore-file-btn')).toBeInTheDocument()

    // Wait for backups to load from mockApi
    expect(
      await screen.findByText('exampanel-backup-20261001-1400.db'),
    ).toBeInTheDocument()
  })

  it('validates backup and executes restore flow on confirm', async () => {
    const validateSpy = vi.spyOn(mockApi, 'validateBackup')
    const restoreSpy = vi.spyOn(mockApi, 'restoreDatabase')

    render(
      <QueryClientProvider client={queryClient}>
        <BackupSection />
      </QueryClientProvider>,
    )

    expect(
      await screen.findByText('exampanel-backup-20261001-1400.db'),
    ).toBeInTheDocument()

    // Click "Phục hồi" on the first backup row
    const restoreButtons = screen.getAllByRole('button', { name: /Phục hồi/i })
    fireEvent.click(restoreButtons[0])

    await waitFor(() => {
      expect(validateSpy).toHaveBeenCalled()
    })

    // Dialog should display validation summary
    expect(await screen.findByText(/Tệp dữ liệu hợp lệ/i)).toBeInTheDocument()
    expect(screen.getByText('Năm học:')).toBeInTheDocument()

    // Confirm restore
    const dispatchSpy = vi.spyOn(window, 'dispatchEvent')
    const confirmBtn = screen.getByTestId('confirm-restore-btn')
    fireEvent.click(confirmBtn)

    await waitFor(() => {
      expect(restoreSpy).toHaveBeenCalled()
      expect(dispatchSpy).toHaveBeenCalledWith(
        expect.objectContaining({ type: 'exampanel:restore' }),
      )
    })
  })
})

describe('BackupSection restore dialog errors (RA-032)', () => {
  it('shows a Vietnamese reason, not the raw English engine text, for a newer-version file', async () => {
    const queryClient = new QueryClient({ defaultOptions: { queries: { retry: false } } })
    vi.spyOn(mockApi, 'validateBackup').mockResolvedValue({
      valid: false,
      user_version: 99,
      school_years_count: 0,
      teachers_count: 0,
      plans_count: 0,
      error: 'Unsupported future database version: 99 > supported 5',
      error_code: 'newer_version',
      supported_version: 5,
    })
    render(
      <QueryClientProvider client={queryClient}>
        <BackupSection />
      </QueryClientProvider>,
    )
    await screen.findByText('exampanel-backup-20261001-1400.db')
    fireEvent.click(screen.getAllByRole('button', { name: /Phục hồi/i })[0])

    expect(await screen.findByText(/mới hơn bản ExamPanel này/)).toBeInTheDocument()
    expect(screen.getByText(/phiên bản ExamPanel mới hơn/i)).toBeInTheDocument()
    expect(screen.queryByText(/Unsupported future database version/)).not.toBeInTheDocument()
    expect(screen.queryByText(/hỏng hoặc không đúng định dạng/)).not.toBeInTheDocument()
    expect(screen.getByTestId('confirm-restore-btn')).toBeDisabled()
  })
})
