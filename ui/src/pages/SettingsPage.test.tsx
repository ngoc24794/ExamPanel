import * as React from 'react'
import { render, screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, expect, it } from 'vitest'
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
})
