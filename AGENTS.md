# Agent Guidelines for ExamPanel

This document defines obligatory conventions and working agreements for AI agents and human contributors working on ExamPanel phases 2 through 10.

---

## 1. Required Preparation
- **Read Documentation First:** Before designing or modifying code, thoroughly review [docs/SPEC.md](docs/SPEC.md) and [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md).
- **Consult Decision Records:** Check [docs/DECISIONS.md](docs/DECISIONS.md) to understand historical architectural trade-offs.

---

## 2. Core & Backend Invariants
- **Zero Coupling for `crates/core`:** `crates/core` must **never** depend on `tauri`, `rusqlite`, or external I/O libraries. It is a pure mathematical and algorithmic domain core.
- **Unit Test Requirement:** Every new entity, constraint validator, and solver routine must have comprehensive unit tests in `crates/core`.
- **Workspace Separation:** Maintain `crates/core` and `crates/storage` in `default-members` of `Cargo.toml`. `src-tauri` must remain decoupled from `default-members`.
- **Database Migrations:** All schema changes in `crates/storage` must be managed via clean, incremental SQL migration files under `crates/storage/migrations/`.

---

## 3. Frontend & UI Invariants
- **100% Internationalization (i18n):**
  - Never hard-code user-facing strings in UI components.
  - Every string must use hierarchical keys (`t('domain.key')`).
  - Always update **both** `ui/src/i18n/locales/vi.json` (Vietnamese default) and `ui/src/i18n/locales/en.json` (English).
- **Theme Consistency:**
  - Never use hard-coded color hexes or raw tailwind color classes like `bg-blue-500` or `text-white` directly for structural elements.
  - Always use semantic CSS variable tokens: `bg-background`, `text-foreground`, `bg-card`, `text-muted-foreground`, `bg-primary`, `border-border`, etc.
- **API Decoupling:**
  - All communication between UI and backend must pass through the `ExamPanelApi` interface in `ui/src/lib/api/types.ts`.
  - When adding a new API method:
    1. Update the `ExamPanelApi` interface in `types.ts`.
    2. Implement it in `mock.ts` (for browser testing).
    3. Implement it in `tauri.ts` (with Tauri `invoke`).

---

## 4. Quality Assurance & Verification
Before marking any task as complete:
1. Run the unified verification command from the repo root:
   ```bash
   pnpm check-all
   ```
2. Verify that:
   - `cargo fmt --check` passes.
   - `cargo clippy -- -D warnings` passes with zero warnings.
   - `cargo test` passes.
   - `pnpm -C ui lint` passes with zero warnings.
   - `pnpm -C ui typecheck` passes with zero errors.
   - `pnpm -C ui test` passes.
   - `pnpm -C ui build` succeeds.
3. Update [docs/DECISIONS.md](docs/DECISIONS.md) if any architectural or technical decision was made during the phase.

---

## 5. Git Commit Protocol
- Use **Conventional Commits** syntax:
  - `feat: <description>` for new features
  - `fix: <description>` for bug fixes
  - `chore: <description>` for build, tooling, or infrastructure tasks
  - `docs: <description>` for documentation updates
  - `refactor: <description>` for non-functional code improvements
  - `test: <description>` for test additions
- Do **not** push to remote unless explicitly instructed by the user.
