import { test, expect } from '@playwright/test'
import path from 'path'
import { fileURLToPath } from 'url'
import fs from 'fs'

const __dirname = path.dirname(fileURLToPath(import.meta.url))
const screenshotsDir = path.resolve(__dirname, '../../docs/screenshots/phase-12')

const qTableTsv =
  "Kì thi/khối\t\tKhối 10\t\tKhối 11\t\tKhối 12\t\n" +
  "\t\tVL\tCN\tVL\tCN\tVL\tCN\n" +
  "GK1\tĐề\tC Hiền\tT Nghĩa\tT Lộc\tT Nghĩa\tC Bình\tT Nghĩa\n" +
  "\tĐề\tC Lài\t\tC Thư\t\tT Phúc\t\n" +
  "\tP.Biện\tT Phúc\tC Hiền\tC Na\tC Lài\tC Quí\tT Lộc\n" +
  "CK1\tĐề\tC Hiền\tT Nghĩa\tC Lài\tT Nghĩa\tC Lan\tT Nghĩa\n" +
  "\tĐề\tC Tú\t\tC Bình\t\tC Quí\t\n" +
  "\tP.Biện\tC Như\tC Tú\tC Thư\tC Lan\tC Lài\tC Bình\n" +
  "GK2\tĐề\tC Tú\tT Nghĩa\tC Na\tT Nghĩa\tC Lan\tT Nghĩa\n" +
  "\tĐề\tC Như\t\tC Thư\t\tC Lài\t\n" +
  "\tP.Biện\tT Lộc\tC Như\tC Hiền\tC Thư\tC Quí\tT Phúc\n" +
  "CK2\tĐề\tC Na\tT Nghĩa\tT Lộc\tT Nghĩa\tT Phúc\tT Nghĩa\n" +
  "\tĐề\tC Như\t\tC Quí\t\tC Hiền\t\n" +
  "\tP.Biện\tC Tú\tC Quí\tC Bình\tC Na\tC Lan\tC Quí"

test.beforeAll(() => {
  if (!fs.existsSync(screenshotsDir)) {
    fs.mkdirSync(screenshotsDir, { recursive: true })
  }
})

test.describe('Phase 12 E2E: Q Plan Import, Grid Editing, Comparison & Visuals', () => {
  test.beforeEach(async ({ page }) => {
    test.setTimeout(60000)
    await page.goto('/#/')
    await page.waitForLoadState('networkidle')
  })

  test('flow: import Q table -> compare with optimizer plan -> edit seat in grid -> export', async ({
    page,
  }) => {
    await page.goto('/#/assignments')
    await page.waitForLoadState('networkidle')

    // 1. Open Import Modal
    const importBtn = page.getByTestId('import-plan-button')
    await expect(importBtn).toBeVisible()
    await importBtn.click()

    await expect(page.getByTestId('import-plan-modal')).toBeVisible()

    // Switch to TSV tab
    const tsvTab = page.getByTestId('tab-source-tsv')
    await tsvTab.click()

    // Paste Q table TSV
    const tsvTextarea = page.getByTestId('textarea-plan-tsv')
    await tsvTextarea.fill(qTableTsv)

    // Click Preview
    const previewBtn = page.getByTestId('btn-plan-import-preview')
    await previewBtn.click()

    // Wait for preview step
    const applyBtn = page.getByTestId('btn-plan-import-apply')
    await expect(applyBtn).toBeVisible({ timeout: 5000 })
    await expect(applyBtn).toBeEnabled()

    // Apply import
    await applyBtn.click()

    // Modal closes
    await expect(page.getByTestId('import-plan-modal')).not.toBeVisible({ timeout: 5000 })

    // 2. Compare with optimizer plan
    const compareBtn = page.getByTestId('compare-plans-button')
    await expect(compareBtn).toBeVisible()
    await compareBtn.click()

    const compareModal = page.getByTestId('plan-compare-modal')
    await expect(compareModal).toBeVisible({ timeout: 5000 })
    await page.keyboard.press('Escape')
    await page.waitForTimeout(300)

    // 3. Edit a seat in the grid (imported plan has source='manual' so is editable)
    const editableCell = page.locator('[data-testid^="q-grid-cell-"]').first()
    await expect(editableCell).toBeVisible()
    await editableCell.click()

    // Candidate modal opens
    const candidateModal = page.getByTestId('candidate-select-modal')
    if (await candidateModal.isVisible()) {
      // Pick first valid candidate button if available
      const validCandidate = page.locator('[data-testid="candidate-item-valid"]').first()
      if (await validCandidate.isVisible()) {
        const selectBtn = validCandidate.getByRole('button')
        if (await selectBtn.isVisible()) {
          await selectBtn.click()
        }
      } else {
        await page.keyboard.press('Escape')
      }
    }

    // 4. Export Menu
    const printMenuBtn = page.getByTestId('print-menu-button')
    await expect(printMenuBtn).toBeVisible()
    await printMenuBtn.click()
    await expect(page.getByTestId('print-plan-item')).toBeVisible()
    await page.keyboard.press('Escape')
  })

  test('capture light/dark x vi/en screenshots into docs/screenshots/phase-12/', async ({
    page,
  }) => {
    test.setTimeout(120000)
    for (const theme of ['light', 'dark'] as const) {
      for (const lang of ['vi', 'en'] as const) {
        // Set theme & language
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

        // Navigate to Assignments page
        await page.goto('/#/assignments')
        await page.waitForLoadState('networkidle')
        await page.waitForTimeout(500)

        // 1. Grid View Screenshot
        const gridRoot = page.getByTestId('q-plan-grid-root')
        if (await gridRoot.isVisible()) {
          await gridRoot.screenshot({
            path: path.join(screenshotsDir, `grid-view-${theme}-${lang}.png`),
          })
        }

        // 2. Totals Panel Screenshot
        const totalsPanel = page.getByTestId('q-plan-totals-panel')
        if (await totalsPanel.isVisible()) {
          await totalsPanel.screenshot({
            path: path.join(screenshotsDir, `totals-panel-${theme}-${lang}.png`),
          })
        }

        // 3. Export Menu Screenshot
        const printMenuBtn = page.getByTestId('print-menu-button')
        if (await printMenuBtn.isVisible()) {
          await printMenuBtn.click()
          await page.waitForTimeout(300)
          await page.screenshot({
            path: path.join(screenshotsDir, `export-menu-${theme}-${lang}.png`),
            fullPage: false,
          })
          await page.keyboard.press('Escape')
          await page.waitForTimeout(200)
        }

        // 4. Import Preview Screenshot
        const importBtn = page.getByTestId('import-plan-button')
        if (await importBtn.isVisible()) {
          await importBtn.click()
          await page.waitForTimeout(300)
          await page.getByTestId('tab-source-tsv').click()
          await page.getByTestId('textarea-plan-tsv').fill(qTableTsv)
          await page.getByTestId('btn-plan-import-preview').click()
          await page.waitForTimeout(500)
          const importModal = page.getByTestId('import-plan-modal')
          if (await importModal.isVisible()) {
            await importModal.screenshot({
              path: path.join(screenshotsDir, `import-preview-${theme}-${lang}.png`),
            })
          }
          await page.keyboard.press('Escape')
          await page.waitForTimeout(200)
        }
      }
    }
  })
})
