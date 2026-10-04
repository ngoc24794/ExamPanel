   Compiling exam-panel-service v0.1.0 (C:\Users\ngocnv.HUONGVIETGROUP\source\repos\ExamPanel\crates\service)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.99s
     Running `target\debug\task5_comparison.exe`
Running Cold Start Optimizer (seed 42, R = 8, 200,000 iters)...
Running Warm Start Optimizer (seed 42, R = 8, 200,000 iters from Q's plan)...

# Task 5 Comparison: Q Manual Plan vs Optimizer (nocampus)

## Warm vs Cold Optimization Summary
- Base Seed: 42, Runs: 8 (seeds 42..49), Iterations: 200,000 per run
- Initial Score of Warm Run (Q's manual plan on nocampus with C Quí's override):
  * With S3: 260.36
  * Without S3: 116.36
- Final Best Score (Cold Start): With S3 = 176.36, Without S3 = 32.36
- Final Best Score (Warm Start): With S3 = 176.36, Without S3 = 32.36
- Evidence of Identicality: Cold and Warm optimization runs produce IDENTICAL unit scores across all 10 rules.

| Rule | Weight | LB Units | Q Units | Q Pen | Q Gap | Cold Units | Cold Pen | Cold Gap | Warm Units | Warm Pen | Warm Gap |
| :--- | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: |
| S1 Reviewer frequency        | 10.0 |   0.00 |    1.00 |  10.00 |  1.00 |       0.00 |     0.00 |     0.00 |       0.00 |     0.00 |     0.00 |
| S2 Role ratio balance        |  3.0 |   0.00 |    1.00 |   3.00 |  1.00 |       0.00 |     0.00 |     0.00 |       0.00 |     0.00 |     0.00 |
| S3 Campus independence       |  4.0 |   0.00 |   36.00 | 144.00 | 36.00 |      36.00 |   144.00 |    36.00 |      36.00 |   144.00 |    36.00 |
| S4 Setter pair diversity     |  6.0 |   0.00 |    0.00 |   0.00 |  0.00 |       0.00 |     0.00 |     0.00 |       0.00 |     0.00 |     0.00 |
| S5 Review relation diversity |  6.0 |   1.00 |    1.00 |   6.00 |  0.00 |       1.00 |     6.00 |     0.00 |       1.00 |     6.00 |     0.00 |
| S6 Setter consecutive        |  2.0 |   2.00 |    8.00 |  16.00 |  6.00 |       3.00 |     6.00 |     1.00 |       3.00 |     6.00 |     1.00 |
| S7 Grade rotation            |  1.0 |   0.00 |    0.00 |   0.00 |  0.00 |       0.00 |     0.00 |     0.00 |       0.00 |     0.00 |     0.00 |
| S8 Workload quota balance    |  8.0 |   2.55 |    4.55 |  36.36 |  2.00 |       2.55 |    20.36 |     0.00 |       2.55 |    20.36 |     0.00 |
| S9 Avoidable exam crowding   |  5.0 |   0.00 |    9.00 |  45.00 |  9.00 |       0.00 |     0.00 |     0.00 |       0.00 |     0.00 |     0.00 |
| S10 Review subject missing   |  4.0 |   0.00 |    0.00 |   0.00 |  0.00 |       0.00 |     0.00 |     0.00 |       0.00 |     0.00 |     0.00 |
| **Total (with S3)** | | | | **260.36** | | | **176.36** | | | **176.36** |
| **Total (without S3)** | | | | **116.36** | | | **32.36** | | | **32.36** |

## Lower Bound Reasoning
- **S6 Lower Bound = 2.0**: Across the 4 exams, 24 non-forced setter presences are distributed over 11 eligible teachers (k_t <= 4), with Teacher 12 excluded as his 12 setter seats are strictly forced. Minimizing sum f(k_t) where f(k) = max(0, 2k - 5) allocates 9 teachers 2 presences (f=0) and 2 teachers 3 presences (f=1 each), establishing an exact lower bound of 2.0 units.
- **S10 Lower Bound = 0.0**: 11 teachers (Teachers 1..11) possess competencies to review both Vật lí (VL) and Công nghệ (CN). Across the schedule there are 12 VL review seats and 12 CN review seats. Since 11 <= 12 in both subjects, it is mathematically possible for every teacher to review each subject at least once.
- **S5 Lower Bound >= 1.0**: In subject CN, all 12 panels require exactly 1 reviewer, and Teacher 12 (Thầy Nghĩa) is the unique forced setter. Only Teachers 1..11 can review CN. By the Pigeonhole Principle, assigning 12 seats among 11 teachers requires at least one teacher to review CN at least ceil(12/11) = 2 times. Each repeated review of Thầy Nghĩa incurs max(0, count - 1) >= 1 unit of penalty. Hence S5 >= 1.0.
- **S3 Single-Campus Explanation**: On nocampus, all teachers reside at Campus 1. For each of the 12 Vật lí (VL) panels (2 setters, 1 reviewer), the reviewer shares campus with 2 setters, contributing 12 x 2 = 24 units. For each of the 12 Công nghệ (CN) panels (1 setter, 1 reviewer), the reviewer shares campus with 1 setter, contributing 12 x 1 = 12 units. Total single-campus S3 units = 24 + 12 = 36.0 units.
