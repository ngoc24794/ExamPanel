# ExamPanel Technical Stack & Environment Reference

## 1. Verified Tool & Dependency Versions

### 1.1 Backend & Desktop Shell (Rust / Tauri 2)
| Component | Actual Version | Notes |
|-----------|----------------|-------|
| **Rust Toolchain** | `rustc 1.99.0` / `cargo 1.98.0+` | `stable-x86_64-pc-windows-gnu` / `stable` |
| **C Compiler / Linker** | GCC `16.2.0` | Portable MinGW-w64 toolchain |
| **Tauri Core** | `tauri 2.12.1` | Thin desktop wrapper shell |
| **Tauri Build** | `tauri-build 2.7.1` | Build script generator |
| **Tauri API Client** | `@tauri-apps/api 2.12.0` | Frontend IPC client |
| **SQLite Engine** | `rusqlite 0.32.1` | `features = ["bundled"]` |
| **Serialization** | `serde 1.0.229`, `serde_json 1.0.151` | Workspace shared |
| **Error Handling** | `thiserror 2.0.21` | Strongly typed errors |

### 1.2 Frontend (React / TypeScript / Vite)
| Component | Actual Version | Notes |
|-----------|----------------|-------|
| **Node.js** | `v24.18.0` | Active runtime |
| **Package Manager** | `pnpm 11.22.0` | Fast, disk-efficient package manager |
| **UI Framework** | `react 18.3.1`, `react-dom 18.3.1` | Dual web and desktop rendering |
| **Language** | `typescript 5.9.3` | Strict type checking enabled |
| **Build Tool / Bundler**| `vite 7.3.6` | Lightning-fast HMR and ESM bundling |
| **Styling** | `tailwindcss 3.4.19`, `postcss 8.5.28` | CSS variables + dark mode |
| **Component Primitives**| `shadcn/ui` (CVA `0.7.1`, `tailwind-merge 3.7.0`) | Accessible UI components |
| **Iconography** | `lucide-react 0.475.0` | Tree-shakeable icons |
| **i18n** | `i18next 24.2.3`, `react-i18next 15.7.4` | Default Vietnamese (`vi`), fallback English (`en`) |
| **Testing** | `vitest 3.2.7`, `@testing-library/react 16.3.3` | In-memory JSDOM test runner |
| **Linting & Formatting**| `eslint 9.39.5`, `prettier 3.9.9` | Flat config ESLint + strict formatting |

---

## 2. Common Developer Commands

All convenience scripts can be run from the repository root:

| Command | Action |
|---------|--------|
| `pnpm dev-ui` | Launches the React frontend in browser development mode on `http://127.0.0.1:5173` |
| `pnpm test` | Runs both Rust tests (`cargo test`) and UI tests (`pnpm -C ui test`) |
| `pnpm test:rust` | Runs Rust unit tests across default workspace members (`crates/core`, `crates/storage`) |
| `pnpm test:ui` | Runs Vitest frontend test suite |
| `pnpm lint` | Runs `cargo clippy -- -D warnings` and `eslint` on UI |
| `pnpm fmt` | Checks Rust (`cargo fmt -- --check`) and UI (`prettier --check`) formatting |
| `pnpm fmt:fix` | Automatically formats Rust and UI code |
| `pnpm check-all` | Full CI verification: fmt check, clippy, cargo test, ui lint, typecheck, ui test, ui build |

Direct Cargo commands:
```bash
# Test core domain and storage crates
cargo test

# Check clippy warnings
cargo clippy -- -D warnings

# Check formatting
cargo fmt --check

# Check Tauri crate
cargo check -p exam-panel-app
```

Direct UI commands:
```bash
cd ui
pnpm dev         # Dev server
pnpm build       # Typecheck + Vite production bundle
pnpm test        # Vitest
pnpm lint        # ESLint
pnpm typecheck   # tsc --noEmit
pnpm format      # Prettier format
```

---

## 3. System Packages & Build Environment

### 3.1 Headless Linux / CI Requirements (Ubuntu / Debian)
When building the full Tauri application on Ubuntu (e.g. in CI or on headless production servers), the following system packages must be present:
```bash
sudo apt-get install -y \
  build-essential \
  curl \
  wget \
  file \
  libssl-dev \
  libgtk-3-dev \
  libwebkit2gtk-4.1-dev \
  libsoup-3.0-dev \
  libjavascriptcoregtk-4.1-dev \
  libayatana-appindicator3-dev \
  librsvg2-dev
```
*Note:* The Cargo workspace isolates `src-tauri` from `default-members`. Therefore, running backend tests and core solver development in headless environments requires **none** of the GUI packages above.

### 3.2 Windows Development Environment
- On Windows systems without Visual Studio C++ Build Tools (`link.exe`), the project uses `stable-x86_64-pc-windows-gnu` paired with a portable MinGW-w64 toolchain (`gcc 16.2.0`).
- Local config is preserved in `.cargo/config.toml`.
