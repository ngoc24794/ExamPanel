import fs from 'node:fs'
import path from 'node:path'
import { execSync } from 'node:child_process'
import { fileURLToPath } from 'node:url'

const __filename = fileURLToPath(import.meta.url)
const __dirname = path.dirname(__filename)
const rootDir = path.resolve(__dirname, '..')

console.log('Generating third-party license notices...')

// 1. Rust packages via cargo metadata
const cargoMetaRaw = execSync('cargo metadata --format-version 1', {
  cwd: rootDir,
  encoding: 'utf-8',
  maxBuffer: 64 * 1024 * 1024,
})
const cargoMeta = JSON.parse(cargoMetaRaw)

const internalCrates = new Set([
  'exam-panel-core',
  'exam-panel-storage',
  'exam-panel-service',
  'exam-panel-app',
])

const rustPackages = new Map()
for (const pkg of cargoMeta.packages) {
  if (internalCrates.has(pkg.name)) continue
  rustPackages.set(pkg.name, {
    name: pkg.name,
    version: pkg.version,
    license: pkg.license || 'Unknown',
    repository: pkg.repository || null,
  })
}

// 2. Node.js packages via ui/package.json + pnpm
const pnpmRaw = execSync('pnpm -C ui ls --json --depth 0', {
  cwd: rootDir,
  encoding: 'utf-8',
  maxBuffer: 32 * 1024 * 1024,
})
const pnpmData = JSON.parse(pnpmRaw)
const jsPackages = new Map()

function inspectDep(depObj) {
  if (!depObj) return
  for (const [name, info] of Object.entries(depObj)) {
    let license = 'Unknown'
    let repo = null
    try {
      if (info.path && fs.existsSync(path.join(info.path, 'package.json'))) {
        const pkgJson = JSON.parse(fs.readFileSync(path.join(info.path, 'package.json'), 'utf-8'))
        license = pkgJson.license || (Array.isArray(pkgJson.licenses) ? pkgJson.licenses.map(l => l.type || l).join(', ') : 'Unknown')
        if (pkgJson.repository) {
          repo = typeof pkgJson.repository === 'string' ? pkgJson.repository : pkgJson.repository.url
        }
      }
    } catch {
      // ignore read error
    }
    jsPackages.set(name, {
      name,
      version: info.version,
      license: typeof license === 'string' ? license : JSON.stringify(license),
      repository: repo,
    })
  }
}

if (Array.isArray(pnpmData) && pnpmData[0]) {
  inspectDep(pnpmData[0].dependencies)
  inspectDep(pnpmData[0].devDependencies)
}

// 3. Count summaries
const licenseCounts = {}
for (const p of rustPackages.values()) {
  licenseCounts[p.license] = (licenseCounts[p.license] || 0) + 1
}
for (const p of jsPackages.values()) {
  licenseCounts[p.license] = (licenseCounts[p.license] || 0) + 1
}

// 4. Output reports
const reportDir = path.join(rootDir, 'docs', 'reports', 'phase-10')
fs.mkdirSync(reportDir, { recursive: true })

let summaryText = `EXAMPANEL THIRD-PARTY LICENSES SUMMARY
Generated on: ${new Date().toISOString()}

Total Rust dependencies: ${rustPackages.size}
Total JS dependencies:   ${jsPackages.size}

LICENSE DISTRIBUTION:
`
for (const [lic, count] of Object.entries(licenseCounts).sort((a, b) => b[1] - a[1])) {
  summaryText += `  - ${lic}: ${count}\n`
}

fs.writeFileSync(path.join(reportDir, 'licenses-summary.txt'), summaryText, 'utf-8')

// Markdown notice
let mdContent = `# Third-Party Software Notices & Licenses

ExamPanel incorporates components from third-party open-source software packages.
This document provides notice and license details for each third-party component.

## Summary

- **Total Rust packages:** ${rustPackages.size}
- **Total JavaScript packages:** ${jsPackages.size}

---

## Rust Dependencies

| Crate Name | Version | License | Repository |
| :--- | :--- | :--- | :--- |
`

const sortedRust = Array.from(rustPackages.values()).sort((a, b) => a.name.localeCompare(b.name))
for (const p of sortedRust) {
  mdContent += `| \`${p.name}\` | ${p.version} | ${p.license} | ${p.repository || 'N/A'} |\n`
}

mdContent += `\n---\n\n## JavaScript Dependencies\n\n| Package Name | Version | License | Repository |\n| :--- | :--- | :--- | :--- |\n`

const sortedJs = Array.from(jsPackages.values()).sort((a, b) => a.name.localeCompare(b.name))
for (const p of sortedJs) {
  mdContent += `| \`${p.name}\` | ${p.version} | ${p.license} | ${p.repository || 'N/A'} |\n`
}

fs.writeFileSync(path.join(rootDir, 'THIRD_PARTY_NOTICES.md'), mdContent, 'utf-8')
fs.writeFileSync(path.join(rootDir, 'THIRD_PARTY_NOTICES'), mdContent, 'utf-8')

console.log(`Generated THIRD_PARTY_NOTICES (${rustPackages.size} Rust crates, ${jsPackages.size} JS packages).`)
console.log(`Saved summary to ${path.join(reportDir, 'licenses-summary.txt')}.`)
