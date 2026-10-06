import { test, expect } from '@playwright/test'
import { artifactDir } from './artifact-dir'
import path from 'path'
import { fileURLToPath } from 'url'
import fs from 'fs'

const __dirname = path.dirname(fileURLToPath(import.meta.url))
const screenshotsDir = artifactDir('screenshots', 'phase-7')

test.beforeAll(() => {
  if (!fs.existsSync(screenshotsDir)) {
    fs.mkdirSync(screenshotsDir, { recursive: true })
  }
})

test.describe('Phase 7 E2E Full Workflow & Visual Verification', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/#/')
    await page.waitForLoadState('networkidle')
  })

  test('full phase 7 workflow: unavailability, indicator, feasibility sheet, go to fix, locks, and presets', async ({ page }) => {
    // 1. Navigate to Unavailability grid
    await page.goto('/#/unavailability')
    await page.waitForLoadState('networkidle')
    await expect(page.getByTestId('unavailability-grid')).toBeVisible()

    // 2. Mark teacher 1 unavailable for Semester 1
    const bulkBtn = page.getByTestId('bulk-actions-1')
    await bulkBtn.click()
    const sem1Action = page.getByTestId('bulk-absent-sem1-1')
    await sem1Action.click()

    // Verify absence cell is marked
    const cell = page.getByTestId('unavail-cell-1-1')
    await expect(cell).toContainText(/Vắng|Unavailable/i)

    // 3. Observe feasibility indicator
    const indicator = page.getByTestId('feasibility-indicator')
    await expect(indicator).toBeVisible()

    // 4. Click indicator to open Feasibility Sheet
    await indicator.click()
    const sheet = page.getByTestId('feasibility-sheet')
    await expect(sheet).toBeVisible()

    // 5. Follow "Go to fix"
    const fixBtn = page.getByTestId('go-to-fix-btn').first()
    if (await fixBtn.isVisible()) {
      await fixBtn.click()
      await expect(sheet).not.toBeVisible()
    } else {
      // Close sheet by pressing Escape or clicking outside
      await page.keyboard.press('Escape')
    }

    // 6. Navigate to Rules screen & Locks tab
    await page.goto('/#/rules?tab=locks')
    await page.waitForLoadState('networkidle')
    await expect(page.getByTestId('locks-section')).toBeVisible()

    // Create a Pin lock
    await page.getByTestId('add-lock-btn').click()
    const dialog = page.getByRole('dialog')
    await expect(dialog).toBeVisible()

    await page.getByTestId('lock-teacher-select').click()
    // Select first enabled teacher
    await page.locator('[role="option"]:not([aria-disabled="true"])').first().click()
    await page.getByTestId('lock-submit-btn').click()
    await expect(dialog).not.toBeVisible()

    // Create a Forbid lock
    await page.getByTestId('add-lock-btn').click()
    await expect(dialog).toBeVisible()
    await page.getByTestId('lock-kind-select').click()
    await page.getByRole('option', { name: /Cấm|Forbid/i }).click()
    await page.getByTestId('lock-teacher-select').click()
    await page.locator('[role="option"]:not([aria-disabled="true"])').first().click()
    await page.getByTestId('lock-submit-btn').click()
    await expect(dialog).not.toBeVisible()

    // 7. Go to Soft rules tab and apply a Preset
    await page.getByTestId('tab-soft-rules').click()
    await expect(page.getByTestId('soft-rules-section')).toBeVisible()

    await page.getByTestId('preset-workload-btn').click()
    const saveBtn = page.getByTestId('save-rules-btn')
    await expect(saveBtn).toBeEnabled()
    await saveBtn.click()
    await page.waitForTimeout(500)
  })

  test('captures screenshots in light/dark × vi/en for all Phase 7 screens and feasibility sheet', async ({ page }) => {
    const themes: ('light' | 'dark')[] = ['light', 'dark']
    const languages: ('vi' | 'en')[] = ['vi', 'en']

    for (const lang of languages) {
      for (const theme of themes) {
        // Navigate to settings to set language and theme cleanly
        await page.goto('/#/settings')
        await page.waitForLoadState('networkidle')

        if (lang === 'vi') {
          await page.getByTestId('lang-vi-btn').click()
        } else {
          await page.getByTestId('lang-en-btn').click()
        }
        await page.waitForTimeout(200)

        if (theme === 'light') {
          await page.getByTestId('theme-light-btn').click()
        } else {
          await page.getByTestId('theme-dark-btn').click()
        }
        await page.waitForTimeout(300)

        // 1. Exams Screen
        await page.goto('/#/exams')
        await page.waitForLoadState('networkidle')
        await page.waitForTimeout(300)
        await page.screenshot({
          path: path.join(screenshotsDir, `exams-${theme}-${lang}.png`),
          fullPage: true,
        })

        // 2. Unavailability Screen
        await page.goto('/#/unavailability')
        await page.waitForLoadState('networkidle')
        await page.waitForTimeout(300)
        await page.screenshot({
          path: path.join(screenshotsDir, `unavailability-${theme}-${lang}.png`),
          fullPage: true,
        })

        // 3. Rules Screen (Soft rules tab)
        await page.goto('/#/rules?tab=soft')
        await page.waitForLoadState('networkidle')
        await page.waitForTimeout(300)
        await page.screenshot({
          path: path.join(screenshotsDir, `rules-${theme}-${lang}.png`),
          fullPage: true,
        })

        // 4. Locks Tab
        await page.goto('/#/rules?tab=locks')
        await page.waitForLoadState('networkidle')
        await page.waitForTimeout(300)
        await page.screenshot({
          path: path.join(screenshotsDir, `locks-${theme}-${lang}.png`),
          fullPage: true,
        })

        // 5. Settings Screen
        await page.goto('/#/settings')
        await page.waitForLoadState('networkidle')
        await page.waitForTimeout(300)
        await page.screenshot({
          path: path.join(screenshotsDir, `settings-${theme}-${lang}.png`),
          fullPage: true,
        })

        // 6. Feasibility Sheet
        await page.getByTestId('feasibility-indicator').click()
        await page.waitForTimeout(400)
        await page.screenshot({
          path: path.join(screenshotsDir, `feasibility-sheet-${theme}-${lang}.png`),
          fullPage: true,
        })
        await page.keyboard.press('Escape')
        await page.waitForTimeout(200)
      }
    }
  })
})
