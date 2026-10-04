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

## ADR-0014: Redefinition of S6 Soft Constraint to Consecutive Setting Only
- **Status:** Accepted
- **Context:** The original soft rule S6 attempted to penalize any consecutive exam assignment (setting or reviewing). However, with 4 standard exams in an academic year and an average teacher workload quota of ~3.5 tasks under H4 (max 1 panel per exam), almost every teacher must participate in at least 3 out of 4 exams. Consequently, avoiding consecutive assignments across all roles is mathematically impossible for most of the faculty, leading to unavoidable baseline penalties.
- **Decision:** Restrict S6 to penalize consecutive assignments where a teacher serves as a SETTER in two adjacent exam terms (ordered by `sort_order`). Setting an exam paper involves authoring questions, marking schemes, and multi-round revisions—the heavy cognitive duty—whereas reviewing is a lighter oversight task.
- **Consequences:** Provides actionable optimization gradients without noise penalties from lighter reviewing assignments, pacing author fatigue realistically across academic semesters.

## ADR-0015: Addition of S8 Soft Rule for Workload Deviation Penalties
- **Status:** Accepted
- **Context:** Hard constraint H7 enforces teacher annual assignment counts within $[lo_t, hi_t]$ with integer tolerance $k$ (default 1). Within this tolerance window, the constructive solver treats any count in $[lo_t, hi_t]$ as equally valid. As a result, teachers with fractional target quotas $q_t = 3.47$ could be assigned 2 or 4 tasks without any preference pulling them toward 3 or their exact fair share.
- **Decision:** Introduce soft constraint S8 (`load_deviation`) with quadratic unit penalty $(count_t - q_t)^2$ and default weight 8.0. Maintain forward compatibility in storage by populating S8 defaults when loading problem snapshots and inserting S8 during school year creation.
- **Consequences:** Strongly biases local search optimization toward exact workload equity across all teachers, even when $k \ge 1$.

## ADR-0016: Deterministic Parallel Seeding for Multi-Run Local Search
- **Status:** Accepted
- **Context:** Local search (Simulated Annealing) explores stochastic neighborhoods. To deliver diverse, high-quality plans, multiple independent runs ($R = 8$) are executed in parallel across CPU cores. If thread scheduling or system entropy affects RNG seeds, repeated runs with the same user configuration would yield non-deterministic results, complicating testing, quality audits, and user reproducibility.
- **Decision:** Derive run seeds deterministically from a single `base_seed`: for run index $i \in [0, R-1]$, initialize PRNG with `base_seed.wrapping_add(i as u64 * 0x9E3779B97F4A7C15 + 1)`. Under iteration budgets, each SA trajectory executes identical moves regardless of thread allocation or execution order.
- **Consequences:** 100% reproducible optimization outputs, deterministic test suites, and embarrassingly parallel multi-core scaling via Rayon.

## ADR-0017: Greedy Max-Min Diversity Selection for Multi-Plan Output
- **Status:** Accepted
- **Context:** Presenting $R = 8$ solutions directly to human academic coordinators overwhelms decision-making. Presenting only the top-scoring plans often results in near-identical solutions that differ by only a single slot swap.
- **Decision:** Select up to $K = 3$ plans using greedy max-min diversity distance with a minimum diversity threshold $\tau = 0.20$ (at least 20% of panel assignment slots must differ). The global best-scoring plan is always selected as Rank 1. Successive plans are selected from the remaining candidate pool to maximize the minimum distance to already-selected plans, subject to $d(P, P_j) \ge \tau$. If fewer than $K$ plans satisfy the threshold, return only the qualifying plans.
- **Consequences:** Users receive genuinely distinct scheduling alternatives representing different operational trade-offs rather than cosmetic variations.

## ADR-0018: Redefinition of S2 Role Balance and Provable Lower Bounds
- **Status:** Accepted (Supersedes original S2 definition)
- **Context:** The original S2 definition penalized $|reviews_t - count_t / 3.0|$ continuously for $count_t \ge 2$. Because task counts and review assignments are discrete integers, teachers with $count_t = 4$ or $5$ were guaranteed an unavoidable penalty ($|1 - 4/3| = 0.333 \implies 1.0$ penalty units), distorting multi-objective trade-offs and artificially penalizing fair integer workload distributions. Furthermore, coordinators had no way to know whether a soft-constraint score could theoretically be improved.
- **Decision:**
  1. Redefine S2: For each teacher with $count_t \ge 2$, let $lo = \lfloor count_t / 3 \rfloor$ and $hi = \lceil count_t / 3 \rceil$. S2 units equal the distance from $reviews_t$ to the integer interval $[lo, hi]$ (0 when inside).
  2. Implement provable lower bounds in `crates/core/src/score/bounds.rs`:
     - S6: Pigeonhole on setter slots with $E$ exams: a teacher can set at most $\lceil E/2 \rceil$ times without adjacency; adjacent incident cost $\max(0, 2 s_t - E - 1)$; greedy allocation over teacher setter capacities yields the provable lower bound (e.g., 2.0 on 11-teacher, 4-exam benchmark).
     - S8: Global discrete minimum of $\sum_t (c_t - q_t)^2$ subject to $\sum c_t = D$ and $lo_t \le c_t \le hi_t$ via greedy marginal-cost allocation (2.60 on demo benchmark).
     - S1: Reviewer capacity pigeonhole bounds.
- **Consequences:** Eliminates artificial fractional role imbalance penalties, allows fair target allocation (e.g. Vũ Hải Hà receiving 1 task on demo seed and S8 reaching its lower bound 2.60), and empowers the UI to display "cannot be improved further" indicators.

## ADR-0019: Dedicated Headless Application Service Layer (crates/service)
- **Status:** Accepted
- **Context:** Application workflows (CRUD orchestration, feasibility verification, optimization execution, plan persistence) require coordinating between domain algorithms in `crates/core` and database access in `crates/storage`. Embedding this logic directly inside Tauri command handlers ties application business logic to Tauri's IPC framework, preventing headless integration testing and complicating testability.
- **Decision:** Create `crates/service` in workspace `default-members`. `AppService` encapsulates all application use-cases behind an API returning serializable DTOs and standardized `AppError`. `src-tauri` becomes a thin forwarding wrapper.
- **Consequences:** 100% headless integration testing on temp-file SQLite databases with zero Tauri or GUI dependencies; fast CI feedback loop and strong architectural decoupling.

## ADR-0020: Channel-Based Streaming IPC for Optimization Progress
- **Status:** Accepted
- **Context:** Simulated annealing local search runs across multiple iterations and parallel runs. The frontend needs throttled real-time progress events (`iteration`, `best_score`, `current_score`, `elapsed_ms`) to update progress bars and metrics. Using Tauri's global event system (`app.emit`) broadcasts events across the entire window system, coupling command invocations to global listeners and risking event collisions between concurrent sessions or tests.
- **Decision:** Utilize Tauri 2's `tauri::ipc::Channel<Progress>` parameter directly in the `start_optimize` command. Progress events are sent directly to the caller's channel closure.
- **Consequences:** Clean request-scoped streaming, zero global event pollution, and natural cancellation alignment.

## ADR-0021: Non-Blocking Snapshot Locking Model for Solver Concurrency
- **Status:** Accepted
- **Context:** `rusqlite::Connection` is not `Sync`, requiring a `Mutex<Store>` in multi-threaded application contexts. Holding the mutex lock while running multi-second optimization runs would block all concurrent read requests (e.g., browsing teacher lists, fetching app info, checking status) across the entire application.
- **Decision:** Strict snapshot-and-release pattern: Acquire `Store` lock, load immutable `Problem` snapshot, immediately drop lock, execute CPU-bound optimization asynchronously via Rayon/blocking worker, and re-acquire lock only to persist generated plans transactionally. Guard against multiple concurrent solver executions using a dedicated `Mutex<Option<Arc<AtomicBool>>>`.
- **Consequences:** Zero GUI freezing or query stalls during heavy optimization runs. Immediate rejection (`optimize_busy`) of concurrent optimization requests.

## ADR-0022: Database Migration 0002 for Extended Plan Persistence
- **Status:** Accepted
- **Context:** Multi-plan optimization outputs $K = 3$ ranked plans along with soft-constraint score breakdowns, provable lower bounds, per-teacher load statistics, and audit parameters. Migration 0001 only stored a single plan table with minimal metadata.
- **Decision:** Introduce migration file `crates/storage/migrations/0002_plans_extension.sql` adding `rank`, `score_report_json`, `run_params_json`, and `source` (`CHECK (source IN ('optimizer', 'manual', 'duplicate'))`). Write migration data preservation tests asserting existing records are upgraded without loss.
- **Consequences:** Complete transactional persistence of all candidate plans, full historical audit trail, and zero data loss on database upgrades.

## ADR-0023: TypeScript Interface Generation via ts-rs and Automated Freshness Testing
- **Status:** Accepted (Updated with MSVC toolchain restoration)
- **Context:** Ensuring type safety between Rust DTOs and the React frontend requires synchronized TypeScript interfaces. Under the previous temporary GNU toolchain, `ts-rs` pulled in platform terminal crates requiring SDK headers that caused linking issues. With the standard MSVC Desktop C++ toolchain active, `ts-rs` compiles natively without workarounds.
- **Decision:** Use `ts-rs` (v12 with `serde-compat`) in `crates/service` under the `dev-tools` feature to generate TypeScript definitions directly from Rust types. Retain the automated freshness test `crates/service/tests/generate_types.rs` which verifies that the committed `ui/src/lib/api/generated/types.ts` is in exact sync with Rust type declarations. CI and `pnpm check-all` will fail if Rust DTOs change without regenerating the TypeScript declarations.
- **Consequences:** End-to-end compile-time type safety across the IPC boundary, automated synchronization, and zero manual transcription errors.

## ADR-0024: UI Foundation Architecture, State Management, and Color Tokens
- **Status:** Accepted
- **Context:** Phase 6 establishes the interactive UI foundation for Campuses and Teachers screens. A robust client-side architecture is required to manage server synchronization, local form state, hash-based desktop routing, optimistic mutations with rollback, and accessible theming.
- **Decision:**
  1. **Server State:** TanStack Query (`@tanstack/react-query` v5) for asynchronous query caching, deduplication, mutation invalidation, and optimistic cache updates.
  2. **Routing:** `react-router-dom` v7 configured with hash routing (`HashRouter` / `createHashRouter`) to ensure full compatibility with Tauri's custom asset protocol (`tauri://localhost` or `http://tauri.localhost`).
  3. **Forms & Validation:** `react-hook-form` paired with `@hookform/resolvers` and `zod` for type-safe schema validation, binding validation errors to localized i18n messages.
  4. **Notifications:** `sonner` for accessible toast notifications mapped to `errors.<code>` localization keys.
  5. **Campus Color Tokens:** Fixed palette of semantic color tokens (`blue`, `emerald`, `amber`, `purple`, `rose`, `indigo`, `teal`, `orange`) with light and dark mode CSS variables guaranteeing WCAG AA text contrast ratio $\ge 4.5:1$. Database and DTOs persist token keys rather than raw hex values.
- **Consequences:** Responsive UI with immediate feedback, zero routing breaks on desktop reloads, full i18n compliance, and WCAG AA contrast accessibility.

## ADR-0025: Aggregated Diagnostics for Degenerate Feasibility Configurations
- **Status:** Accepted
- **Context:** When initializing an empty database or deleting campuses/teachers, checking feasibility previously triggered a cascading flood of 60+ individual panel errors (e.g., F1 panel missing teachers, F2 missing campuses across all exams and grades). This created confusing error sheets and masked root-cause issues.
- **Decision:** Check for degenerate baseline conditions before running panel-level and flow checks. If any of: `no_campuses`, `no_active_teachers`, `single_campus`, `no_grades`, or `no_exams` are met, emit an aggregated structural diagnostic with category `Data` and suppress downstream panel cascading diagnostics.
- **Consequences:** Clean diagnostic reporting for empty or freshly initialized databases (reducing 65 errors to 2), with clear actionable advice for the user.

## ADR-0026: Canonical Rule Presets Defined in crates/core Domain Layer
- **Status:** Accepted
- **Context:** The application provides preset rule weights: Balanced (default), Workload Fairness (amplified S8 and S1), and Team Diversity (amplified S4, S5, S3). Defining these presets in the frontend creates duplicate logic and risks inconsistency with solver benchmarking.
- **Decision:** Define `RulePreset` enum and weights canonically in `crates/core::domain::entities::RulePreset`. Expose preset values via `AppService::get_rule_presets` to the frontend and mock APIs.
- **Consequences:** Single source of truth across backend solver algorithms, headless tests, and client user interfaces.

## ADR-0027: Stateless Quota Preview Service Operation
- **Status:** Accepted
- **Context:** Users tuning hard rule H7 tolerance and load parameters in the Rules & Quotas screen need real-time visualization of per-teacher quotas ($q_t$ and $[lo_t, hi_t]$) before committing changes to the database.
- **Decision:** Implement `AppService::preview_quotas(PreviewQuotasInput)` which accepts unsaved tolerance and teacher weight overrides, loads teacher exam availability in-memory, computes mathematical quota distributions, and returns `Vec<QuotaPreviewItem>` without persisting any state or requiring transaction rollbacks.
- **Consequences:** Instant interactive UI feedback with zero database write side-effects.

## ADR-0028: Semester Grouping by Exam Sequence Split
- **Status:** Accepted
- **Context:** Academic years often divide into Semester 1 (Midterm 1 + Final 1) and Semester 2 (Midterm 2 + Final 2). The Unavailability grid requires bulk actions ("Vắng cả học kỳ 1", "Vắng cả học kỳ 2") for teacher leaves.
- **Decision:** Group exams by index: first half ($\lfloor N / 2 \rfloor$ exams) belongs to Semester 1, remaining exams belong to Semester 2. Document this convention in SPEC section 2.5 and expose bulk toggle actions accordingly.
- **Consequences:** Clean, deterministic semester mapping that works for standard 4-exam configurations (GK1, CK1 | GK2, CK2) as well as arbitrary user-defined exam sequences.

## ADR-0029: Canonical Problem Hashing for Plan Staleness Detection
- **Status:** Accepted
- **Context:** An optimization plan's validity and score are computed against a specific snapshot of problem inputs (teachers, grades, campuses, exams, unavailabilities, locks, and rule configurations). If a user modifies master data (e.g., adds an unavailability, changes a campus, or adjusts rule weights) after a plan is generated, the plan may become stale or violate constraints.
- **Decision:** Define a deterministic, order-independent canonical hash (`Problem::canonical_hash`) in `crates/core`. All collections (campuses, grades, teachers with their taught grades, exams, unavailabilities, locks, and rules) are sorted by canonical identifier order before feeding into SHA-256 with structured domain separators. Store `plans.problem_hash` (Migration `0003_plans_problem_hash.sql`) upon plan generation. When evaluating `plan_status(plan_id)`, compare the current canonical hash against the stored hash and re-evaluate hard violations and score against the live database state.
- **Consequences:** Deterministic staleness tracking immune to JSON key ordering or database retrieval sequence; clear UI warnings and guardrails against marking stale plans as final.

## ADR-0030: Fast Incremental State Evaluation for Real-Time Manual Plan Editing
- **Status:** Accepted
- **Context:** Interactive candidate selection and drag-drop swapping require computing delta scores ($\Delta\text{Score}$) and hard constraint violations across all eligible candidates in milliseconds to avoid UI lag.
- **Decision:** Implement `evaluate_candidates` and `evaluate_swap` using domain incremental scoring structures (`crates/core::optimize::eval_edit`). For each candidate or swap, only affected rules and panels are evaluated rather than recalculating the entire schedule from scratch. Automated property tests verify that the incremental delta matches a full `evaluate()` call to machine precision across random problem configurations.
- **Consequences:** Sub-millisecond candidate and swap evaluation for responsive keyboard navigation and drag-drop previews.

## ADR-0031: Re-Optimization Around Kept Slots via Ephemeral In-Memory Pin Locks
- **Status:** Accepted
- **Context:** When users find desirable panels in a generated plan but wish to improve other panels, they need the ability to "keep" specific slots while allowing the simulated annealing optimizer to re-optimize remaining assignments.
- **Decision:** Implement `reoptimize_from(plan_id, keep: Vec<SlotRef>, request)`:
  1. Validate kept slots against existing assignments in the plan.
  2. Synthesize ephemeral `LockKind::Pin` constraints for each kept slot in-memory without persisting them into the user's permanent database locks table.
  3. Initialize the solver's initial solution directly from the kept plan assignments.
  4. Run simulated annealing subject to the combined locks.
  5. Return new ranked draft plans via standard optimization outcomes.
- **Consequences:** Clean separation between temporary optimization pinning and permanent master data locks.

## ADR-0032: Zero-Dependency DOM and CSS Visualization for Statistics Page
- **Status:** Accepted
- **Context:** The Statistics page visualizes teacher workload distribution, deviation from quota, and setter vs. reviewer splits. Adding heavy charting libraries (e.g., Chart.js, Recharts, ECharts) adds 150–500 KB to the webview bundle and complicates automated headless testing and print styling.
- **Decision:** Build all statistical charts using semantic HTML tables, flexbox progress indicators, and CSS design tokens (bg-primary, bg-muted, etc.) styled with Tailwind CSS.
- **Consequences:** Zero external chart dependencies, instantaneous rendering, full i18n support, complete headless Playwright testability, and seamless print stylesheets.

## ADR-0033: Dual-Hash Staleness Tracking (Data Hash vs Rules Hash)
- **Status:** Accepted
- **Context:** In Phase 8, any modification to problem inputs triggered a single problem_hash mismatch, flagging the plan as stale and blocking mark_final. However, modifying soft constraint weights only affects the objective function score without altering feasibility or master data validity.
- **Decision:** Split the problem hash into two independent SHA-256 hashes:
  - data_hash: covers structural entities (campuses, grades, teachers + per-year grades, exams, unavailabilities, locks, and hard rule settings H4 and H7 tolerance).
  - rules_hash: covers soft constraint enabled flags and penalty weights (S1–S8).
  plan_status returns { data_changed, rules_changed, hard_violations_now, score_now }. Only data_changed = true blocks mark_final. When only rules_changed = true, the UI displays an informational notice and permits finalization. Saving an edited plan copy recomputes and synchronizes both hashes with live data.
- **Consequences:** Administrators can adjust evaluation criteria and tune soft weights without invalidating existing feasible plans, while preventing obsolete plans from being finalized against altered master rosters.

## ADR-0034: macOS Portable Packaging and Bundle-Aware Marker Resolution
- **Status:** Accepted
- **Context:** On macOS, application executables are encapsulated within `.app` application bundles (`<Bundle>.app/Contents/MacOS/<Executable>`). When distributing a portable version on macOS (e.g., on a flash drive or standalone directory), the portable marker `ExamPanel.portable` is placed alongside `ExamPanel.app` in the distribution root, or inside `Contents/MacOS`.
- **Decision:** Enhance `crates/storage/src/paths.rs` with `inspect_data_location_for_exe` to recognize macOS `.app` bundle hierarchies: if the executable path resides within an `.app` directory and `ExamPanel.portable` is present in the parent directory containing the `.app` bundle, the portable data path resolves to `./data` beside `ExamPanel.app`. Support read-only fallback (`PortableReadOnly`) if running on read-only mounted disk images. Add platform-specific `src-tauri/tauri.macos.conf.json` for `app` and `dmg` targets, automated packaging scripts (`scripts/build-portable-macos.sh`, `scripts/build-portable-macos.mjs`), and CI release pipeline steps.
- **Consequences:** Consistent, zero-configuration portable execution across macOS and Windows, honoring USB drive isolation and macOS bundle conventions.

## ADR-0035: Multi-Subject Exam Panels, Competencies with Grade Scoping, and Structural Forced Propagation
- **Status:** Accepted
- **Context:** Prior to Phase 11, panels were defined strictly as (Exam × Grade) with a hard-coded 2 setters + 1 reviewer composition, and teachers were restricted to teaching their assigned grades. Real-world high school operation (such as the manual schedule constructed by Coordinator Q) requires multiple subjects (e.g., Physics "VL" and Technology "CN"), subject-specific compositions (VL 2+1, CN 1+1, customizable min campuses), and qualifications where a teacher can review across grades outside their direct teaching assignment (`GradeScope::Any`), or serve as the sole specialized setter across all panels (e.g. Teacher Nghĩa setting all 12 CN panels).
- **Decision:**
  1. Introduce `Subject` entity with configurable `setters`, `reviewers`, and `min_campuses`.
  2. Redefine `PanelKey` as `(ExamId, GradeId, SubjectId)`, with seats identified by `(PanelKey, Role, position)`.
  3. Introduce `Competency` mapping `(TeacherId, SubjectId, Role, GradeScope)` per academic year, where `GradeScope::Taught` requires the teacher to teach the grade in that school year, and `GradeScope::Any` allows assignment across all grades.
  4. Implement fixpoint forced placement propagation (`find_forced_placements`): when a panel role has exactly as many eligible candidates as seats, those assignments are permanently forced, removing them from consideration in other roles for that panel, repeating until fixpoint. Forced placements are immutable in solver, optimizer, and manual drag-drop editing.
- **Consequences:** Accurate modeling of multi-subject schools; automated handling of specialized teachers without manual locks; zero hard violations when evaluating real school schedules under configurable process constraints.

## ADR-0036: Workload Quota Formula v2 with Capacity Bounds and Bisection Balancing
- **Status:** Accepted
- **Context:** The Phase 3 quota formula assumed equal distribution among active teachers with single-panel-per-exam limits. With multi-subject scheduling, per-exam task limits (Rule H4) allowing up to $M$ tasks (and setter tasks) per teacher per exam, manual quota overrides, and structural forced placements (e.g., 12 forced setter tasks for Teacher Nghĩa), the legacy quota calculation led to unachievable bounds and violated total demand $D = \sum \text{seats}$.
- **Decision:** Implement Quota Formula v2 in `crates/core::domain::quota`:
  1. For each teacher $t$, compute forced seats $F_t$, effective exam limit $\text{eff\_max\_tasks}(t, e)$, and capacity bound $\text{cap}_t = \min(\text{eligible\_seats}_t, \sum_{e \in \text{avail}_t} \text{eff\_max\_tasks}(t, e))$.
  2. For teachers without manual overrides, set $q_t(\lambda) = \text{clamp}(\lambda \cdot w'_t, F_t, \text{cap}_t)$, where $w'_t = \text{load\_weight}_t \times \text{avail}_t / E$.
  3. Solve for $\lambda$ using binary search / bisection until $\sum q_t(\lambda) = D$.
  4. Set integer tolerance bounds: $\text{lo}_t = \max(F_t, \lfloor q_t \rfloor - k)$ and $\text{hi}_t = \min(\text{cap}_t, \max(F_t, \lceil q_t \rceil + k))$.
  5. Add soft rules S9 (Exam Crowding Relief: penalizing teachers holding > 1 task in a single exam when unnecessary) and S10 (Review Subject Coverage: encouraging reviewer-qualified teachers to review every competent subject at least once).
- **Consequences:** Mathematically sound quota balancing that perfectly absorbs forced workloads, honors capacity limits and overrides, preserves exact backward compatibility with single-subject legacy databases, and optimizes cross-subject fairness.

## ADR-0037: Configurable Per-Exam and Per-Setter Task Limits (H4 Redefinition)
- **Status:** Accepted
- **Context:** Prior to Phase 11, hard constraint H4 strictly prohibited any teacher from serving on more than 1 panel in the same exam period. In multi-subject departments or specialized schools where some faculty members hold dual competencies (e.g. Physics and Technology), teachers must occasionally handle 2 assignments within one exam term (e.g. authoring a Physics paper and reviewing a Technology paper). A rigid binary cap rendered multi-subject instances infeasible.
- **Decision:** Redefine Rule H4 with configurable global parameters:
  - `max_tasks_per_exam` (default: 2, system upper limit per exam term).
  - `max_setter_per_exam` (default: 1, ensuring no teacher authors more than 1 paper per term).
  - Support per-teacher override `max_tasks_per_exam_override` in the database to accommodate specialized faculty (or medical/administrative restrictions).
- **Consequences:** Enables realistic multi-subject scheduling, guarantees authoring sanity by preventing multi-paper authoring in the same term, and gives coordinators granular per-teacher control.

## ADR-0038: S9 Avoidable Crowding Penalty with Teacher Unavoidable Offset
- **Status:** Accepted
- **Context:** Teachers whose total quota $c_t$ exceeds their available exam count $m_t$ (e.g. $c_t = 5$ tasks across $m_t = 4$ exams) must unavoidably hold at least $c_t - m_t$ doubled-up tasks in some exam. Penalizing gross crowding without accounting for this structural lower bound penalizes the optimizer for unavoidable reality, distorting objective function trade-offs against Coordinator Q's manual schedule.
- **Decision:** Formulate the S9 soft constraint strictly around avoidable crowding:
  $$\text{avoidable}_t = \max(0, \text{crowding}_t - \max(0, c_t - m_t))$$
  where $\text{crowding}_t = \sum_{e \in E} \max(0, \text{tasks}_{t, e} - 1)$.
  Total S9 penalty is $W_{S9} \sum_t \text{avoidable}_t$. When a plan achieves the theoretical minimum crowding for all teachers, S9 evaluates to exactly 0.0. Maintain $O(1)$ incremental state updates during local search.
- **Consequences:** Fair multi-objective optimization, correct assessment of Coordinator Q's manual plan (13 gross crowding, 4 offset, 9 avoidable units = 45.00 penalty), and 0.0 score when local search attains the structural minimum.

## ADR-0039: S10 Review Subject Diversity Penalty
- **Status:** Accepted
- **Context:** In departments managing multiple subjects, reviewer oversight should promote cross-disciplinary perspective and prevent siloed peer reviews. If a teacher qualified to review both Physics and Technology is only assigned to Physics reviews, the department loses cross-subject review balance.
- **Decision:** Introduce soft constraint S10 (`review_subject_coverage`):
  For each active teacher $t$ with reviewer competency in two or more subjects, penalize $W_{S10}$ for each competent subject in which teacher $t$ is assigned 0 review tasks across the academic year:
  $$\text{S10 penalty} = W_{S10} \sum_{t \in T_{\text{multi}}} \sum_{s \in \text{CompRev}(t)} [\text{reviews}_{t, s} == 0]$$
- **Consequences:** The provable lower bound is 0.0 whenever task counts allow each multi-competent teacher at least one review per subject. Drives simulated annealing to balance review distribution across disciplines.

## ADR-0040: Dynamic S1 Auto-Max Pacing Mode
- **Status:** Accepted
- **Context:** Rule S1 penalizes assignment concentration above an ideal threshold. In single-subject schools with 4 exams, 1 task per exam was the obvious target. In multi-subject schedules where average quota is 4.36 tasks, setting a hardcoded threshold of 1 or 2 either causes universal penalties or provides zero gradient.
- **Decision:** Support an "Auto" mode for Rule S1:
  - When `max_tasks_per_exam` is unset (default/Auto), the pacing threshold is dynamically calculated as $\lceil q_t \rceil$, penalizing only assignments that exceed the teacher's individual quota-paced expectation.
  - When an explicit integer is configured by the user, S1 penalizes assignments exceeding that specific integer.
- **Consequences:** Smooth, adaptive pacing across any faculty size, exam schedule, or multi-subject workload density without manual parameter recalibration.

## ADR-0041: Database Migration 0005 and Zero-Drift Legacy Equivalence
- **Status:** Accepted
- **Context:** Upgrading ExamPanel to support multiple subjects, competencies, teacher display names, and quota overrides required altering the SQLite schema. Existing v1–v4 databases and automated backup files must upgrade deterministically on startup or restore, and historical plans must maintain identical validity and score before and after migration.
- **Decision:** Create migration `0005_subjects_and_competencies.sql`:
  1. Add `subjects` table and create default record `"CHUNG"` (`setters=2, reviewers=1, min_campuses=2`).
  2. Add `competencies` table and auto-populate active teachers with `"CHUNG"` competency (`grade_scope='taught'`).
  3. Add `display_name`, `quota_override`, and `max_tasks_per_exam_override` to `teachers`.
  4. Add `subject_id` references to `assignments`, `locks`, and `unavailabilities`, backfilled to `"CHUNG"`.
  5. Enforce legacy equivalence via automated tests verifying that all historical plans have identical hard violations and penalty scores before and after applying migration 0005.
- **Consequences:** 100% backward compatibility, zero schema drift, and safe automated restoration of legacy backup archives.

## ADR-0042: Canonical Test Fixture Variants (`nocampus` and `synthetic_campuses`)
- **Status:** Accepted
- **Context:** Phase 11 previously introduced invented campuses with non-standard names ("Cơ sở 1/2"), conflicting with official school terminology (the school operates 4 phân hiệu). Furthermore, the actual academic schedule constructed by Coordinator Q operates in a unified setting without campus boundaries, while multi-campus constraint H3 requires deterministic synthetic distribution for testing.
- **Decision:** Establish two canonical dataset variants:
  1. `nocampus`: Single placeholder campus `"Chưa phân hiệu"`, Rule H3 disabled. Represents the authentic baseline for Coordinator Q's manual plan comparison.
  2. `synthetic_campuses`: Four campuses `"Phân hiệu 1"`, `"Phân hiệu 2"`, `"Phân hiệu 3"`, `"Phân hiệu 4"` (labeled `(giả lập)`), with deterministic assignment spread and Teacher Nghĩa in Phân hiệu 1. Used for H3 multi-campus verification and spatial diversity benchmarks.
  3. Permanently remove all "Cơ sở" terminology across code, fixtures, and tests, enforced by automated scanning in `test_no_forbidden_terms_in_fixtures_and_seeds`.
- **Consequences:** Consistent, reproducible benchmark environments matching authentic institutional structures, and strict adherence to prohibited term policies.


