# ExamPanel

> Cross-platform, portable desktop application that automatically assigns teachers to write and review exam papers across a full school year. Built with Rust, Tauri 2, React, TypeScript, and SQLite.

---

## Overview & Architecture

ExamPanel solves the high school exam panel assignment problem (Phân công ra đề và phản biện đề kiểm tra) by combining a mathematical constraint solver with an intuitive, bilingual (Vietnamese/English) desktop interface.

### Architectural Invariants
- **`crates/core`:** Pure mathematical domain core, constraint checkers (H1–H7 hard constraints, S1–S7 soft objectives), simulated annealing solver, and scoring algorithms. Zero external I/O or Tauri dependencies.
- **`crates/storage`:** Embedded SQLite (`rusqlite`) storage engine, incremental schema migrations (`crates/storage/migrations/`), automated backup rotations, and portable path resolution.
- **`crates/service`:** Application business services, Excel template importing and report generation (`rust_xlsxwriter`, `calamine`), session management, and trial sandbox isolation.
- **`src-tauri`:** Tauri 2 desktop shell with single-instance enforcement, window-state persistence, native logging (`tauri-plugin-log`), and scoped security permissions.
- **`ui`:** React 18, TypeScript, Tailwind CSS, shadcn/ui, TanStack Query, and 100% i18n (`vi` default, `en` secondary).

```
ExamPanel/
├── crates/
│   ├── core/                 # Pure domain + solver (no Tauri/SQLite)
│   ├── storage/              # SQLite connection, migrations, backups
│   └── service/              # Business services, Excel import/export
├── src-tauri/                # Desktop shell wrapper (Tauri 2)
├── ui/                       # React + TypeScript + Vite + Tailwind UI
├── docs/                     # Specifications, architecture, user guides
│   ├── SPEC.md               # Full business specification
│   ├── ARCHITECTURE.md       # Architecture diagram and layers
│   ├── DECISIONS.md          # Architectural Decision Records (ADRs)
│   ├── user-guide/           # Bilingual user guides (vi, en) and PDF
│   └── release/              # Release procedures and manual checklist
├── scripts/                  # Version checks, packaging, and CI helpers
└── .github/workflows/        # CI/CD and release automation
```

---

## Getting Started

### Prerequisites
- **Node.js** `v20+` (tested on `v24.18.0`)
- **pnpm** `v9+` (tested on `v11.22.0`)
- **Rust** `1.80+` (tested on `1.98.0 / 1.99.0`)
- **Microsoft Edge WebView2** (preinstalled on modern Windows 10/11)

### Installation
```bash
# Clone the repository
git clone <repo-url>
cd ExamPanel

# Install dependencies
pnpm install
pnpm -C ui install
```

---

## Development & Verification Commands

```bash
# Start frontend dev server in browser mode (http://127.0.0.1:5173)
pnpm dev-ui

# Start Tauri desktop app in development mode
pnpm tauri dev

# Run full test suite (Rust unit/integration tests + UI Vitest tests)
pnpm test

# Run code linters (Cargo clippy + ESLint)
pnpm lint

# Check code formatting (rustfmt + prettier)
pnpm fmt

# Run complete CI verification pipeline
pnpm check-all

# Run full Playwright End-to-End test suite
pnpm -C ui test:e2e
```

---

## Release Packaging

To build the release packages on Windows:

```bash
# 1. Verify version consistency across Cargo.toml, tauri.conf.json, and package.json
node scripts/check-version.mjs

# 2. Build release binary and NSIS installer
pnpm tauri build

# 3. Create Windows Portable ZIP
# Generates target/ExamPanel-0.1.0-windows-x64-portable.zip
# Containing ExamPanel.exe, ExamPanel.portable marker, DOC-TOI.txt, and user guide.
```

---

## Documentation

- **Vietnamese User Guide:** [docs/user-guide/vi/01-bat-dau-nhanh.md](docs/user-guide/vi/01-bat-dau-nhanh.md)
- **English User Guide:** [docs/user-guide/en/README.md](docs/user-guide/en/README.md)
- **Release Manual Checklist:** [docs/release/v0.1.0-checklist.md](docs/release/v0.1.0-checklist.md)
- **Architecture & ADRs:** [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md), [docs/DECISIONS.md](docs/DECISIONS.md)
