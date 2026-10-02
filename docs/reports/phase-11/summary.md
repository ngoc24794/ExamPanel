# Phase 11 Validation Report: Q's Real Manual Table and Optimizer Benchmark

## 1. Canonical Dataset Overview
- **Teachers**: 12 (11 Vật lí teachers + Thầy Nghĩa)
- **Campuses**: 2 (Cơ sở 1, Cơ sở 2)
- **Subjects**: 2
  - **VL (Vật lí)**: 2 setters + 1 reviewer, min 2 campuses
  - **CN (Công nghệ)**: 1 setter + 1 reviewer, min 2 campuses
- **Exams**: 4 terms (GK1, CK1, GK2, CK2)
- **Grades**: 3 (Khối 10, 11, 12)
- **Panels**: 4 exams × 3 grades × 2 subjects = 24 panels
- **Seats**: (12 VL × 3) + (12 CN × 2) = 36 + 24 = 60 seats

## 2. Invariants & Q's Manual Table Validation
- **Total seats placed**: 60
- **Thầy Nghĩa**: 12 tasks (all CN setter), 0 reviews (0)
- **Cô Quí**: 4 reviews (2 VL + 2 CN)
- **Hard Violations under Q's process (H3 disabled)**: **0 violations** (Feasible: true)
- **Hard Violations with H3 enabled**: 7 (Expected: Q's table did not record campuses; because Thầy Nghĩa is the sole CN setter and all 11 VL teachers review CN, CN multi-campus requires VL teachers on Campus 1, which conflicts with VL internal multi-campus).

## 3. Q's Table Soft Score vs Optimizer Benchmark
- **Q's Table Total Soft Penalty (No H3)**: 172.80
- **Optimizer Best Total Soft Penalty (No H3)**: 146.80
- **Optimizer Best Total Soft Penalty (With H3)**: 146.80
- **Optimizer Runs / Iterations**: 8 runs, 400000 total iterations
- **Optimizer Runtime**: 813 ms
