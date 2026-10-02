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

### Semester Grouping by Exam Order
Exams within a school year are ordered by `sort_order` ASC.
The exams are partitioned into two semesters:
- **Semester 1 (Học kỳ 1):** The first $\lfloor N / 2 \rfloor$ exams (or for standard $N = 4$, exams 1 and 2: GK1 and CK1).
- **Semester 2 (Học kỳ 2):** The remaining exams (for $N = 4$, exams 3 and 4: GK2 and CK2).
If $N$ is odd (e.g. $N = 3$), Semester 1 contains $\lceil N / 2 \rceil$ exams, and Semester 2 contains the remaining $\lfloor N / 2 \rfloor$ exams.
Bulk unavailability actions ("Vắng cả học kỳ 1", "Vắng cả học kỳ 2", "Có mặt tất cả") apply this partition.

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

### 3.2 Soft Constraints (S1–S8)
Soft constraints guide schedule quality and optimization. Each constraint can be individually enabled/disabled and configured with a penalty weight $W \ge 0$.
All counts are evaluated over a complete school year plan. "Unit" = one incident; weighted penalty = weight $\times$ units. Each rule emits stable violation codes for user explanations.

| Code | Constraint Name | Description | Default Weight | Violation Codes | Formal Unit Metric |
|------|-----------------|-------------|----------------|-----------------|---------------------|
| **S1** | Reviewer Count | Reviewer assignments for eligible teachers with quota $q_t \ge 1$. | `10.0` | `reviewer_never`, `reviewer_too_many` | For each teacher with $q_t \ge 1$ eligible as reviewer: $(1 \text{ if reviews } = 0) + \max(0, \text{reviews} - 2)$. Params: $\min = 1, \max = 2$. |
| **S2** | Role Balance | Ratio of reviewer tasks to total tasks ($1/3$). | `3.0` | `role_imbalance` | For each teacher with $count_t \ge 2$, let $lo = \lfloor count_t / 3 \rfloor, hi = \lceil count_t / 3 \rceil$: distance from $reviews_t$ to $[lo, hi]$ (0 when inside). |
| **S3** | Independent Reviewer | Reviewer should not share campus with panel setters. | `4.0` | `reviewer_same_campus` | Per panel: number of setters sharing the reviewer's campus ($0 \dots 2$). |
| **S4** | Repeated Setter Pair | Distinctness of setter co-author pairs. | `6.0` | `setter_pair_repeated` | Per unordered setter pair $\{A, B\}$: $\max(0, \text{times\_together} - 1)$. |
| **S5** | Repeated Review Relation | Diversity of directed reviewer-to-author oversight. | `6.0` | `review_relation_repeated` | Per ordered pair $(\text{reviewer } A, \text{setter } B)$: $\max(0, \text{times} - 1)$. |
| **S6** | Consecutive Setting | Rest periods between heavy authoring duties across adjacent exams. | `2.0` | `setter_consecutive` | Per teacher, per pair of consecutive exams (by sort_order) where teacher is a SETTER in both: $1\text{ unit}$. |
| **S7** | Grade Rotation | Grade variety for multi-grade instructors. | `1.0` | `grade_not_rotated` | For teachers teaching $\ge 2$ grades in the year: $\max(0, \min(count_t, \|\text{grades}_t\|) - \text{distinct\_grades\_assigned}_t)$. |
| **S8** | Load Balance | Deviation from ideal availability-scaled quota $q_t$. | `8.0` | `load_deviation` | Per teacher: $(count_t - q_t)^2$. Pulls toward fair target even under H7 tolerance $k \ge 1$. |

### 3.3 Provable Lower Bounds
To inform human coordinators when schedule quality cannot be improved further, provable mathematical lower bounds are computed per rule:
- **S1 (Reviewer Capacity Pigeonhole):** Total reviewer slots $P = \text{exams} \times \text{grades}$. If eligible reviewers $N_{rev} > P$, at least $N_{rev} - P$ teachers cannot receive a review assignment (`reviewer_never`). If $P > 2 N_{rev}$, at least $P - 2 N_{rev}$ assignments exceed the maximum threshold of 2 (`reviewer_too_many`). Bound: $\max(0, N_{rev} - P) + \max(0, P - 2 N_{rev})$.
- **S6 (Setter Consecutive Pigeonhole):** With $E$ exams, a teacher can set at most $\lceil E/2 \rceil$ times without being assigned in adjacent exams. For $s_t$ setter assignments, the minimum unavoidable adjacent pairs is $\max(0, 2 s_t - E - 1)$.
  *Proof:* Any subset of $s_t$ exams out of $E$ decomposes into $k$ contiguous runs of setters. The number of adjacent pairs is $\sum_j (r_j - 1) = s_t - k$. Since there must be at least one non-setting exam between every pair of runs, the number of non-setting exams $E - s_t \ge k - 1 \implies k \le E - s_t + 1$. Also $k \le s_t$. Hence $k_{max} = \min(s_t, E - s_t + 1)$. Minimizing adjacent pairs yields $A_{min}(s_t, E) = s_t - k_{max} = \max(0, 2 s_t - E - 1)$.
  Specifically for $E = 4$:
  - $s_t \le 2$: 0 adjacent pairs (e.g., exams 1 and 3).
  - $s_t = 3$: $\ge 1$ adjacent pair ($\max(0, 2 \times 3 - 4 - 1) = 1$).
  - $s_t = 4$: 3 adjacent pairs ($\max(0, 2 \times 4 - 4 - 1) = 3$ because all 3 adjacent pairs (1,2), (2,3), (3,4) are unavoidably present).
  *(Note: A naive heuristic $\max(0, k - 2)$ would incorrectly predict $4 - 2 = 2$ for $k = 4$, whereas the exact bound is 3).*
  The aggregate lower bound is determined by greedily allocating total setter slots $S_{total} = 2 \times \text{panels}$ across teachers up to their setter capacities $cap_t = \min(hi_t, \text{eligible setter exams})$ minimizing $\sum_t \max(0, 2 s_t - E - 1)$. On the 11-teacher, 4-exam benchmark ($S_{total} = 24$), 11 teachers take 2 slots with 0 adjacent pairs ($11 \times 2 = 22$), forcing at least 2 teachers to take 3 slots ($2 \times 3 - 4 - 1 = 1$ pair each). Thus, this lower bound is provably 2.0 (while additional coupling constraints in practice may result in 3 teachers having 3 tasks, giving optimum $\ge 3$).
- **S8 (Discrete Optimal Load Balance):** The exact global minimum of $\sum_t (c_t - q_t)^2$ subject to $\sum c_t = D$ and $lo_t \le c_t \le hi_t$ is computed via greedy marginal-cost allocation over separable convex objectives. On the demo benchmark, this global discrete minimum is exactly $\approx 2.60$.
- **S2, S3, S4, S5, S7:** 0.0 unless proved otherwise by counting arguments.

### 3.4 Rule Presets
ExamPanel defines three canonical rule weight presets in `crates/core` as the single source of truth:
1. **Cân bằng (mặc định) / Balanced (Default):**
   Standard production balance between workload equality, team variety, and role health.
   - S1: `10.0`, S2: `3.0`, S3: `4.0`, S4: `6.0`, S5: `6.0`, S6: `2.0`, S7: `1.0`, S8: `8.0`.
   - H4 enabled (`100.0`), H7 tolerance = `1` (`100.0`).
2. **Ưu tiên công bằng khối lượng / Prioritize Workload Fairness:**
   Raises S8 and S1 weights to emphasize exact quota adherence and strict reviewer task bounds.
   - S1: `14.0`, S2: `3.0`, S3: `2.0`, S4: `3.0`, S5: `3.0`, S6: `2.0`, S7: `1.0`, S8: `16.0`.
   - H4 enabled (`100.0`), H7 tolerance = `1` (`100.0`).
3. **Ưu tiên đa dạng ê-kíp / Prioritize Team Diversity:**
   Raises S4, S5, and S3 weights to eliminate repeated authoring pairings and repeated oversight relations.
   - S1: `6.0`, S2: `3.0`, S3: `8.0`, S4: `12.0`, S5: `12.0`, S6: `2.0`, S7: `1.0`, S8: `4.0`.
   - H4 enabled (`100.0`), H7 tolerance = `1` (`100.0`).

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
| **F1** | `duplicate_campus_id` | Error | Duplicate campus ID | `campus_id` |
| **F1** | `duplicate_campus_code` | Error | Duplicate campus code | `code` |
| **F1** | `duplicate_grade_id` | Error | Duplicate grade ID | `grade_id` |
| **F1** | `duplicate_grade_code` | Error | Duplicate grade code | `code` |
| **F1** | `duplicate_exam_id` | Error | Duplicate exam ID | `exam_id` |
| **F1** | `duplicate_exam_code` | Error | Duplicate exam code for school year | `code` |
| **F1** | `duplicate_teacher_id` | Error | Duplicate teacher ID | `teacher_id` |
| **F1** | `unknown_campus_ref` | Error | Teacher references unknown campus | `teacher_id`, `campus_id` |
| **F1** | `load_weight_out_of_range` | Error | Teacher load weight is out of [0.0, 1.0] | `teacher_id`, `load_weight` |
| **F1** | `unknown_teacher_ref` | Error | Record references unknown teacher | `teacher_id`, `context` |
| **F1** | `unknown_grade_ref` | Error | Record references unknown grade | `grade_id`, `context` |
| **F1** | `unknown_exam_ref` | Error | Record references unknown exam | `exam_id`, `context` |
| **F1** | `duplicate_teacher_grade` | Error | Duplicate teacher-grade qualification entry | `teacher_id`, `grade_id` |
| **F1** | `duplicate_unavailability` | Error | Duplicate teacher unavailability entry | `teacher_id`, `exam_id` |
| **F1** | `duplicate_rule_key` | Error | Duplicate rule key in rule settings | `rule_key` |
| **F1** | `negative_rule_weight` | Error | Rule weight cannot be negative | `rule_key`, `weight` |
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

### 5.2 Stage 2: Simulated Annealing Local Search
- **API Signature:** `pub fn optimize(problem: &Problem, opts: &OptimizeOptions) -> Result<OptimizeResult, SolveError>`
- **Starting Point:** Valid feasible schedule from `solve_hard` (or caller-supplied initial plan).
- **Moves (Strictly Preserving H1–H7 Invariants; PINs Immovable; FORBID Respected):**
  - **M1 Replace:** Swap one panel slot's teacher with an eligible teacher not already assigned in that exam period (verifying H4, $[lo_t, hi_t]$ bounds for both teachers, and H3 campus diversity).
  - **M2 Intra-Exam Swap:** Exchange two teachers between two panels of the same exam period (roles may differ; respects H2, H3, H5, H6, H7).
  - **M3 Cross-Exam Swap:** Exchange two teachers between panels of different exam periods (checks H4 single-panel rule and availability for both teachers).
  - **M4 Role Swap:** Swap role (`Setter` $\leftrightarrow$ `Reviewer`) within the same panel (respects H6 role-specific locks and H2 reviewer eligibility).
- **Cooling Schedule & Parameters:**
  - Auto-calibration of initial temperature $T_0$ by sampling random candidate moves so that $\sim 80\%$ of worsening moves are initially accepted.
  - Geometric cooling: $T_{k+1} = \alpha \cdot T_k$.
  - Metropolis acceptance criterion: accept if $\Delta \le 0$ or with probability $\exp(-\Delta / T)$.
  - Termination by budget: `Budget::Iterations(n)` (deterministic) or `Budget::TimeMs(ms)`.
  - Cancellation and progress: `OptimizeOptions` accepts an `Arc<AtomicBool>` cancel flag and an optional `Fn(Progress) + Send + Sync` callback (`Progress { run, iteration, best_score, current_score, elapsed_ms }`), throttled to $\le 10$ invocations per second.

### 5.3 Multi-Plan Output and Diversity Selection
- **Parallel Multi-Run:** Runs $R$ independent simulated annealing runs (default $R = 8$) in parallel using pure-Rust thread scheduling (`rayon`), initialized from deterministic pseudo-random seeds derived from `base_seed`. Under `Budget::Iterations`, results are 100% deterministic regardless of thread scheduling.
- **Diversity Distance Metric:**
  Given two plans $P_1$ and $P_2$, each consisting of $D$ slot assignments $(p, t, r)$:
  $$d(P_1, P_2) = \frac{|\{ (p, t, r) \in P_1 \mid (p, t, r) \notin P_2 \}|}{D} \in [0.0, 1.0]$$
- **Ranked Selection:**
  - Selects up to $K$ plans (default $K = 3$) from the $R$ completed runs.
  - The plan with the lowest total penalty is always selected as Rank 1.
  - Subsequent plans are selected via greedy max-min diversity: at each step, select the candidate plan satisfying $d(P, P_j) \ge \tau$ (default threshold $\tau = 0.20$) for all already-selected plans $P_j$ that maximizes $\min_j d(P, P_j)$. If fewer than $K$ plans satisfy the threshold, return only the qualifying plans.
- **Result Structure (`OptimizeResult`):**
  - `plans: Vec<RankedPlan { rank: usize, seed: u64, assignments: Vec<Assignment>, report: ScoreReport }>`
  - `initial_report: ScoreReport`
  - `stats: OptimizeStats { total_runs: usize, total_iterations: u64, elapsed_ms: u64 }`

### 5.4 Determinism and Seed Reproducibility
PRNG seeds guarantee exact, bit-for-bit schedule reproducibility within a given software release and dependency graph. However, solver trajectories and intermediate scores for a given seed are not guaranteed to remain invariant across changes in underlying PRNG dependencies or library versions (e.g., changes in `rand` algorithms or feature flags). For long-term archival and auditability, users should persist the finalized assignment matrix, exported workbooks, or database backups.

---

## 6. Portable Storage Specification
- **Portable detection:** At startup, ExamPanel checks if the directory containing the running executable is writable.
  - If writable: Database is stored at `<exe_dir>/data/exam-panel.db`.
  - If not writable (e.g., system Program Files or read-only volume): Falls back to user application data directory (`%APPDATA%/ExamPanel/data` on Windows, `~/.local/share/ExamPanel/data` on Linux, `~/Library/Application Support/ExamPanel/data` on macOS).
- **Engine:** SQLite with standard rollback journal mode (`PRAGMA journal_mode = DELETE;`) and enforced foreign keys (`PRAGMA foreign_keys = ON;`), ensuring the entire database remains a single self-contained file suitable for USB and portable execution.
- **Plan Persistence (Migration 0002):**
  - `rank INTEGER`: Rank order among optimizer outputs (1 for global best).
  - `score_report_json TEXT`: Serialized `ScoreReport` containing complete penalty breakdown, per-teacher load statistics, and provable lower bounds.
  - `run_params_json TEXT`: Serialized audit metadata snapshot recording execution parameters (`base_seed`, `runs`, `budget`, `k`, `diversity_threshold`, and `app_version`).
  - `source TEXT NOT NULL DEFAULT 'optimizer'`: Provenance tag with database check constraint `CHECK (source IN ('optimizer', 'manual', 'duplicate'))`.

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

---

## 8. Glossary & Vietnamese Terminology Standards

To maintain consistency across user interface strings, documentation, and error reports, the following standardized terminology must be strictly used:

| English Term | Vietnamese Term | Context / Definition |
|---|---|---|
| campus | phân hiệu | Đơn vị phân hiệu / chi nhánh đào tạo (tuyệt đối không dùng "cơ sở" trong UI vi.json). |
| grade | khối | Khối lớp học sinh (ví dụ: Khối 10, Khối 11, Khối 12). |
| exam | kỳ thi | Đợt thi / kỳ thi trong năm học (ví dụ: GK1, CK1, GK2, CK2). |
| panel | ban đề | Ban ra đề thi cho 1 kỳ thi × 1 khối (gồm 2 người ra đề + 1 người phản biện). |
| setter | người ra đề | Giáo viên giữ vai trò biên soạn đề thi. |
| reviewer | người phản biện | Giáo viên giữ vai trò thẩm định, phản biện đề thi. |
| plan | phương án | Một lịch phân công hoàn chỉnh cho toàn bộ các ban đề trong năm học. |
| pin | ghim | Khóa cố định một phân công bắt buộc (LockKind::Pin). |
| forbid | cấm | Khóa ngăn cấm một phân công (LockKind::Forbid). |
| unavailability | vắng | Khoảng thời gian / kỳ thi giáo viên báo bận, không thể tham gia. |
| load weight | hệ số tải | Hệ số định mức công việc của giáo viên (0.0 đến 1.0). |
| school year | năm học | Năm học (ví dụ: 2026-2027). |
| feasibility | tính khả thi | Khả năng toán học thỏa mãn toàn bộ các điều kiện bắt buộc. |
| soft rule | tiêu chí ưu tiên | Quy tắc mềm hướng đến chất lượng và tính công bằng (S1–S8). |
| hard rule | điều kiện bắt buộc | Ràng buộc cứng bất di bất dịch (H1–H7). |
