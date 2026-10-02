import { test, expect } from '@playwright/test'
import path from 'path'
import { fileURLToPath } from 'url'
import fs from 'fs'

const __dirname = path.dirname(fileURLToPath(import.meta.url))
const screenshotsDir = path.resolve(__dirname, '../../docs/screenshots/phase-9')
const reportsDir = path.resolve(__dirname, '../../docs/reports/phase-9')

test.beforeAll(() => {
  if (!fs.existsSync(screenshotsDir)) {
    fs.mkdirSync(screenshotsDir, { recursive: true })
  }
  if (!fs.existsSync(reportsDir)) {
    fs.mkdirSync(reportsDir, { recursive: true })
  }
})

test.describe('Phase 9 E2E Full Workflow, PDF Output & Visual Verification', () => {
  test.beforeEach(async ({ page }) => {
    test.setTimeout(60000)
    await page.goto('/#/')
    await page.waitForLoadState('networkidle')
  })

  test('full phase 9 workflow: import wizard preview & apply, export/print menu, backup & restore in settings', async ({ page }) => {
    // 1. Teachers: Download template & Import Wizard
    await page.goto('/#/teachers')
    await page.waitForLoadState('networkidle')

    const downloadTplBtn = page.getByTestId('download-template-btn')
    await expect(downloadTplBtn).toBeVisible()

    const importBtn = page.getByTestId('import-excel-btn')
    await expect(importBtn).toBeVisible()
    await importBtn.click()

    // Import Wizard modal open
    const fileInput = page.getByTestId('import-file-input')
    await expect(fileInput).toBeVisible()
    await fileInput.fill('mau-nhap-du-lieu.xlsx')

    const syncCard = page.getByTestId('mode-sync-card')
    await syncCard.click()

    const previewBtn = page.getByTestId('import-run-preview-btn')
    await previewBtn.click()

    // Check preview tables and tabs
    await expect(page.getByTestId('tab-teachers')).toBeVisible()
    await expect(page.getByTestId('tab-campuses')).toBeVisible()
    await expect(page.getByTestId('tab-unavailabilities')).toBeVisible()
    await expect(page.getByTestId('tab-deactivated')).toBeVisible()

    // Filter statuses
    await page.getByTestId('filter-new').click()
    await page.getByTestId('filter-all').click()

    // Apply import
    const applyBtn = page.getByTestId('import-apply-btn')
    await applyBtn.click()
    await expect(page.getByRole('dialog')).not.toBeVisible({ timeout: 5000 })

    // 2. Assignments: Export button & Print menu
    await page.goto('/#/assignments')
    await page.waitForLoadState('networkidle')

    const exportBtn = page.getByTestId('export-excel-button')
    await expect(exportBtn).toBeVisible()

    const printMenuBtn = page.getByTestId('print-menu-button')
    await expect(printMenuBtn).toBeVisible()
    await printMenuBtn.click()

    await expect(page.getByTestId('print-plan-item')).toBeVisible()
    await expect(page.getByTestId('print-notices-item')).toBeVisible()
    await page.keyboard.press('Escape')

    // 3. Settings: Backup & Restore section
    await page.goto('/#/settings')
    await page.waitForLoadState('networkidle')

    const backupNowBtn = page.getByTestId('backup-now-btn')
    await expect(backupNowBtn).toBeVisible()

    const restoreFileBtn = page.getByTestId('restore-file-btn')
    await expect(restoreFileBtn).toBeVisible()

    // Click restore on the first automatic backup
    const restoreBtn = page.getByRole('button', { name: /Phục hồi/i }).first()
    if (await restoreBtn.isVisible()) {
      await restoreBtn.click()
      const confirmBtn = page.getByTestId('confirm-restore-btn')
      await expect(confirmBtn).toBeVisible()
      await page.keyboard.press('Escape')
    }
  })

  test('generate sample print PDFs: plan-print.pdf and notices.pdf', async ({ page }) => {
    // 1. Plan matrix print view
    await page.goto('/#/print/plan/1')
    await page.waitForLoadState('networkidle')
    await page.waitForTimeout(500)

    const planPdfPath = path.join(reportsDir, 'plan-print.pdf')
    await page.pdf({
      path: planPdfPath,
      format: 'A4',
      landscape: true,
      printBackground: true,
      margin: {
        top: '12mm',
        bottom: '12mm',
        left: '12mm',
        right: '12mm',
      },
    })
    expect(fs.existsSync(planPdfPath)).toBe(true)

    // 2. Notices print view
    await page.goto('/#/print/notices/1')
    await page.waitForLoadState('networkidle')
    await page.waitForTimeout(500)

    const noticesPdfPath = path.join(reportsDir, 'notices.pdf')
    await page.pdf({
      path: noticesPdfPath,
      format: 'A4',
      landscape: false,
      printBackground: true,
      margin: {
        top: '15mm',
        bottom: '15mm',
        left: '15mm',
        right: '15mm',
      },
    })
    expect(fs.existsSync(noticesPdfPath)).toBe(true)
  })

  test('capture light/dark x vi/en visual verification screenshots for Phase 9', async ({ page }) => {
    for (const theme of ['light', 'dark'] as const) {
      for (const lang of ['vi', 'en'] as const) {
        // Setup theme & language
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

        // 1. Import Wizard Preview
        await page.goto('/#/teachers')
        await page.waitForLoadState('networkidle')
        await page.getByTestId('import-excel-btn').click()
        await page.getByTestId('import-file-input').fill('mau-nhap-du-lieu.xlsx')
        await page.getByTestId('import-run-preview-btn').click()
        await page.waitForTimeout(400)
        await page.screenshot({
          path: path.join(screenshotsDir, `import-preview-${theme}-${lang}.png`),
          fullPage: true,
        })
        await page.keyboard.press('Escape')
        await page.waitForTimeout(200)

        // 2. Export & Print Menu in Assignments
        await page.goto('/#/assignments')
        await page.waitForLoadState('networkidle')
        await page.getByTestId('print-menu-button').click()
        await page.waitForTimeout(300)
        await page.screenshot({
          path: path.join(screenshotsDir, `export-print-menu-${theme}-${lang}.png`),
          fullPage: true,
        })
        await page.keyboard.press('Escape')
        await page.waitForTimeout(200)

        // 3. Settings Backup Section
        await page.goto('/#/settings')
        await page.waitForLoadState('networkidle')
        await page.waitForTimeout(400)
        await page.screenshot({
          path: path.join(screenshotsDir, `settings-backup-${theme}-${lang}.png`),
          fullPage: true,
        })
      }
    }
  })
})
