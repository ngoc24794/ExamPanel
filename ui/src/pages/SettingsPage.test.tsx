import * as React from 'react'
import { render, screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, expect, it, vi } from 'vitest'
import { api } from '@/lib/api'
import i18n from '@/i18n'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { HashRouter } from 'react-router-dom'
import { SettingsPage } from './SettingsPage'
import { ThemeProvider } from '@/lib/theme'

function renderWithProviders(ui: React.ReactElement) {
  const queryClient = new QueryClient({
    defaultOptions: {
      queries: { retry: false },
    },
  })
  return render(
    <QueryClientProvider client={queryClient}>
      <ThemeProvider>
        <HashRouter>{ui}</HashRouter>
      </ThemeProvider>
    </QueryClientProvider>,
  )
}

describe('SettingsPage', () => {
  it('renders settings page, data path, and copies path to clipboard', async () => {
    const user = userEvent.setup()
    renderWithProviders(<SettingsPage />)

    await waitFor(() => {
      expect(screen.getByTestId('settings-page')).toBeInTheDocument()
      expect(screen.getByTestId('data-path-display')).toBeInTheDocument()
      expect(screen.getByTestId('copy-path-btn')).toBeInTheDocument()
    })

    // Click copy path
    await user.click(screen.getByTestId('copy-path-btn'))
    await waitFor(async () => {
      expect(await navigator.clipboard.readText()).toBe('/mock/data')
    })
  })

  it('allows switching theme and language', async () => {
    const user = userEvent.setup()
    renderWithProviders(<SettingsPage />)

    await waitFor(() => {
      expect(screen.getByTestId('theme-dark-btn')).toBeInTheDocument()
      expect(screen.getByTestId('lang-en-btn')).toBeInTheDocument()
    })

    // Click Dark theme button
    await user.click(screen.getByTestId('theme-dark-btn'))

    // Click English language button
    await user.click(screen.getByTestId('lang-en-btn'))
  })

  // RA-033: the Settings buttons only switched the live i18n language and never persisted it.
  it('persists the language chosen on the Settings page', async () => {
    const user = userEvent.setup()
    const persist = vi.spyOn(api, 'setLanguage')
    renderWithProviders(<SettingsPage />)

    await user.click(await screen.findByTestId('lang-en-btn'))
    await waitFor(() => expect(persist).toHaveBeenCalledWith('en'))
    expect(localStorage.getItem('exampanel_language')).toBe('en')
    expect(await api.getLanguage()).toBe('en')

    await user.click(screen.getByTestId('lang-vi-btn'))
    await waitFor(() => expect(persist).toHaveBeenLastCalledWith('vi'))
    expect(i18n.language).toBe('vi')
    persist.mockRestore()
  })

  it('renders organization section and saves organization info', async () => {
    const user = userEvent.setup()
    renderWithProviders(<SettingsPage />)

    await waitFor(() => {
      expect(screen.getByTestId('org-info-card')).toBeInTheDocument()
      expect(screen.getByTestId('org-school-name-input')).toBeInTheDocument()
      expect(screen.getByTestId('save-org-info-btn')).toBeInTheDocument()
    })

    const schoolInput = screen.getByTestId('org-school-name-input')
    await user.clear(schoolInput)
    await user.type(schoolInput, 'THPT Chuyên Hà Nội')

    const deptInput = screen.getByTestId('org-department-name-input')
    await user.clear(deptInput)
    await user.type(deptInput, 'Tổ Toán')

    await user.click(screen.getByTestId('save-org-info-btn'))

    await waitFor(() => {
      expect(schoolInput).toHaveValue('THPT Chuyên Hà Nội')
      expect(deptInput).toHaveValue('Tổ Toán')
    })
  })

  it('renders Giới thiệu (About) section with version, commit hash, and build date', async () => {
    renderWithProviders(<SettingsPage />)

    await waitFor(() => {
      expect(screen.getByTestId('about-section')).toBeInTheDocument()
      expect(screen.getByTestId('about-version')).toHaveTextContent('v0.1.0')
      expect(screen.getByTestId('about-commit')).toHaveTextContent('0c24c8e')
      expect(screen.getByTestId('about-build-date')).toHaveTextContent('2026-10-02')
      expect(screen.getByTestId('licenses-dialog-trigger')).toBeInTheDocument()
    })
  })

  it('renders open logs button and handles click', async () => {
    const user = userEvent.setup()
    renderWithProviders(<SettingsPage />)

    await waitFor(() => {
      expect(screen.getByTestId('open-logs-btn')).toBeInTheDocument()
    })

    await user.click(screen.getByTestId('open-logs-btn'))
  })

  it('renders trial mode card and supports entering/exiting trial mode', async () => {
    const user = userEvent.setup()
    renderWithProviders(<SettingsPage />)

    await waitFor(() => {
      expect(screen.getByTestId('trial-mode-section')).toBeInTheDocument()
      expect(screen.getByTestId('enter-trial-mode-btn')).toBeInTheDocument()
    })

    // Enter trial mode
    await user.click(screen.getByTestId('enter-trial-mode-btn'))

    await waitFor(() => {
      expect(screen.getByTestId('exit-trial-mode-btn')).toBeInTheDocument()
    })

    // Exit trial mode
    await user.click(screen.getByTestId('exit-trial-mode-btn'))

    await waitFor(() => {
      expect(screen.getByTestId('enter-trial-mode-btn')).toBeInTheDocument()
    })
  })
})
