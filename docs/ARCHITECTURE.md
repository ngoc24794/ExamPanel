# ExamPanel Architecture Documentation

## 1. System Overview
ExamPanel is architected as a modular, decoupled desktop application separating pure domain algorithms, data persistence, the platform shell, and the user interface.

```mermaid
flowchart TD
    subgraph Frontend["Frontend Layer (React + TypeScript)"]
        UI[UI Components & Layout]
        State[State & Hooks]
        I18n[i18n: vi / en]
        API_LAYER[ExamPanelApi Interface]
        MOCK[Mock Implementation]
        TAURI_CLIENT[Tauri IPC Client]

        UI --> State
        UI --> I18n
        State --> API_LAYER
        API_LAYER -->|Browser Dev| MOCK
        API_LAYER -->|Desktop App| TAURI_CLIENT
    end

    subgraph Shell["Desktop Shell (Tauri 2)"]
        TAURI_SHELL[src-tauri Shell]
        IPC_HANDLERS[Tauri Command Handlers]

        TAURI_CLIENT -.->|IPC Invoke| IPC_HANDLERS
        IPC_HANDLERS --> TAURI_SHELL
    end

    subgraph Backend["Rust Workspace Backend"]
        CORE[crates/core\nDomain + Feasibility + Solver\nPure Rust, No Tauri/DB]
        STORAGE[crates/storage\nSQLite + Migrations + Paths\nPortable Path Resolution]

        TAURI_SHELL --> CORE
        TAURI_SHELL --> STORAGE
        STORAGE -.->|References Domain Types| CORE
    end

    subgraph StorageEngine["Local File System"]
        DB[(exam-panel.db SQLite)]
        STORAGE --> DB
    end
```

---

## 2. Layer Responsibilities & Boundaries

### 2.1 `crates/core` (Domain & Algorithm Core)
- **Zero External Shell / DB Dependencies:** Does not depend on `tauri`, `rusqlite`, or any I/O framework. Pure algorithmic domain core.
- **Components:**
  - `domain`: Core immutable domain types (`Teacher`, `Campus`, `Exam`, `Grade`, `PanelKey`, `Lock`, `Assignment`, `Problem`, `TeacherQuota`).
  - `validate`: Source of truth for hard-constraint compliance (H1–H7); validates both complete and partial assignments.
  - `feasibility`: Pre-solve mathematical consistency checks; includes in-house Dinic bipartite max-flow and generates structured `Diagnostic` items with i18n parameter maps.
  - `solver`:
    - Stage 1: Deterministic MRV-directed randomized backtracking with forward checking for 100% hard-constraint satisfaction.
    - Stage 2 (Phase 4): Simulated annealing local search for soft-constraint optimization.

```mermaid
flowchart TD
    ProblemSnapshot["Problem Snapshot\n(Roster, Exams, Grades, Locks, Rules)"]

    subgraph CoreEngine["crates/core"]
        QUOTA["domain::quota\nAvailability-Scaled Quotas (H7)"]
        FEAS["feasibility::check_feasibility\nF1 Structural + F2 Per-Panel\n+ F3 Locks + F4 Dinic Max-Flow"]
        SOLVER["solver::solve_hard\nMRV Backtracking + Forward Checking\nSeeded PRNG Value Ordering"]
        VALIDATOR["validate::validate_assignments\nHard-Constraint Source of Truth\nH1..H7 Verification (Complete/Partial)"]

        ProblemSnapshot --> QUOTA
        ProblemSnapshot --> FEAS
        QUOTA --> FEAS
        QUOTA --> SOLVER
        FEAS -->|is_feasible == false| InfeasibleReport["SolveError::Infeasible\n(FeasibilityReport with Diagnostics)"]
        FEAS -->|is_feasible == true| SOLVER
        SOLVER --> Solution["Solution\n(assignments, seed, stats)"]
        Solution -.->|Cross-Verify| VALIDATOR
    end
```

### 2.2 `crates/storage` (Data Access & Persistence)
- **Technology:** `rusqlite` with the `bundled` feature (embedded SQLite engine).
- **Pragmas & Storage Invariants:**
  - Foreign key constraint enforcement (`PRAGMA foreign_keys = ON;`).
  - Standard rollback journal mode (`PRAGMA journal_mode = DELETE;`), guaranteeing the SQLite database remains a single, completely self-contained file without `-wal` or `-shm` sidecars for seamless USB and portable execution.
- **Components:**
  - `paths`: Resolves portable database path (`./data/exam-panel.db` if directory is writable; OS app-data directory fallback otherwise).
  - `migrations`: Embedded schema migration management using incremental SQL scripts bundled with `include_str!`, executed within transactions and tracked via `PRAGMA user_version`. Idempotent across re-runs.
  - `store`: Strongly-typed CRUD operations mapping between SQLite tables and `core` domain structs, with structured error handling (`StorageError` distinguishing `NotFound`, `Constraint`, `Sqlite`, `Io`, and `Serialization`).
  - `seeds`: Idempotent default seeding (`seed_defaults` for grades and UI settings) and complete test/development fixture datasets (`seed_demo`).

### 2.3 `src-tauri` (Desktop Application Shell)
- **Tauri 2 Framework:** Thin platform integration wrapper.
- **Isolation:** Excluded from the default Cargo workspace members (`default-members`) to guarantee that headless CI environments and pure backend development can compile and test without desktop system GUI libraries.
- **Identifier:** `vn.exampanel.app` (configurable in `tauri.conf.json` and `lib.rs`).
- **Commands:** Thin routing layer deserializing IPC arguments, calling `core`/`storage`, and returning JSON payloads.

### 2.4 `ui` (Presentation Layer)
- **Stack:** React 18, TypeScript, Vite, Tailwind CSS, shadcn/ui.
- **Dual Runtime Support:**
  - `ExamPanelApi` interface decouples the UI from Tauri IPC.
  - When running in browser (`pnpm dev`), automatically activates `mock.ts`.
  - When running inside Tauri, automatically activates `tauri.ts` (`@tauri-apps/api/core`).
- **Internationalization (i18n):** `i18next` with default Vietnamese (`vi`) and secondary English (`en`). Zero hard-coded UI strings.
- **Theme Engine:** `light`, `dark`, and `system` modes using semantic CSS tokens (`hsl(var(--...))`).

---

## 3. Data Flow

### 3.1 Plan Generation Flow
```mermaid
sequenceDiagram
    participant User
    participant UI as React UI
    participant IPC as Tauri IPC Bridge
    participant Storage as crates/storage
    participant Core as crates/core (Solver)

    User->>UI: Click "Generate Assignments"
    UI->>IPC: invoke("generate_plans", { schoolYearId, options })
    IPC->>Storage: Load teachers, panels, rules, locks
    Storage-->>IPC: Roster & Constraint Data
    IPC->>Core: feasibility::check(roster, rules)
    alt Feasibility check fails
        Core-->>IPC: FeasibilityError { key, params }
        IPC-->>UI: Localized diagnostic message
        UI-->>User: Display actionable resolution prompt
    else Feasibility check passes
        IPC->>Core: solver::solve(roster, rules, seedCount)
        Core-->>IPC: Top 3-5 distinct candidate plans
        IPC->>Storage: Persist candidate plans
        IPC-->>UI: Return plans with soft-constraint scores
        UI-->>User: Present interactive comparison view
    end
```

### 3.2 Problem Snapshot & Pre-Solve Validation Flow
```mermaid
sequenceDiagram
    participant IPC as Tauri IPC Command
    participant Store as crates/storage (Store)
    participant Core as crates/core (Problem)

    IPC->>Store: load_problem(school_year_id)
    Store->>Store: Query school_year, campuses, grades
    Store->>Store: Query active teachers & teacher_grades for year
    Store->>Store: Query exams, unavailabilities, locks, rule_settings
    Store-->>IPC: Problem snapshot struct
    IPC->>Core: problem.validate()
    alt Validation has structural errors
        Core-->>IPC: Vec<ValidationError>
        IPC-->>UI: Return validation error list to user
    else Structural integrity holds
        IPC->>Core: feasibility::check(&problem) / solver::solve(&problem)
    end
```

### 3.3 Dual-Mode API Resolution
```mermaid
flowchart TD
    Start[App Starts] --> CheckEnv{Is window.__TAURI_INTERNALS__ present?}
    CheckEnv -->|Yes| UseTauri[Initialize TauriExamPanelApi]
    CheckEnv -->|No| UseMock[Initialize MockExamPanelApi]
    UseTauri --> API[Expose unified ExamPanelApi]
    UseMock --> API
    API --> UIComponents[UI Components call api.ping, api.getTheme, etc.]
```

---

## 4. Architectural Invariants
1. **Purity of Core:** Any modification to `crates/core` must not introduce file I/O, network I/O, or SQLite dependencies.
2. **Deterministic Feasibility Checks:** Feasibility check failures must always explain *why* the configuration is invalid and name the exact exam, grade, or teacher group causing the conflict.
3. **Data Portability:** Storage location resolution must always prioritize adjacent `./data/` directories when write permissions exist, enabling USB/folder portability without installer lock-in.
4. **Zero String Hardcoding:** Every UI text label, notification, table header, or error message must resolve through `t('path.key')`.
