import { test, expect } from '@playwright/test'
import { artifactDir } from './artifact-dir'
import path from 'path'
import { fileURLToPath } from 'url'
import fs from 'fs'

const __dirname = path.dirname(fileURLToPath(import.meta.url))
const screenshotsDir = artifactDir('screenshots', 'phase-8')

test.beforeAll(() => {
  if (!fs.existsSync(screenshotsDir)) {
    fs.mkdirSync(screenshotsDir, { recursive: true })
  }
})

test.describe('Phase 8 E2E Full Workflow & Visual Verification', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/#/')
    await page.waitForLoadState('networkidle')
  })

  test('full phase 8 workflow: run optimize → compare → duplicate → keyboard replace → undo → save → statistics', async ({ page }) => {
    // 1. Navigate to Assignments workspace
    await page.goto('/#/assignments')
    await page.waitForLoadState('networkidle')
    await expect(page.getByTestId('assignments-page')).toBeVisible()

    // 2. Open Run Optimizer dialog
    const runBtn = page.getByTestId('run-optimizer-button')
    await expect(runBtn).toBeVisible()
    await runBtn.click()

    const runDialog = page.getByRole('dialog')
    await expect(runDialog).toBeVisible()

    // Select "Nhanh" effort and start
    const fastBtn = runDialog.getByRole('button', { name: /Nhanh|Fast/i })
    await fastBtn.click()

    const startBtn = runDialog.getByTestId('start-optimize-button')
    await startBtn.click()

    // Wait for optimization to finish and dialog to close
    await expect(runDialog).not.toBeVisible({ timeout: 15000 })

    // 3. Compare plans
    const compareBtn = page.getByTestId('compare-plans-button')
    await expect(compareBtn).toBeEnabled()
    await compareBtn.click()

    const compareModal = page.getByTestId('plan-compare-modal')
    await expect(compareModal).toBeVisible()
    await page.keyboard.press('Escape')
    await expect(compareModal).not.toBeVisible()

    // 4. Duplicate plan for editing
    const createEditBtn = page.getByTestId('create-edit-copy-button')
    if (await createEditBtn.isVisible()) {
      await createEditBtn.click()
    } else {
      // Toggle history and duplicate first plan
      await page.getByTestId('history-plans-button').click()
      await expect(page.getByTestId('plans-history-list')).toBeVisible()
      const dupBtn = page.getByTestId('duplicate-plan-btn-1')
      if (await dupBtn.isVisible()) {
        await dupBtn.click()
      }
    }

    // Wait for editable mode: undo button appears
    const undoBtn = page.locator('button[title*="Hoàn tác"], button[title*="Undo"]')
    await expect(undoBtn).toBeVisible()

    // 5. Replace candidate in a slot
    const menuBtn = page.getByTestId('slot-menu-btn').first()
    if (await menuBtn.isVisible()) {
      await menuBtn.click({ force: true })
      const replaceMenuItem = page.locator('[role="menuitem"]').filter({ hasText: /Thay thế|Replace/i })
      if (await replaceMenuItem.isVisible()) {
        await replaceMenuItem.click()
        const candidateModal = page.getByTestId('candidate-select-modal')
        await expect(candidateModal).toBeVisible()

        // Wait for candidate items to load and select first valid candidate
        const validCand = candidateModal.locator('[data-testid="candidate-item-valid"]').first()
        await expect(validCand).toBeVisible()
        await validCand.click()
        await expect(candidateModal).not.toBeVisible()
      }
    }

    // 6. Undo
    if (await undoBtn.isEnabled()) {
      await undoBtn.click()
    }

    // 7. Save
    const saveBtn = page.locator('button[title*="Lưu"], button[title*="Save"]').or(page.getByTestId('save-assignments-btn'))
    if (await saveBtn.isVisible() && await saveBtn.isEnabled()) {
      await saveBtn.click()
    }

    // 8. Navigate to Statistics page
    await page.goto('/#/statistics')
    await page.waitForLoadState('networkidle')
    await expect(page.getByTestId('statistics-page')).toBeVisible()
    await expect(page.locator('table').first()).toBeVisible()
  })

  test('captures screenshots in light/dark × vi/en for all Phase 8 screens', async ({ page }) => {
    const themes: ('light' | 'dark')[] = ['light', 'dark']
    const languages: ('vi' | 'en')[] = ['vi', 'en']

    for (const lang of languages) {
      for (const theme of themes) {
        // Set language and theme cleanly in settings
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

        // 1. Plan View (Matrix)
        await page.goto('/#/assignments')
        await page.waitForLoadState('networkidle')
        await page.waitForTimeout(400)
        await page.screenshot({
          path: path.join(screenshotsDir, `plan-view-${theme}-${lang}.png`),
          fullPage: true,
        })

        // 2. Run Optimizer Dialog (with progress / options)
        await page.getByTestId('run-optimizer-button').click()
        await page.waitForTimeout(300)
        await page.screenshot({
          path: path.join(screenshotsDir, `run-progress-${theme}-${lang}.png`),
          fullPage: true,
        })
        await page.keyboard.press('Escape')
        await page.waitForTimeout(200)

        // 3. Plan Compare Modal
        const compareBtn = page.getByTestId('compare-plans-button')
        if (await compareBtn.isEnabled()) {
          await compareBtn.click()
          await page.waitForTimeout(500)
          await page.screenshot({
            path: path.join(screenshotsDir, `compare-${theme}-${lang}.png`),
            fullPage: true,
          })
          await page.keyboard.press('Escape')
          await page.waitForTimeout(200)
        }

        // 4. Edit Mode with Candidate List
        const createEditBtn = page.getByTestId('create-edit-copy-button')
        if (await createEditBtn.isVisible()) {
          await createEditBtn.click()
          await page.waitForTimeout(400)
        }
        const menuBtn = page.getByTestId('slot-menu-btn').first()
        if (await menuBtn.isVisible()) {
          await menuBtn.click({ force: true })
          const replaceMenuItem = page.locator('[role="menuitem"]').filter({ hasText: /Thay thế|Replace/i })
          if (await replaceMenuItem.isVisible()) {
            await replaceMenuItem.click()
            await page.waitForTimeout(400)
            await page.screenshot({
              path: path.join(screenshotsDir, `edit-mode-${theme}-${lang}.png`),
              fullPage: true,
            })
            await page.keyboard.press('Escape')
            await page.waitForTimeout(200)
          }
        }

        // 5. Statistics Page
        await page.goto('/#/statistics')
        await page.waitForLoadState('networkidle')
        await page.waitForTimeout(400)
        await page.screenshot({
          path: path.join(screenshotsDir, `statistics-${theme}-${lang}.png`),
          fullPage: true,
        })
      }
    }
  })
})
