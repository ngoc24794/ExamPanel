import { describe, it, expect, beforeEach, vi } from 'vitest'
import { render, screen, waitFor, act } from '@testing-library/react'
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
    expect(screen.getByText('ExamPanel')).toBeInTheDocument()

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

    // Click on Teachers nav item
    const teachersButtons = screen.getAllByText(/Giáo viên|Teachers/i)
    await act(async () => {
      teachersButtons[0].click()
    })

    // Should display placeholder notice
    expect(screen.getAllByText(/Phase 1 Skeleton/i)[0]).toBeInTheDocument()
  })
})
