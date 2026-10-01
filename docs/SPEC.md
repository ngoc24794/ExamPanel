# ExamPanel Business Specification

## Table of Contents
1. [Introduction and Overview](#1-introduction-and-overview)
2. [Domain Entities](#2-domain-entities)
3. [Constraint Specifications](#3-constraint-specifications)
   - [3.1 Hard Constraints (H1–H7)](#31-hard-constraints-h1h7)
   - [3.2 Soft Constraints (S1–S7)](#32-soft-constraints-s1s7)
4. [Pre-Solve Feasibility Verification](#4-pre-solve-feasibility-verification)
5. [Solver Architecture & Optimization Pipeline](#5-solver-architecture--optimization-pipeline)
6. [Portable Storage Specification](#6-portable-storage-specification)
7. [Reference Scale and Benchmark Profile](#7-reference-scale-and-benchmark-profile)

---

## 1. Introduction and Overview
ExamPanel is a portable, cross-platform desktop application designed for secondary education institutions to automate the assignment of teachers to exam paper drafting (setting) and review panels across a full academic year.

Exam panel construction in schools requires satisfying multi-campus diversity, teacher grade qualifications, equitable workload quotas, conflict-of-interest prohibitions, and review balance. Manual scheduling is labor-intensive and frequently leads to schedule collisions and reviewer burnout. ExamPanel solves this problem using a two-stage hybrid constraint satisfaction and local search optimization solver.

---

## 2. Domain Entities

### Campus
- Represents a physical school branch or campus location.
- **Attributes:** Unique ID, name, code, active status.
- Used to guarantee institutional diversity across exam review panels.

### Teacher
- Represents a teaching staff member eligible for exam paper assignments.
- **Attributes:**
  - `id`: Unique identifier (64-bit integer ID).
  - `full_name`: Teacher's full name.
  - `campus_id`: Foreign key referencing the teacher's primary campus.
  - `grades_taught`: Set of grades the teacher is qualified and assigned to teach in a specific school year (`teacher_grades` relation: qualifications are maintained per school year and can change between academic years).
  - `load_weight`: Normalized floating-point value in range `[0.0, 1.0]`. A weight of `1.0` denotes standard full workload; `0.0` denotes complete exemption (e.g., department heads, maternity leave, sabbatical).
  - `active`: Boolean status flag. Inactive teachers are excluded from assignment cycles.
  - `note`: Optional administrative notes.

### School Year
- Represents the academic year under management (e.g., 2026–2027).
- Houses all exam sessions, teacher rosters, constraints, and generated plans.

### Exam
- Represents an exam term during the school year.
- **Default terms:**
  - `GK1`: Midterm 1 (Giữa kỳ 1)
  - `CK1`: Final 1 (Cuối kỳ 1)
  - `GK2`: Midterm 2 (Giữa kỳ 2)
  - `CK2`: Final 2 (Cuối kỳ 2)
- Ordered sequentially: `GK1 -> CK1 -> GK2 -> CK2`.

### Grade
- Represents a cohort grade level.
- **Defaults:** Grades `10`, `11`, `12`.
- Configurable per institution.

### Panel
- Every `(Exam, Grade)` pair of the school year is an exam commission / panel (e.g. 4 exams × 3 grades = 12 panels by default).
- Consists of exactly three roles:
  - 2 **SETTER**s (Ra đề): Teachers responsible for authoring the exam questions.
  - 1 **REVIEWER** (Phản biện): Teacher responsible for peer-reviewing and validating the paper.

### Eligibility
Teacher `t` is eligible for panel `p = (exam e, grade g)` in role `r` if:
- `t.active` is true;
- `t.load_weight > 0.0`;
- `t` is qualified to teach grade `g` in this school year (`teacher_grades` relation);
- `t` is not marked unavailable for exam `e` (`unavailability` relation);
- No `FORBID` lock matches `(e, g, t, r)` or `(e, g, t, any role)`.

### Plan
- A full candidate schedule across all panels for the entire school year.
- Evaluated on feasibility (100% hard constraints satisfied) and objective fitness score (penalty sum of soft constraints).

### Assignment
- A single record associating a Teacher to a specific Panel with an assigned Role (`SETTER` or `REVIEWER`).

### Lock (PIN / FORBID)
- Manual user override:
  - `PIN`: Mandatory assignment. Forces teacher `T` into panel `P` with role `R` (or any role if `role` is null, where the role is chosen by the solver). PINs count toward `lo_t` / `hi_t` quotas like any other assignment.
  - `FORBID`: Prohibited assignment. Forbids teacher `T` from panel `P` (or role `R` on panel `P`).

### Unavailability
- Per-exam unavailability flag or reason. Indicates that a teacher cannot be scheduled for any panel in a designated exam period (e.g., medical leave, official duty).

---

## 3. Constraint Specifications

### 3.1 Hard Constraints (H1–H7)
All hard constraints must be strictly satisfied for a plan to be valid.

| Code | Constraint Name | Description | Default Setting |
|------|-----------------|-------------|-----------------|
| **H1** | Panel Composition | Each panel has exactly 2 `SETTER`s and 1 `REVIEWER`, comprising 3 distinct teachers. | Enforced |
| **H2** | Grade Qualification | A teacher can only be assigned to a panel for a grade that they currently teach in the school year. Assigned teachers must be active and have `load_weight > 0`. | Enforced |
| **H3** | Multi-Campus Diversity | Each panel must include teachers from at least 2 distinct campuses. | Enforced |
| **H4** | Single Panel Per Exam | A teacher sits on at most 1 panel per exam (can be toggled/disabled in institution settings). | Enabled (`weight = 100.0`) |
| **H5** | Exam Availability | Teachers marked unavailable for a given exam must not be assigned to any panel in that exam. | Enforced |
| **H6** | Lock Compliance | All manual `PIN` and `FORBID` locks must be strictly obeyed. PIN without role fixes the teacher on the panel with role chosen by the solver. | Enforced |
| **H7** | Workload Quota | Annual teacher assignments must stay within `[lo_t, hi_t]` derived from availability-scaled quota formulas with tolerance `k` (default `1`). Teachers with `load_weight = 0` have `lo = hi = 0`. | Enabled (`tolerance = 1`, `weight = 100.0`) |

#### Workload Quota Formula (H7):
- $D = \text{number of panels} \times 3$ (total required slot assignments).
- $\text{availability}_t = \text{number of exams where teacher } t \text{ is available and eligible for at least one grade}$.
- Effective weight: $w'_t = \text{load\_weight}_t \times \frac{\text{availability}_t}{\text{number of exams}}$ (0 if teacher is not eligible anywhere).
- Base quota: $q_t = D \times \frac{w'_t}{\sum w'}$.
- Tolerance bounds with parameter $k$ (default 1):
  $$\text{lo}_t = \max(0, \lfloor q_t \rfloor - k)$$
  $$\text{hi}_t = \lceil q_t \rceil + k$$
  Then clamp $\text{hi}_t$ to the maximum achievable: $\text{hi}_t \le \text{availability}_t$ when H4 is enabled (and $\le D$). A teacher with $w'_t = 0$ has $\text{lo}_t = \text{hi}_t = 0$.
- *Note:* $k = 0$ is the strictest fair setting. Phase 4 adds a soft "load deviation" penalty that pulls each teacher toward $q_t$ even when $k \ge 1$.

### 3.2 Soft Constraints (S1–S7)
Soft constraints guide schedule quality. Each constraint can be individually enabled/disabled and configured with an integer penalty weight `W >= 0`.

| Code | Constraint Name | Description | Default Weight | Initial Rationale |
|------|-----------------|-------------|----------------|-------------------|
| **S1** | Reviewer Frequency | Each active teacher acts as a reviewer at least once and at most twice throughout the year. | `10.0` | Highest soft priority: ensures every teacher shares reviewing responsibility. |
| **S2** | Role Ratio Balance | Maintain approximately a 2:1 ratio between setter roles and reviewer roles per teacher (`~2 SETTER + 1 REVIEWER`). | `5.0` | Balanced role experience across teaching staff. |
| **S3** | Campus Reviewer Independence | The reviewer should originate from a different campus than both setters on the panel. | `4.0` | Enhances institutional impartiality during review. |
| **S4** | Setter Pair Diversity | The same pair of teachers should not be co-setters on multiple panels within the same academic year. | `6.0` | Promotes collaborative diversity among exam authors. |
| **S5** | Reciprocal Review Avoidance | Minimize instances where Teacher A reviews Teacher B's exam paper more than once in the year. | `6.0` | Eliminates circular review cliques. |
| **S6** | Consecutive Exam Relief | Minimize back-to-back panel assignments across two consecutive exam terms for the same teacher (e.g., avoid `GK1` followed immediately by `CK1` if idle alternatives exist). | `2.0` | Ergonomic workload pacing across terms. |
| **S7** | Multi-Grade Rotation | Teachers qualified to teach multiple grades should rotate across those grades rather than remaining fixed to a single grade all year. | `1.0` | Staff development and varied grade experience. |


---

## 4. Pre-Solve Feasibility Verification
Before launching the constructive solver, a deterministic pre-check verifies whether input parameters mathematically permit a valid solution.
> **Note:** These checks are **necessary conditions**; passing them does not guarantee that a valid schedule exists, but failing any check guarantees infeasibility.

### 4.1 Verification Stages
1. **F1 Structural Integrity**: Snapshot validation (`Problem::validate()`).
2. **F2 Per-Panel Adequacy**: Each panel must have $\ge 2$ setter-eligible, $\ge 1$ reviewer-eligible, $\ge 3$ distinct eligible teachers, and the eligible set must span $\ge 2$ distinct campuses.
3. **F3 Lock Consistency**:
   - `PIN` and `FORBID` conflict detection for the same teacher on the same panel.
   - Panel lock limits ($\le 2$ pinned setters, $\le 1$ pinned reviewer, $\le 3$ pins total).
   - Ineligible pinned teachers (not teaching grade, unavailable for exam term, inactive, or load weight 0).
   - Teacher pinned in multiple panels of the same exam period when H4 is enabled.
   - Pinned campus monopoly (3 pins in a panel from the same campus).
   - Teacher pins exceeding their calculated upper quota $hi_t$.
4. **F4 Capacity & Max-Flow**:
   - Aggregate capacity bounds: $\sum lo_t \le D \le \sum hi_t$.
   - Per-exam bottleneck max-flow under H4: constructs a bipartite network ($S \to \text{teachers with cap 1} \to \text{eligible panels of exam with cap 1} \to \text{panels with cap 3} \to T$) using Dinic's algorithm to prove whether all panels of the exam can be simultaneously staffed.
5. **Non-blocking Warnings**:
   - Panels with exactly 3 eligible teachers (tight margin).
   - Teachers with $q_t > 0$ eligible for only a single panel.
   - Restricted reviewer-eligible pool (affects soft rule S1).

### 4.2 Diagnostic Code Catalog

| Category | Diagnostic Code | Severity | Description | Interpolated Parameters |
|----------|-----------------|----------|-------------|-------------------------|
| **F1** | `structural_error` | Error | Structural data inconsistency in snapshot | `message` |
| **F2** | `insufficient_setters` | Error | Fewer than 2 setter-eligible teachers for panel | `exam`, `grade`, `count` |
| **F2** | `insufficient_reviewers` | Error | No reviewer-eligible teachers for panel | `exam`, `grade`, `count` |
| **F2** | `insufficient_panel_teachers` | Error | Fewer than 3 distinct eligible teachers for panel | `exam`, `grade`, `count` |
| **F2** | `insufficient_campuses` | Error | Eligible teachers span fewer than 2 campuses | `exam`, `grade`, `count` |
| **F3** | `lock_conflict` | Error | Teacher has both PIN and FORBID locks on panel | `exam`, `grade` |
| **F3** | `excess_pinned_setters` | Error | Panel has more than 2 pinned setters | `exam`, `grade`, `count` |
| **F3** | `excess_pinned_reviewers` | Error | Panel has more than 1 pinned reviewer | `exam`, `grade`, `count` |
| **F3** | `excess_pins_in_panel` | Error | Panel has more than 3 total pinned teachers | `exam`, `grade`, `count` |
| **F3** | `pinned_teacher_ineligible` | Error | Pinned teacher is unqualified, unavailable, or inactive | `exam`, `grade` |
| **F3** | `pinned_teacher_multiple_panels` | Error | Teacher pinned in multiple panels of one exam under H4 | `exam`, `count` |
| **F3** | `pinned_campus_monopoly` | Error | All 3 pinned teachers belong to the same campus | `exam`, `grade` |
| **F3** | `pinned_quota_exceeded` | Error | Pinned assignments exceed teacher's max quota ($hi_t$) | `count`, `hi` |
| **F4** | `insufficient_total_capacity` | Error | Aggregate max capacity ($\sum hi$) is less than total slots $D$ | `total_slots`, `max_capacity` |
| **F4** | `excess_minimum_capacity` | Error | Aggregate min capacity ($\sum lo$) exceeds total slots $D$ | `total_slots`, `min_capacity` |
| **F4** | `exam_capacity_infeasible` | Error | Max-flow through exam is strictly less than required slots | `exam`, `max_flow`, `required` |
| **F4** | `panel_unfillable_under_h4` | Error | Panel cannot reach 3 assignments under single-panel rule H4 | `exam`, `grade`, `flow`, `required` |
| **Warn** | `tight_panel_roster` | Warning | Panel has exactly 3 eligible teachers (zero substitution slack) | `exam`, `grade`, `count` |
| **Warn** | `teacher_single_panel_eligibility` | Warning | Teacher with positive quota is eligible for only 1 panel | `teacher_name`, `count` |
| **Warn** | `restricted_reviewer_pool` | Warning | Total reviewer-eligible teachers is less than active staff | `reviewer_count`, `teacher_count` |


---

## 5. Solver Architecture & Optimization Pipeline

The solver runs in pure Rust (`crates/core`) with zero UI or database coupling:

### 5.1 Stage 1: Hard-Constraint Backtracking Solver
- **API Signature:** `pub fn solve_hard(problem: &Problem, opts: &SolveOptions) -> Result<Solution, SolveError>`
- **Options (`SolveOptions`):**
  - `seed: u64`: PRNG seed for deterministic reproducibility.
  - `time_limit_ms: u64` (default `2000`): Maximum search duration.
  - `max_nodes: u64` (default `2_000_000`): Maximum search tree nodes explored.
- **Result Types:**
  - `Solution`: `assignments: Vec<Assignment>`, `seed: u64`, `stats: SolveStats { nodes, backtracks, elapsed_ms }`.
  - `SolveError::Infeasible { report: FeasibilityReport }`: Pre-solve checks proved input is mathematically impossible.
  - `SolveError::Exhausted { stats, unfilled_panels }`: Search space completely explored with no valid solution.
  - `SolveError::LimitReached { stats, best_partial }`: Reached time or node limit before proving a solution.
- **Algorithm Invariants:**
  - Mandatory execution of `check_feasibility` prior to search; halts immediately on errors.
  - Pre-placement of all manual `PIN` locks (with fixed or solver-chosen roles).
  - Dense indexing and fixed-width teacher bitsets (`TeacherBitSet` up to 256 teachers).
  - Minimum Remaining Values (MRV) panel selection heuristic (panel with fewest valid candidate triples chosen first).
  - Symmetrical setter deduplication ($s_1 < s_2$).
  - Forward checking on upper bounds ($hi_t$), single-panel-per-exam (H4), non-emptiness of remaining panel triples, and lower bound reachability ($\sum \max(0, lo_t - used_t) \le \text{remaining slots}$).
  - Value ordering biased toward teachers furthest below their availability quota ($q_t - used_t$) with PRNG jitter. Same seed strictly yields identical assignments.

### 5.2 Stage 2: Simulated Annealing (Phase 4 Local Search)
   - Operates on valid candidate plans to minimize total soft constraint penalties.
   - **Neighborhood operators:**
     - `SwapTeachers`: Swap two teachers between compatible panels.
     - `SwapRoles`: Swap role (Setter ↔ Reviewer) within the same panel if constraints permit.
     - `ReplaceIdle`: Replace an assigned teacher with a qualified idle teacher.
   - Cooling schedule with exponential temperature decay and Metropolis acceptance criterion.

3. **Multi-Seed Parallel Execution**
   - Spawns multi-threaded worker seeds.
   - Gathers top results, deduplicates schedules, and returns the top 3–5 distinct plans.
   - Each returned plan includes an itemized breakdown of soft constraint satisfaction and penalties.

---

## 6. Portable Storage Specification
- **Portable detection:** At startup, ExamPanel checks if the directory containing the running executable is writable.
  - If writable: Database is stored at `<exe_dir>/data/exam-panel.db`.
  - If not writable (e.g., system Program Files or read-only volume): Falls back to user application data directory (`%APPDATA%/ExamPanel/data` on Windows, `~/.local/share/ExamPanel/data` on Linux, `~/Library/Application Support/ExamPanel/data` on macOS).
- **Engine:** SQLite with standard rollback journal mode (`PRAGMA journal_mode = DELETE;`) and enforced foreign keys (`PRAGMA foreign_keys = ON;`), ensuring the entire database remains a single self-contained file suitable for USB and portable execution.

---

## 7. Reference Scale and Benchmark Profile
Standard institutional benchmark scenario:
- **Teachers:** 11 teachers.
- **Campuses:** 4 campuses.
- **Exams:** 4 exam terms (`GK1`, `CK1`, `GK2`, `CK2`).
- **Grades:** 3 grades (`10`, `11`, `12`).
- **Total Panels:** 4 exams × 3 grades = 12 panels.
- **Total Assignments:** 12 panels × 3 teachers = 36 assignments (24 setters, 12 reviewers).
- **Target Solve Time:** < 500 ms for feasibility check and Stage 1; < 3 seconds for Stage 2 optimization.
