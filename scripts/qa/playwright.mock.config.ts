// QA-only wrapper: same as ui/playwright.config.ts but uses the sandbox's preinstalled Chromium
// (Playwright CDN is blocked in the sandbox -> the pinned browser build 1243 cannot be downloaded).
import base from '../../ui/playwright.config'
import { defineConfig } from '@playwright/test'
export default defineConfig({
  ...base,
  testDir: '../../ui/e2e',
  outputDir: '../../.tools/pw-out',
  projects: [{ name: 'chromium', use: { ...(base.projects?.[0]?.use ?? {}), launchOptions: { executablePath: process.env.QA_CHROMIUM } } }],
  webServer: { ...(base.webServer as object), cwd: '../../ui' } as never,
})
