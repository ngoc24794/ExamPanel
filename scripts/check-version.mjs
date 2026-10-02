import fs from 'node:fs'
import path from 'node:path'
import { fileURLToPath } from 'node:url'

const __filename = fileURLToPath(import.meta.url)
const __dirname = path.dirname(__filename)
const rootDir = path.resolve(__dirname, '..')

// 1. Root Cargo.toml
const cargoTomlPath = path.join(rootDir, 'Cargo.toml')
const cargoContent = fs.readFileSync(cargoTomlPath, 'utf8')
const cargoMatch = cargoContent.match(/\[workspace\.package\][\s\S]*?version\s*=\s*"([^"]+)"/)
if (!cargoMatch) {
  console.error('Error: Could not extract [workspace.package] version from Cargo.toml')
  process.exit(1)
}
const cargoVersion = cargoMatch[1]

// 2. tauri.conf.json
const tauriConfPath = path.join(rootDir, 'src-tauri', 'tauri.conf.json')
const tauriConf = JSON.parse(fs.readFileSync(tauriConfPath, 'utf8'))
const tauriVersion = tauriConf.version

// 3. ui/package.json
const uiPkgPath = path.join(rootDir, 'ui', 'package.json')
const uiPkg = JSON.parse(fs.readFileSync(uiPkgPath, 'utf8'))
const uiVersion = uiPkg.version

// 4. root package.json
const rootPkgPath = path.join(rootDir, 'package.json')
const rootPkg = JSON.parse(fs.readFileSync(rootPkgPath, 'utf8'))
const rootPkgVersion = rootPkg.version

console.log(`Checking version consistency (Source of truth: Cargo.toml = ${cargoVersion}):`)
console.log(`  - Cargo.toml:             ${cargoVersion}`)
console.log(`  - src-tauri/tauri.conf:   ${tauriVersion}`)
console.log(`  - ui/package.json:        ${uiVersion}`)
console.log(`  - package.json:           ${rootPkgVersion}`)

let mismatch = false

if (tauriVersion !== cargoVersion) {
  console.error(`Mismatch: src-tauri/tauri.conf.json version (${tauriVersion}) != Cargo.toml version (${cargoVersion})`)
  mismatch = true
}

if (uiVersion !== cargoVersion) {
  console.error(`Mismatch: ui/package.json version (${uiVersion}) != Cargo.toml version (${cargoVersion})`)
  mismatch = true
}

if (rootPkgVersion !== cargoVersion) {
  console.error(`Mismatch: package.json version (${rootPkgVersion}) != Cargo.toml version (${cargoVersion})`)
  mismatch = true
}

if (mismatch) {
  console.error('FAILED: Version inconsistency detected.')
  process.exit(1)
}

console.log(`\nVersion consistency verified: ${cargoVersion} matches across all configuration files.`)
process.exit(0)
