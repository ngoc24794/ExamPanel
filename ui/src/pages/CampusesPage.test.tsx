import { describe, it, expect, beforeEach, vi } from 'vitest'
import { render, screen, waitFor, fireEvent, within, act } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { CampusesPage } from './CampusesPage'
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
  return {
    ...render(
      <ThemeProvider>
        <QueryClientProvider client={testClient}>{ui}</QueryClientProvider>
      </ThemeProvider>,
    ),
    testClient,
  }
}

describe('CampusesPage', () => {
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

  it('renders the campuses list with codes, names, and color chips', async () => {
    renderWithClient(<CampusesPage />)

    await waitFor(() => {
      expect(screen.getByText('CS1')).toBeInTheDocument()
      expect(screen.getByText('CS2')).toBeInTheDocument()
      expect(screen.getByText('CS3')).toBeInTheDocument()
      expect(screen.getByText('CS4')).toBeInTheDocument()
    })

    // Active teachers count
    expect(screen.getAllByText(/giáo viên/i).length).toBeGreaterThan(0)
  })

  it('opens create dialog, validates inputs, and creates a new campus', async () => {
    const user = userEvent.setup()
    renderWithClient(<CampusesPage />)

    await waitFor(() => {
      expect(screen.getByText('CS1')).toBeInTheDocument()
    })

    // Click "Thêm phân hiệu" button
    const createBtn = screen.getByRole('button', { name: /Thêm phân hiệu|New Campus/i })
    await user.click(createBtn)

    const dialog = await screen.findByRole('dialog')
    const codeInput = within(dialog).getByPlaceholderText(/PH1|CS1/i)
    const nameInput = within(dialog).getByPlaceholderText(/Ba Đình|Central/i)
    fireEvent.change(codeInput, { target: { value: 'CS_TEST' } })
    fireEvent.change(nameInput, { target: { value: 'Phân hiệu Thử Nghiệm' } })

    const saveBtn = within(dialog).getByRole('button', { name: /Lưu|Save/i })
    await act(async () => {
      fireEvent.click(saveBtn)
    })

    await waitFor(() => {
      expect(screen.queryByRole('dialog')).not.toBeInTheDocument()
    })

    // Verify it appears in table
    await waitFor(() => {
      expect(screen.getByText('CS_TEST')).toBeInTheDocument()
      expect(screen.getAllByText('Phân hiệu Thử Nghiệm').length).toBeGreaterThan(0)
    })
  })

  it('shows campus_in_use advice when attempting to delete a campus with assigned teachers', async () => {
    renderWithClient(<CampusesPage />)

    await waitFor(() => {
      expect(screen.getByText('CS1')).toBeInTheDocument()
    })

    // Campus 1 (CS1) has assigned teachers (Nguyễn Văn An, etc.)
    const deleteButtons = screen.getAllByRole('button', { name: /Xóa|Delete/i })
    fireEvent.click(deleteButtons[0])

    // AlertDialog should be displayed
    await waitFor(() => {
      expect(screen.getByRole('alertdialog')).toBeInTheDocument()
    })

    // Confirm delete
    const confirmDeleteBtn = screen.getByRole('button', {
      name: /Xóa|Delete/i,
    })
    fireEvent.click(confirmDeleteBtn)

    // Should display campus_in_use error hint
    await waitFor(() => {
      expect(
        screen.getByText(/đang có giáo viên trực thuộc|assigned teachers/i),
      ).toBeInTheDocument()
    })
  })
})
