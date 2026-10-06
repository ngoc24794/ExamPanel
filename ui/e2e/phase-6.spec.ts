import { test, expect } from '@playwright/test'
import { artifactDir } from './artifact-dir'
import path from 'path'
import { fileURLToPath } from 'url'

const __dirname = path.dirname(fileURLToPath(import.meta.url))
const screenshotsDir = artifactDir('screenshots', 'phase-6')

test.describe('Phase 6 E2E & Visual Verification', () => {
  test.beforeEach(async ({ page }) => {
    // Navigate to base URL
    await page.goto('/#/')
    await page.waitForLoadState('networkidle')
  })

  test('navigates all screens and verifies placeholders & features', async ({ page }) => {
    // 1. Overview
    await expect(page.locator('h1, h2, h3').filter({ hasText: /Bảng điều khiển tổng quan|Overview/i })).toBeVisible()

    // 2. Campuses
    await page.click('a[href="#/campuses"]')
    await expect(page.getByRole('heading', { name: /Quản lý Phân hiệu|Campuses/i })).toBeVisible()
    await expect(page.getByText('CS1')).toBeVisible()

    // 3. Teachers
    await page.click('a[href="#/teachers"]')
    await expect(page.getByRole('heading', { name: /Quản lý Giáo viên|Teachers/i })).toBeVisible()
    await expect(page.getByText('Nguyễn Văn An')).toBeVisible()

    // 4. Exams
    await page.click('a[href="#/exams"]')
    await expect(page.getByText(/Kỳ thi|Exams/i).first()).toBeVisible()

    // 5. Rules
    await page.click('a[href="#/rules"]')
    await expect(page.getByText(/Tiêu chí|Rules/i).first()).toBeVisible()

    // 6. Assignments
    await page.click('a[href="#/assignments"]')
    await expect(page.getByText(/Phân công|Assignments/i).first()).toBeVisible()

    // 7. Statistics
    await page.click('a[href*="/stat"]')
    await expect(page.getByText(/Thống kê|Statistics/i).first()).toBeVisible()

    // 8. Settings
    await page.click('a[href="#/settings"]')
    await expect(page.getByText(/Cài đặt|Settings/i).first()).toBeVisible()
  })

  test('performs campus CRUD, teacher CRUD, and optimistic grade toggle', async ({ page }) => {
    // Navigate to Campuses
    await page.goto('/#/campuses')
    await page.waitForLoadState('networkidle')

    // Create Campus
    await page.getByRole('button', { name: /Thêm phân hiệu|New Campus/i }).click()
    const campusDialog = page.getByRole('dialog')
    await expect(campusDialog).toBeVisible()

    await campusDialog.locator('input[name="code"]').fill('CS_E2E')
    await campusDialog.locator('input[name="name"]').fill('Phân hiệu E2E Playwright')
    await campusDialog.getByRole('button', { name: /Lưu|Save/i }).click()
    await expect(campusDialog).not.toBeVisible()

    // Verify campus in table
    await expect(page.getByText('CS_E2E')).toBeVisible()
    await expect(page.getByText('Phân hiệu E2E Playwright').first()).toBeVisible()

    // Navigate to Teachers
    await page.goto('/#/teachers')
    await page.waitForLoadState('networkidle')

    // Create Teacher
    await page.getByRole('button', { name: /Thêm giáo viên|New Teacher/i }).click()
    const teacherDialog = page.getByRole('dialog')
    await expect(teacherDialog).toBeVisible()

    await teacherDialog.locator('input[name="fullName"]').fill('Đoàn Văn E2E')
    await teacherDialog.getByRole('button', { name: /Lưu|Save/i }).click()
    await expect(teacherDialog).not.toBeVisible()

    // Verify teacher in table
    await expect(page.getByText('Đoàn Văn E2E')).toBeVisible()

    // Optimistic grade toggle: click grade toggle chip
    const teacherRow = page.locator('tr').filter({ hasText: 'Đoàn Văn E2E' })
    const gradeChip = teacherRow.getByRole('button', { name: '10' })
    await gradeChip.click()

    // Should become active (bg-primary class)
    await expect(gradeChip).toHaveClass(/bg-primary/)
  })

  test('captures screenshots in light/dark × vi/en for Campuses and Teachers screens', async ({ page }) => {
    const themes: ('light' | 'dark')[] = ['light', 'dark']
    const languages: ('vi' | 'en')[] = ['vi', 'en']

    for (const lang of languages) {
      // Set language via header
      await page.goto('/#/campuses')
      await page.waitForLoadState('networkidle')

      // Switch language
      const langBtn = page.getByRole('button', { name: /Ngôn ngữ|Language/i })
      await langBtn.click()
      if (lang === 'vi') {
        await page.getByText(/Tiếng Việt/i).click()
      } else {
        await page.getByText(/English/i).click()
      }
      await page.waitForTimeout(300)

      for (const theme of themes) {
        // Switch theme
        const themeBtn = page.getByRole('button', { name: /Giao diện|Theme/i })
        await themeBtn.click()
        if (theme === 'light') {
          await page.getByText(/Sáng|Light/i).click()
        } else {
          await page.getByText(/Tối|Dark/i).click()
        }
        await page.waitForTimeout(400)

        // Capture Campuses screen
        await page.goto('/#/campuses')
        await page.waitForLoadState('networkidle')
        await page.waitForTimeout(300)
        await page.screenshot({
          path: path.join(screenshotsDir, `campuses-${theme}-${lang}.png`),
          fullPage: true,
        })

        // Capture Teachers screen
        await page.goto('/#/teachers')
        await page.waitForLoadState('networkidle')
        await page.waitForTimeout(300)
        await page.screenshot({
          path: path.join(screenshotsDir, `teachers-${theme}-${lang}.png`),
          fullPage: true,
        })
      }
    }
  })
})
