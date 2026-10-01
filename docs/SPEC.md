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
  - `id`: Unique identifier (UUID or integer).
  - `full_name`: Teacher's full name.
  - `campus_id`: Foreign key referencing the teacher's primary campus.
  - `grades_taught`: Set of grades the teacher is certified and active to teach (e.g. `[10, 11]`).
  - `load_weight`: Normalized floating-point value in range `[0.0, 1.0]`. A weight of `1.0` denotes standard full workload; `0.0` denotes complete exemption (e.g., department heads, maternity leave, sabbatical).
  - `active`: Boolean status flag. Inactive teachers are excluded from assignment cycles.

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
- An exam commission for an (Exam × Grade) pair (e.g., `GK1` × Grade `10`).
- Consists of exactly three roles:
  - 2 **SETTER**s (Ra đề): Teachers responsible for authoring the exam questions.
  - 1 **REVIEWER** (Phản biện): Teacher responsible for peer-reviewing and validating the paper.

### Plan
- A full candidate schedule across all panels for the entire school year.
- Evaluated on feasibility (100% hard constraints satisfied) and objective fitness score (penalty sum of soft constraints).

### Assignment
- A single record associating a Teacher to a specific Panel with an assigned Role (`SETTER` or `REVIEWER`).

### Lock (PIN / FORBID)
- Manual user override:
  - `PIN`: Mandatory assignment. Forces teacher `T` into panel `P` with role `R`.
  - `FORBID`: Prohibited assignment. Forbids teacher `T` from panel `P` (or role `R` on panel `P`).

### Unavailability
- Per-exam unavailability flag or reason. Indicates that a teacher cannot be scheduled for any panel in a designated exam period (e.g., medical leave, official duty).

---

## 3. Constraint Specifications

### 3.1 Hard Constraints (H1–H7)
All hard constraints must be strictly satisfied for a plan to be valid.

| Code | Constraint Name | Description |
|------|-----------------|-------------|
| **H1** | Panel Composition | Each panel has exactly 2 `SETTER`s and 1 `REVIEWER`, comprising 3 distinct teachers. |
| **H2** | Grade Qualification | A teacher can only be assigned to a panel for a grade that they currently teach. |
| **H3** | Multi-Campus Diversity | Each panel must include teachers from at least 2 distinct campuses. |
| **H4** | Single Panel Per Exam | A teacher sits on at most 1 panel per exam (can be toggled/disabled in institution settings). |
| **H5** | Exam Availability | Teachers marked unavailable for a given exam must not be assigned to any panel in that exam. |
| **H6** | Lock Compliance | All manual `PIN` and `FORBID` locks must be strictly obeyed. |
| **H7** | Workload Quota | Each teacher's total assignments over the year must fall within `[floor(quota), ceil(quota)]` derived from their `load_weight` with a configurable tolerance (default `±1`). Teachers with `load_weight = 0` are exempt (0 assignments). |

### 3.2 Soft Constraints (S1–S7)
Soft constraints guide schedule quality. Each constraint can be individually enabled/disabled and configured with an integer penalty weight `W >= 0`.

| Code | Constraint Name | Description |
|------|-----------------|-------------|
| **S1** | Reviewer Frequency | Each active teacher acts as a reviewer at least once and at most twice throughout the year. |
| **S2** | Role Ratio Balance | Maintain approximately a 2:1 ratio between setter roles and reviewer roles per teacher (`~2 SETTER + 1 REVIEWER`). |
| **S3** | Campus Reviewer Independence | The reviewer should originate from a different campus than both setters on the panel. |
| **S4** | Setter Pair Diversity | The same pair of teachers should not be co-setters on multiple panels within the same academic year. |
| **S5** | Reciprocal Review Avoidance | Minimize instances where Teacher A reviews Teacher B's exam paper more than once in the year. |
| **S6** | Consecutive Exam Relief | Minimize back-to-back panel assignments across two consecutive exam terms for the same teacher (e.g., avoid `GK1` followed immediately by `CK1` if idle alternatives exist). |
| **S7** | Multi-Grade Rotation | Teachers qualified to teach multiple grades should rotate across those grades rather than remaining fixed to a single grade all year. |

---

## 4. Pre-Solve Feasibility Verification
Before launching the compute-intensive solver, a deterministic pre-check verifies whether input parameters permit a valid solution:
1. **Teacher Roster Adequacy**: For every `(Exam, Grade)` pair, there must be at least 3 qualified, available teachers belonging to at least 2 distinct campuses.
2. **Quota Consistency**: The total demand of panel slots (`panels × 3`) must be mathematically satisfiable by the aggregate available teacher quotas.
3. **Lock Consistency**:
   - Conflicting locks (e.g. `PIN` and `FORBID` on the same teacher/panel) are detected.
   - At most 2 `PIN` setters and at most 1 `PIN` reviewer per panel.
4. **Actionable Error Reporting**: If a violation is found, the system halts and returns a localized, parameterized error message (e.g., `CK2 – Grade 12: only 2 teachers available from 1 campus`).

---

## 5. Solver Architecture & Optimization Pipeline

The solver runs in pure Rust (`crates/core`) with zero UI or database coupling:

1. **Stage 1: Randomized Backtracking (Constructive Phase)**
   - Orders unassigned panels by Most-Constrained-Panel first (MRV - Minimum Remaining Values heuristic).
   - Generates an initial assignment satisfying 100% of hard constraints (H1–H7).
   - Applies random restarts and seed exploration.

2. **Stage 2: Simulated Annealing (Local Search Optimization)**
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
- **Engine:** SQLite with WAL mode (`PRAGMA journal_mode = WAL;`) and enforced foreign keys (`PRAGMA foreign_keys = ON;`).

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
