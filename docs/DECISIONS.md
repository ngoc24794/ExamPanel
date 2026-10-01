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

## ADR-0008: Rollback Journal Mode for Single-File SQLite Portability
- **Status:** Accepted
- **Context:** Write-Ahead Logging (WAL) mode generates auxiliary `-wal` and `-shm` sidecar files in the data directory. When running on removable media (FAT32/exFAT USB drives) or copying the database across computers, stray locks or uncommitted sidecar files risk data divergence or file lock failures. ExamPanel is a single-user desktop application with low write contention.
- **Decision:** Use standard rollback journal mode (`PRAGMA journal_mode = DELETE;`) with foreign keys enforced (`PRAGMA foreign_keys = ON;`).
- **Consequences:** The entire SQLite database resides strictly in a single portable file (`data/exam-panel.db`). Safe to copy, move, back up, or execute directly from external storage drives without sidecar synchronization risks.

## ADR-0009: Embedded SQL Migrations Managed via PRAGMA user_version
- **Status:** Accepted
- **Context:** Database schema evolution requires repeatable, ordered execution without external CLI dependencies or heavy third-party ORM migration crates.
- **Decision:** Embed incremental SQL migration scripts into the binary using `include_str!` under `crates/storage/migrations/`. Track the applied schema version using SQLite's built-in `PRAGMA user_version`. Apply pending scripts sequentially inside explicit transactions during database initialization.
- **Consequences:** Zero external crate overhead, guaranteed forward progression, idempotent execution across application starts, and compile-time inclusion of all migration scripts.

## ADR-0010: Per-School-Year Teacher Grade Qualifications (`teacher_grades`)
- **Status:** Accepted
- **Context:** In educational institutions, teaching assignments shift across academic years. A teacher instructing grade 10 this year may be assigned to grades 11 and 12 next year. Modeling grades taught as a static attribute on the `teachers` entity would overwrite historical qualification records and prevent historical plan replay.
- **Decision:** Model grade assignments via a separate ternary associative entity `teacher_grades (teacher_id, school_year_id, grade_id)` with a `copy_teacher_grades(from_year, to_year)` routine for convenient rollover.
- **Consequences:** Full historical fidelity for prior school years and schedules, flexible yearly roster management, and clean isolation between teacher profile records and annual assignments.

## ADR-0011: Migration Immutability Starting at v0.1.0
- **Status:** Accepted
- **Context:** Early during Phase 1–2 development prior to the first release (v0.1.0), baseline schema migrations could be directly refined. Once a version is officially released, mutating past migrations causes divergence and failure when updating existing client databases.
- **Decision:** Migrations become immutable starting with the first release (v0.1.0); afterwards every schema change must be a new numbered migration file. Additionally, migration DDL removes all `IF NOT EXISTS` clauses because SQLite's `user_version` is the single source of truth; silent no-ops hide migration mistakes and masking schema drift.
- **Consequences:** Deterministic schema evolution, transparent failure on migration errors, and clean forward-only evolution post-v0.1.0.

## ADR-0012: Availability-Scaled Workload Quota Formulation (H7)
- **Status:** Accepted
- **Context:** In real academic environments, teachers have varying availability across exam periods (e.g. maternal/medical leaves, administrative exemptions) and differing grade qualifications. Assigning uniform annual quotas based solely on static load weight leads to infeasible lower bounds when a teacher is unavailable for several exam terms.
- **Decision:** Scale each teacher's effective weight $w'_t$ by their actual exam availability: $w'_t = \text{load\_weight}_t \times \frac{\text{availability}_t}{\text{number of exams}}$. Quota bounds $[\text{lo}_t, \text{hi}_t]$ with tolerance $k$ (default 1) are derived from $q_t = D \times \frac{w'_t}{\sum w'}$, and $\text{hi}_t$ is clamped to the maximum achievable assignments ($\le \text{availability}_t$ under H4).
- **Consequences:** Mathematical feasibility is preserved even under heavy leave schedules, while avoiding unfair assignment overloads.

## ADR-0013: In-House Max-Flow and Backtracking Solver Without External Crate Coupling
- **Status:** Accepted
- **Context:** Feasibility verification requires exact bipartite maximum flow for per-exam panel capacity under H4. Hard-constraint generation requires deterministic constructive search. Relying on external mathematical programming or SAT/SMT solvers (e.g., OR-Tools, Z3) introduces heavyweight C/C++ FFI dependencies, cross-compilation friction, and dynamic linking failure modes on portable media.
- **Decision:** Implement a pure Rust, allocation-conscious Dinic/Edmonds-Karp max-flow algorithm and an MRV-directed randomized backtracking solver directly in `crates/core`. Seeded with PRNG for 100% reproducible results.
- **Consequences:** Zero external solver crate dependencies, instantaneous compile times, zero unsafe code, complete portability, and predictable microsecond-level execution.


