    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.16s
     Running `target\debug\examples\report.exe`
# Reproducible Phase 5 & 6 Verification Report

> Generated automatically by `crates/service/examples/report.rs` on 2026-10-02

## 1. Rule Weights: Database Defaults vs SPEC Defaults

| Rule | Name | DB Weight | SPEC Weight | Status |
|------|------|-----------|-------------|--------|
| H4 | H4: Single Panel Per Exam | 100.0 | 100.0 | MATCH |
| H7 | H7: Workload Quota Compliance | 100.0 | 100.0 | MATCH |
| S1 | S1: Reviewer Count / Frequency | 10.0 | 10.0 | MATCH |
| S2 | S2: Role Balance (1/3 Reviewer) | 3.0 | 3.0 | MATCH |
| S3 | S3: Independent Reviewer Campus | 4.0 | 4.0 | MATCH |
| S4 | S4: Repeated Setter Pair Diversity | 6.0 | 6.0 | MATCH |
| S5 | S5: Repeated Review Relation Diversity | 6.0 | 6.0 | MATCH |
| S6 | S6: Consecutive Setter Exam Relief | 2.0 | 2.0 | MATCH |
| S7 | S7: Multi-Grade Rotation | 1.0 | 1.0 | MATCH |
| S8 | S8: Discrete Load Balance | 8.0 | 8.0 | MATCH |

**All rule weights in DB match SPEC defaults perfectly.**

## 2. Per-Rule Score & Lower Bound Comparison

| Rule | Weight | LB Units | Hard Units | Hard Pen | Plan 1 Units | Plan 1 Pen | Plan 2 Units | Plan 2 Pen | Plan 3 Units | Plan 3 Pen |
|------|--------|----------|------------|----------|--------------|------------|--------------|------------|--------------|------------|
| S1 | 10.0 | 0.00 | 4.00 | 40.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 |
| S2 | 3.0 | 0.00 | 4.00 | 12.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 |
| S3 | 4.0 | 0.00 | 1.00 | 4.00 | 0.00 | 0.00 | 0.00 | 0.00 | 1.00 | 4.00 |
| S4 | 6.0 | 0.00 | 2.00 | 12.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 |
| S5 | 6.0 | 0.00 | 3.00 | 18.00 | 1.00 | 6.00 | 1.00 | 6.00 | 0.00 | 0.00 |
| S6 | 2.0 | 2.00 | 8.00 | 16.00 | 6.00 | 12.00 | 6.00 | 12.00 | 7.00 | 14.00 |
| S7 | 1.0 | 0.00 | 1.00 | 1.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 |
| S8 | 8.0 | 2.60 | 2.60 | 20.80 | 2.60 | 20.80 | 2.60 | 20.80 | 2.60 | 20.80 |
| **TOTAL** | — | — | — | **123.80** | — | **38.80** | — | **38.80** | — | **38.80** |

*Theoretical Lower Bound Penalty Sum: 24.80*

## 3. Per-Teacher Workload & Quota Allocation (Plan 1)

| Teacher | Campus | Load | Available | Quota (q) | [lo, hi] | S8-Opt | Actual | Setter | Reviewer | Assigned Grades |
|---|---|---|---|---|---|---|---|---|---|---|
| Nguyễn Văn An | Phân hiệu 1 - Ba Đình (CS1) | 1.00 | 4/4 | 3.47 | [2, 4] | 4 | 4 | 3 | 1 | Khối 10, Khối 11 |
| Trần Thị Bình | Phân hiệu 1 - Ba Đình (CS1) | 1.00 | 4/4 | 3.47 | [2, 4] | 4 | 3 | 2 | 1 | Khối 11 |
| Lê Hoàng Cường | Phân hiệu 2 - Cầu Giấy (CS2) | 1.00 | 4/4 | 3.47 | [2, 4] | 4 | 4 | 3 | 1 | Khối 10, Khối 12 |
| Phạm Minh Đức | Phân hiệu 2 - Cầu Giấy (CS2) | 1.00 | 4/4 | 3.47 | [2, 4] | 4 | 3 | 2 | 1 | Khối 11 |
| Hoàng Thu Giang | Phân hiệu 3 - Hà Đông (CS3) | 1.00 | 4/4 | 3.47 | [2, 4] | 4 | 3 | 2 | 1 | Khối 10 |
| Vũ Hải Hà | Phân hiệu 3 - Hà Đông (CS3) | 0.50 | 3/4 | 1.30 | [0, 3] | 1 | 1 | 0 | 1 | Khối 11 |
| Đặng Quốc Hùng | Phân hiệu 3 - Hà Đông (CS3) | 1.00 | 4/4 | 3.47 | [2, 4] | 3 | 3 | 2 | 1 | Khối 12 |
| Bùi Thị Lan | Phân hiệu 4 - Hoàn Kiếm (CS4) | 1.00 | 4/4 | 3.47 | [2, 4] | 3 | 3 | 2 | 1 | Khối 10 |
| Đỗ Tuấn Minh | Phân hiệu 4 - Hoàn Kiếm (CS4) | 1.00 | 4/4 | 3.47 | [2, 4] | 3 | 4 | 3 | 1 | Khối 11 |
| Ngô Phương Nam | Phân hiệu 1 - Ba Đình (CS1) | 1.00 | 4/4 | 3.47 | [2, 4] | 3 | 4 | 2 | 2 | Khối 12 |
| Dương Thùy Trang | Phân hiệu 2 - Cầu Giấy (CS2) | 1.00 | 4/4 | 3.47 | [2, 4] | 3 | 4 | 3 | 1 | Khối 12 |

## 4. Assignment Matrix: 4 Exams × 3 Grades (Plan 1)

| Exam | Khối 10 | Khối 11 | Khối 12 |
|---|---|---|---|
| **Giữa kỳ 1** (GK1) | **Ra đề:** Hoàng Thu Giang (CS3), Lê Hoàng Cường (CS2)<br/>**Phản biện:** Bùi Thị Lan (CS4) | **Ra đề:** Nguyễn Văn An (CS1), Đỗ Tuấn Minh (CS4)<br/>**Phản biện:** Vũ Hải Hà (CS3) | **Ra đề:** Ngô Phương Nam (CS1), Dương Thùy Trang (CS2)<br/>**Phản biện:** Đặng Quốc Hùng (CS3) |
| **Cuối kỳ 1** (CK1) | **Ra đề:** Lê Hoàng Cường (CS2), Bùi Thị Lan (CS4)<br/>**Phản biện:** Nguyễn Văn An (CS1) | **Ra đề:** Phạm Minh Đức (CS2), Trần Thị Bình (CS1)<br/>**Phản biện:** Đỗ Tuấn Minh (CS4) | **Ra đề:** Dương Thùy Trang (CS2), Đặng Quốc Hùng (CS3)<br/>**Phản biện:** Ngô Phương Nam (CS1) |
| **Giữa kỳ 2** (GK2) | **Ra đề:** Hoàng Thu Giang (CS3), Nguyễn Văn An (CS1)<br/>**Phản biện:** Lê Hoàng Cường (CS2) | **Ra đề:** Đỗ Tuấn Minh (CS4), Phạm Minh Đức (CS2)<br/>**Phản biện:** Trần Thị Bình (CS1) | **Ra đề:** Ngô Phương Nam (CS1), Đặng Quốc Hùng (CS3)<br/>**Phản biện:** Dương Thùy Trang (CS2) |
| **Cuối kỳ 2** (CK2) | **Ra đề:** Nguyễn Văn An (CS1), Bùi Thị Lan (CS4)<br/>**Phản biện:** Hoàng Thu Giang (CS3) | **Ra đề:** Trần Thị Bình (CS1), Đỗ Tuấn Minh (CS4)<br/>**Phản biện:** Phạm Minh Đức (CS2) | **Ra đề:** Dương Thùy Trang (CS2), Lê Hoàng Cường (CS2)<br/>**Phản biện:** Ngô Phương Nam (CS1) |

## 5. Constraint Validator Output

- **Stage 1 (Solve Hard):** 0 hard violations (valid = true)
- **Plan 1 (Rank 1):** 0 hard violations (valid = true)
- **Plan 2 (Rank 2):** 0 hard violations (valid = true)
- **Plan 3 (Rank 3):** 0 hard violations (valid = true)

## 6. Pairwise Plan Distances

| Pair | Distance | Similarity | Interpretation |
|---|---|---|---|
| Plan 1 vs Plan 2 | 0.611 | 38.9% | Exceeds 20% threshold (PASS) |
| Plan 1 vs Plan 3 | 0.556 | 44.4% | Exceeds 20% threshold (PASS) |
| Plan 2 vs Plan 3 | 0.667 | 33.3% | Exceeds 20% threshold (PASS) |
| Solve Hard vs Plan 1 | 0.472 | 52.8% | Divergence after 200k anneal moves |

