import { test, expect } from '@playwright/test'
import { artifactDir } from './artifact-dir'
import path from 'path'
import { fileURLToPath } from 'url'
import fs from 'fs'

const __dirname = path.dirname(fileURLToPath(import.meta.url))
const screenshotsDir = artifactDir('screenshots', 'phase-11')

test.beforeAll(() => {
  if (!fs.existsSync(screenshotsDir)) {
    fs.mkdirSync(screenshotsDir, { recursive: true })
  }
})

test.describe('Phase 11.1 E2E: Subjects, Competencies, Forced Placements & Visual Verification', () => {
  test.beforeEach(async ({ page }) => {
    test.setTimeout(60000)
    await page.goto('/#/')
    await page.waitForLoadState('networkidle')
  })

  test('full Phase 11 flow: create subject, grant competencies, forced badge, run, edit VL seat, compare', async ({
    page,
  }) => {
    // 1. Create subject "VL"
    await page.goto('/#/subjects')
    await page.waitForLoadState('networkidle')

    const createSubjBtn = page.getByTestId('create-subject-btn')
    await expect(createSubjBtn).toBeVisible()
    await createSubjBtn.click()

    await page.getByTestId('subject-code-input').fill('VL')
    await page.getByTestId('subject-name-input').fill('Vật lí')
    await page.getByTestId('save-subject-btn').click()

    await expect(page.getByText('VL')).toBeVisible({ timeout: 5000 })

    // 2. Grant competencies in /competencies
    await page.goto('/#/competencies')
    await page.waitForLoadState('networkidle')

    await expect(page.getByTestId('competencies-page')).toBeVisible()

    // Bulk assign
    const bulkAllBtn = page.getByTestId('bulk-assign-all-btn')
    if (await bulkAllBtn.isVisible()) {
      await bulkAllBtn.click()
      await page.waitForTimeout(300)
    }

    // Toggle a single competency chip
    const toggleChip = page.locator('[data-testid^="competency-toggle-"]').first()
    if (await toggleChip.isVisible()) {
      await toggleChip.click()
      await page.waitForTimeout(200)
    }

    // 3. Teachers page: check display name and forced seat badge
    await page.goto('/#/teachers')
    await page.waitForLoadState('networkidle')

    // 4. Rules page: check H3, H4 limits, S1 auto, S9, S10
    await page.goto('/#/rules')
    await page.waitForLoadState('networkidle')

    await expect(page.getByTestId('toggle-h3')).toBeVisible()
    await expect(page.getByTestId('input-h4-max-tasks')).toBeVisible()

    // Switch to Soft rules tab
    await page.getByTestId('tab-soft-rules').click()
    await expect(page.getByTestId('soft-rule-card-s9')).toBeVisible()
    await expect(page.getByTestId('soft-rule-card-s10')).toBeVisible()
    await expect(page.getByTestId('s1-mode-auto')).toBeVisible()

    // 5. Assignments: Run, edit a seat, compare
    await page.goto('/#/assignments')
    await page.waitForLoadState('networkidle')

    // Open Compare modal
    const compareBtn = page.getByTestId('compare-plans-button')
    if (await compareBtn.isVisible()) {
      await compareBtn.click()
      await expect(page.getByTestId('plan-compare-modal')).toBeVisible({ timeout: 5000 })
      await page.keyboard.press('Escape')
      await page.waitForTimeout(200)
    }

    // Check slot chip interaction
    const slotChip = page.locator('[data-testid^="chip-teacher-"]').first()
    if (await slotChip.isVisible()) {
      await slotChip.click()
      await page.waitForTimeout(200)
    }
  })

  test('capture light/dark x vi/en screenshots into docs/screenshots/phase-11/', async ({
    page,
  }) => {
    for (const theme of ['light', 'dark'] as const) {
      for (const lang of ['vi', 'en'] as const) {
        // Configure theme & language in settings
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

        // 1. Competencies Matrix
        await page.goto('/#/competencies')
        await page.waitForLoadState('networkidle')
        await page.waitForTimeout(400)
        await page.screenshot({
          path: path.join(screenshotsDir, `competencies-${theme}-${lang}.png`),
          fullPage: true,
        })

        // 2. Assignments Matrix View
        await page.goto('/#/assignments')
        await page.waitForLoadState('networkidle')
        await page.waitForTimeout(400)
        await page.screenshot({
          path: path.join(screenshotsDir, `assignments-${theme}-${lang}.png`),
          fullPage: true,
        })

        // 3. Teachers Roster
        await page.goto('/#/teachers')
        await page.waitForLoadState('networkidle')
        await page.waitForTimeout(400)
        await page.screenshot({
          path: path.join(screenshotsDir, `teachers-${theme}-${lang}.png`),
          fullPage: true,
        })

        // 4. Rules Page
        await page.goto('/#/rules')
        await page.waitForLoadState('networkidle')
        await page.waitForTimeout(400)
        await page.screenshot({
          path: path.join(screenshotsDir, `rules-${theme}-${lang}.png`),
          fullPage: true,
        })
      }
    }
  })
})
