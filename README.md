# ExamPanel

> Cross-platform, portable desktop application that automatically assigns teachers to write and review exam papers across a full school year.

---

## Features (Phase 1: Architecture & Foundation)
- **High-Performance Pure Core:** Rust workspace isolating domain logic, constraint checkers, and solvers from UI and database concerns.
- **Embedded Portable Database:** Bundled SQLite (`rusqlite`) with automatic portable path resolution (`./data/exam-panel.db` with OS app-data fallback).
- **Modern Responsive Frontend:** React 18, TypeScript, Vite, Tailwind CSS, and shadcn/ui.
- **Dual-Mode Execution:** Automatic environment detection running in standard browsers (`MockExamPanelApi`) or inside desktop shells (`TauriExamPanelApi`).
- **Complete Internationalization (i18n):** Default Vietnamese (`vi`) and English (`en`) with zero hard-coded UI strings.
- **Adaptive Theme System:** Light, dark, and system color scheme support with CSS variable tokens.

---

## Directory Structure
```
exam-panel/
├── AGENTS.md                 # Agent working agreements & invariants
├── README.md                 # Project overview and quickstart guide
├── Cargo.toml                # Rust workspace configuration
├── rust-toolchain.toml       # Rust toolchain pinning
├── package.json              # Root orchestration and convenience scripts
├── .editorconfig             # Editor whitespace and encoding standards
├── .gitignore                # Git exclusions (target, data, db, node_modules)
├── .github/
│   └── workflows/
│       └── ci.yml            # Automated CI workflows (Rust & UI)
├── docs/
│   ├── SPEC.md               # Full business specification (H1-H7, S1-S7)
│   ├── ARCHITECTURE.md       # Architecture diagram, data flow, layers
│   ├── TECH.md               # Tool versions & common commands
│   └── DECISIONS.md          # Architectural Decision Records (ADRs)
├── crates/
│   ├── core/                 # Pure domain + solver (no Tauri/SQLite)
│   └── storage/              # SQLite connection, paths, migrations
├── src-tauri/                # Desktop shell wrapper (Tauri 2)
└── ui/                       # React + TypeScript + Vite + Tailwind UI
```

---

## Getting Started

### Prerequisites
- **Node.js** `v20+` (tested on `v24.18.0`)
- **pnpm** `v9+` (tested on `v11.22.0`)
- **Rust** `1.80+` (tested on `1.98.0 / 1.99.0`)

### Quick Setup
```bash
# Clone the repository
git clone <repo-url>
cd exam-panel

# Install frontend dependencies
pnpm -C ui install
```

---

## Development Scripts

From the repository root, you can run:

```bash
# Start frontend dev server in browser mode (http://127.0.0.1:5173)
pnpm dev-ui

# Run full test suite (Rust unit tests + UI Vitest tests)
pnpm test

# Run code linters (Cargo clippy + ESLint)
pnpm lint

# Check code formatting (rustfmt + prettier)
pnpm fmt

# Automatically format code
pnpm fmt:fix

# Run complete CI verification pipeline
pnpm check-all
```

---

## Testing & Verification

### Backend Tests
```bash
cargo test
cargo clippy -- -D warnings
cargo fmt --check
```

### Frontend Tests
```bash
cd ui
pnpm test
pnpm lint
pnpm typecheck
pnpm build
```

---

## License
MIT OR Apache-2.0
