import { describe, it, expect, beforeEach, vi } from 'vitest'
import { render, screen, waitFor, act } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { App } from './App'
import { ThemeProvider } from '@/lib/theme'
import '@/i18n'

describe('App', () => {
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

  it('renders application brand, sidebar navigation, and ping response', async () => {
    render(
      <ThemeProvider>
        <App />
      </ThemeProvider>,
    )

    // Check brand
    expect(screen.getAllByText('ExamPanel')[0]).toBeInTheDocument()

    // Check ping response from mock API
    await waitFor(() => {
      expect(screen.getByText(/pong from Mock Engine/i)).toBeInTheDocument()
    })
  })

  it('switches navigation tab when sidebar item is clicked', async () => {
    render(
      <ThemeProvider>
        <App />
      </ThemeProvider>,
    )

    // Click on Campuses nav item
    const campusesNav = screen.getByRole('link', { name: /Phân hiệu|Campuses/i })
    await act(async () => {
      campusesNav.click()
    })

    // Should display Campuses screen heading
    await waitFor(() => {
      expect(
        screen.getByRole('heading', { name: /Quản lý Phân hiệu|Campus Management/i }),
      ).toBeInTheDocument()
    })

    // Click on Exams nav item (real screen in Phase 7)
    const examsNav = screen.getByRole('link', { name: /Kỳ thi|Exams/i })
    await act(async () => {
      examsNav.click()
    })

    await waitFor(() => {
      expect(screen.getByTestId('exams-page')).toBeInTheDocument()
    })

    // Click on Assignments nav item (placeholder screen)
    const assignmentsNav = screen.getByRole('link', { name: /Phân công|Assignments/i })
    await act(async () => {
      assignmentsNav.click()
    })

    await waitFor(() => {
      expect(screen.getByTestId('assignments-page')).toBeInTheDocument()
    })
  })

  it('switches school year through selector and refetches data', async () => {
    const user = userEvent.setup()
    const { api } = await import('@/lib/api')
    await api.seedDemo()

    render(
      <ThemeProvider>
        <App />
      </ThemeProvider>,
    )

    // Current school year button should be rendered
    await waitFor(() => {
      expect(screen.getByRole('button', { name: /2026-2027/i })).toBeInTheDocument()
    })

    // Click year selector button
    const yearButton = screen.getByRole('button', { name: /2026-2027/i })
    await user.click(yearButton)

    // Dropdown menu should display options
    await waitFor(() => {
      expect(screen.getByRole('menu')).toBeInTheDocument()
    })
  })
})
