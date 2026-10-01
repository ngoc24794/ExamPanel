# ExamPanel Architecture Decision Records (ADR)

## ADR-0001: Workspace Isolation of Tauri Shell
- **Status:** Accepted
- **Context:** Developers and continuous integration workflows frequently run in headless server environments without display servers or GTK/WebKit system libraries.
- **Decision:** Exclude `src-tauri` from Cargo workspace `default-members`. The default workspace members include only `crates/core` and `crates/storage`.
- **Consequences:** Backend developers and CI runners can run `cargo test`, `cargo clippy`, and `cargo fmt` without installing native desktop GUI libraries.

## ADR-0002: Embedded SQLite Engine via rusqlite `bundled`
- **Status:** Accepted
- **Context:** ExamPanel is designed as a portable application that can be run directly from portable USB drives or without external database server installation.
- **Decision:** Use `rusqlite` with the `bundled` feature flag to compile SQLite directly into the binary.
- **Consequences:** Ensures zero dependency on external SQLite dynamic libraries on host machines, providing consistent behavior across Windows, macOS, and Linux.

## ADR-0003: Dual-Mode API Interface (`ExamPanelApi`)
- **Status:** Accepted
- **Context:** Web frontend development is fastest inside standard browsers (`pnpm dev` with Vite HMR), but production execution occurs within Tauri's Webview IPC.
- **Decision:** Define a strongly typed TypeScript interface `ExamPanelApi` and provide two implementations: `tauri.ts` (using `@tauri-apps/api/core`) and `mock.ts` (browser fallback). Automatically detect the runtime environment (`__TAURI_INTERNALS__`).
- **Consequences:** Allows standard browser-based frontend development and automated Vitest testing without needing a running Tauri desktop window.

## ADR-0004: Internationalization (i18n) Strategy
- **Status:** Accepted
- **Context:** The application serves Vietnamese educational institutions while needing international language capability.
- **Decision:** Implement `i18next` + `react-i18next` with Vietnamese (`vi`) as default and English (`en`) as secondary. Enforce hierarchical keys (`nav.*`, `common.*`, etc.) and prohibit hard-coded strings in components.
- **Consequences:** Easy localization, complete parity between languages, and maintainable string dictionaries.

## ADR-0005: CSS Variable-Driven Theme Engine
- **Status:** Accepted
- **Context:** Desktop applications require dark and light themes, including automatic adaptation to the OS system theme.
- **Decision:** Use Tailwind CSS with CSS variable tokens (`hsl(var(--...))`) controlled by a `.dark` class on the root element. Support `light`, `dark`, and `system` modes with persistence routed through the API layer.
- **Consequences:** Instant theme switching without page reload, smooth transitions, and adherence to system color preferences.

## ADR-0006: Portable Data Directory Resolution
- **Status:** Accepted
- **Context:** Portable installations must store data alongside the executable, while standard OS installations must write to appropriate user directories without permission failures.
- **Decision:** Probe writability of `<exe_dir>/data`. If writable, use `<exe_dir>/data/exam-panel.db`. Otherwise, fall back to `%APPDATA%/ExamPanel/data` (Windows) or `~/.local/share/ExamPanel/data` (Linux/macOS).
- **Consequences:** Seamless portable operation on flash drives while remaining fully compatible with protected installation folders like `C:\Program Files`.

## ADR-0007: Official Windows Release Uses MSVC Target; GNU Target as Optional Local-Dev Fallback
- **Status:** Accepted
- **Context:** Windows builds under Rust can target either MSVC (`x86_64-pc-windows-msvc`) or GNU (`x86_64-pc-windows-gnu`). The official Tauri Windows runtime, WebView2 bindings, and CI pipelines rely on Microsoft C++ runtime and SDK standards. However, local developer workstations may lack full Visual Studio C++ build installations.
- **Decision:** The official Windows release build and CI pipelines target `x86_64-pc-windows-msvc`. The GNU toolchain (`x86_64-pc-windows-gnu`) is supported solely as an optional local developer fallback. Repository configuration files (`rust-toolchain.toml`, `.cargo/config.toml`) must remain host-neutral. Any developer-specific toolchain overrides or MinGW linker paths must reside in user-level configuration (`~/.cargo/config.toml`), never in the git repository.
- **Consequences:** Eliminates hard-coded machine paths in the repository, guarantees clean headless CI builds on Ubuntu/Windows runners, and preserves developer flexibility.

