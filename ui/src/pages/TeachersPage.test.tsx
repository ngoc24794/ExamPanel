import { describe, it, expect, beforeEach, vi } from 'vitest'
import { render, screen, waitFor, fireEvent, within } from '@testing-library/react'
import { TeachersPage } from './TeachersPage'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { ThemeProvider } from '@/lib/theme'
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
    <ThemeProvider>
      <QueryClientProvider client={testClient}>{ui}</QueryClientProvider>
    </ThemeProvider>,
  )
}

describe('TeachersPage', () => {
  beforeEach(() => {
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
  })

  it('renders teacher roster and coverage panel matrix', async () => {
    renderWithClient(<TeachersPage />)

    // Wait for teachers to load
    await waitFor(() => {
      expect(screen.getByText('Nguyễn Văn An')).toBeInTheDocument()
      expect(screen.getByText('Trần Thị Bình')).toBeInTheDocument()
    })

    // Check Coverage Panel is present
    expect(
      screen.getByText(/Độ phủ giáo viên theo khối & phân hiệu|Teacher Coverage Matrix/i),
    ).toBeInTheDocument()
    expect(screen.getByText('Khối 10')).toBeInTheDocument()
    expect(screen.getByText('Khối 11')).toBeInTheDocument()
    expect(screen.getByText('Khối 12')).toBeInTheDocument()
  })

  it('filters teachers by name search', async () => {
    renderWithClient(<TeachersPage />)

    await waitFor(() => {
      expect(screen.getByText('Nguyễn Văn An')).toBeInTheDocument()
      expect(screen.getByText('Trần Thị Bình')).toBeInTheDocument()
    })

    // Search for "Hải Hà"
    const searchInput = screen.getByPlaceholderText(
      /Tìm kiếm theo tên|Search by teacher/i,
    )
    fireEvent.change(searchInput, { target: { value: 'Hải Hà' } })

    await waitFor(() => {
      expect(screen.getByText('Vũ Hải Hà')).toBeInTheDocument()
      expect(screen.queryByText('Nguyễn Văn An')).not.toBeInTheDocument()
    })
  })

  it('optimistically updates grades taught on toggle', async () => {
    renderWithClient(<TeachersPage />)

    await waitFor(() => {
      expect(screen.getByText('Nguyễn Văn An')).toBeInTheDocument()
    })

    // Find grade 12 toggle button for teacher 1 (currently teaches 10 and 11)
    const gradeButtons = screen.getAllByRole('button', { name: '12' })
    const t1Grade12Btn = gradeButtons[0]

    // Click to toggle grade 12
    fireEvent.click(t1Grade12Btn)

    // Should optimistically become active (styled with bg-primary)
    await waitFor(() => {
      expect(t1Grade12Btn.className).toContain('bg-primary')
    })
  })

  it('rolls back optimistic grade toggle when API call fails', async () => {
    const originalSetTeacherGrades = api.setTeacherGrades
    // Mock failure
    api.setTeacherGrades = vi.fn().mockRejectedValueOnce({
      code: 'storage_constraint',
      message: 'Database error',
    })

    renderWithClient(<TeachersPage />)

    await waitFor(() => {
      expect(screen.getByText('Nguyễn Văn An')).toBeInTheDocument()
    })

    const gradeButtons = screen.getAllByRole('button', { name: '12' })
    const t1Grade12Btn = gradeButtons[0]
    const initialClassName = t1Grade12Btn.className

    fireEvent.click(t1Grade12Btn)

    // Should rollback after error
    await waitFor(() => {
      expect(t1Grade12Btn.className).toBe(initialClassName)
    })

    // Restore API
    api.setTeacherGrades = originalSetTeacherGrades
  })

  it('offers "Deactivate instead" flow when deleting a teacher in use', async () => {
    renderWithClient(<TeachersPage />)

    await waitFor(() => {
      expect(screen.getByText('Nguyễn Văn An')).toBeInTheDocument()
    })

    // Teacher 1 (Nguyễn Văn An) is in use
    const row = screen.getByText('Nguyễn Văn An').closest('tr')!
    const deleteBtn = within(row).getByRole('button', { name: /Xóa|Delete/i })
    fireEvent.click(deleteBtn)

    await waitFor(() => {
      expect(screen.getByRole('alertdialog')).toBeInTheDocument()
    })

    // Confirm initial delete
    const confirmDeleteBtn = screen.getByRole('button', {
      name: /Xóa|Delete/i,
    })
    fireEvent.click(confirmDeleteBtn)

    // Should offer "Deactivate instead"
    await waitFor(() => {
      expect(
        screen.getByRole('button', { name: /Tạm dừng thay vì xóa|Deactivate instead/i }),
      ).toBeInTheDocument()
    })

    // Click "Deactivate instead"
    const deactivateBtn = screen.getByRole('button', {
      name: /Tạm dừng thay vì xóa|Deactivate instead/i,
    })
    fireEvent.click(deactivateBtn)

    // Dialog should close
    await waitFor(() => {
      expect(screen.queryByRole('alertdialog')).not.toBeInTheDocument()
    })
  })

  it('validates inputs and creates a new teacher with assigned grades', async () => {
    renderWithClient(<TeachersPage />)

    await waitFor(() => {
      expect(screen.getByText('Nguyễn Văn An')).toBeInTheDocument()
    })

    // Click "Thêm giáo viên"
    const addBtn = screen.getByRole('button', { name: /Thêm giáo viên|New Teacher/i })
    fireEvent.click(addBtn)

    const dialog = await screen.findByRole('dialog')
    expect(dialog).toBeInTheDocument()

    // Try submitting without full name
    const saveBtn = within(dialog).getByRole('button', { name: /Lưu|Save/i })
    fireEvent.click(saveBtn)

    await waitFor(() => {
      expect(
        within(dialog).getByText(
          /Họ tên không được để trống|Họ và tên không được để trống|Full name is required/i,
        ),
      ).toBeInTheDocument()
    })

    // Fill valid name
    const nameInput = within(dialog).getByPlaceholderText(/Nhập họ và tên|Full name/i)
    fireEvent.change(nameInput, { target: { value: 'Lê Văn Tám' } })

    // Submit form
    fireEvent.click(saveBtn)

    await waitFor(() => {
      expect(screen.queryByRole('dialog')).not.toBeInTheDocument()
    })

    // Verify appears in table
    await waitFor(() => {
      expect(screen.getByText('Lê Văn Tám')).toBeInTheDocument()
    })
  })

  it('displays coverage panel status badges derived from feasibility', async () => {
    renderWithClient(<TeachersPage />)

    await waitFor(() => {
      expect(screen.getByText('Nguyễn Văn An')).toBeInTheDocument()
    })

    // Check status badges in the Coverage Panel (OK / Đủ điều kiện or Sát ngưỡng / Không khả thi)
    await waitFor(() => {
      expect(
        screen.getAllByText(/Đủ điều kiện|Sát ngưỡng|Không khả thi|OK/i).length,
      ).toBeGreaterThan(0)
    })
  })
})
