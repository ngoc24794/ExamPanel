import { chromium } from '@playwright/test'
import fs from 'node:fs'
import path from 'node:path'
import { fileURLToPath } from 'node:url'

const __filename = fileURLToPath(import.meta.url)
const __dirname = path.dirname(__filename)
const rootDir = path.resolve(__dirname, '..', '..')

const svgPath = path.join(rootDir, 'src-tauri', 'icons', 'icon.svg')
const pngPath = path.join(rootDir, 'src-tauri', 'icons', 'icon.png')
const svgContent = fs.readFileSync(svgPath, 'utf-8')

const browser = await chromium.launch()
const page = await browser.newPage({
  viewport: { width: 1024, height: 1024 },
  deviceScaleFactor: 1,
})

await page.setContent(`<!DOCTYPE html>
<html>
  <head>
    <style>
      * { margin: 0; padding: 0; box-sizing: border-box; }
      body { width: 1024px; height: 1024px; overflow: hidden; background: transparent; }
      svg { width: 1024px; height: 1024px; display: block; }
    </style>
  </head>
  <body>
    ${svgContent}
  </body>
</html>`)

await page.screenshot({
  path: pngPath,
  omitBackground: true,
})

await browser.close()
console.log(`Rendered 1024x1024 icon to ${pngPath}`)
