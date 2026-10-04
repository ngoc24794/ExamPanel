
# Task 5 Comparison: Q Manual Plan vs Optimizer (nocampus)

| Rule | Weight | LB Units | Q Units | Q Pen | Q Gap | Cold Units | Cold Pen | Cold Gap | Warm Units | Warm Pen | Warm Gap |
| :--- | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: |
| S1 Reviewer frequency        | 10.0 |   0.00 |    1.00 |  10.00 |  1.00 |       0.00 |     0.00 |     0.00 |       0.00 |     0.00 |     0.00 |
| S2 Role ratio balance        |  3.0 |   0.00 |    1.00 |   3.00 |  1.00 |       0.00 |     0.00 |     0.00 |       0.00 |     0.00 |     0.00 |
| S3 Campus independence       |  4.0 |   0.00 |   36.00 | 144.00 | 36.00 |      36.00 |   144.00 |    36.00 |      36.00 |   144.00 |    36.00 |
| S4 Setter pair diversity     |  6.0 |   0.00 |    0.00 |   0.00 |  0.00 |       0.00 |     0.00 |     0.00 |       0.00 |     0.00 |     0.00 |
| S5 Review relation diversity |  6.0 |   1.00 |    1.00 |   6.00 |  0.00 |       1.00 |     6.00 |     0.00 |       1.00 |     6.00 |     0.00 |
| S6 Setter consecutive        |  2.0 |   0.00 |    8.00 |  16.00 |  8.00 |       3.00 |     6.00 |     3.00 |       3.00 |     6.00 |     3.00 |
| S7 Grade rotation            |  1.0 |   0.00 |    0.00 |   0.00 |  0.00 |       0.00 |     0.00 |     0.00 |       0.00 |     0.00 |     0.00 |
| S8 Workload quota balance    |  8.0 |   2.55 |    4.55 |  36.36 |  2.00 |       2.55 |    20.36 |     0.00 |       2.55 |    20.36 |     0.00 |
| S9 Avoidable exam crowding   |  5.0 |   0.00 |    9.00 |  45.00 |  9.00 |       0.00 |     0.00 |     0.00 |       0.00 |     0.00 |     0.00 |
| S10 Review subject missing   |  4.0 |   0.00 |    0.00 |   0.00 |  0.00 |       0.00 |     0.00 |     0.00 |       0.00 |     0.00 |     0.00 |
| **Total (with S3)** | | | | **260.36** | | | **176.36** | | | **176.36** | |
| **Total (without S3)** | | | | **116.36** | | | **32.36** | | | **32.36** | |

## Lower Bound Reasoning
- **S10 Lower Bound = 0.0**: 11 teachers (Teachers 1..11) possess competencies to review both Vật lí (VL) and Công nghệ (CN). Across the schedule there are 12 VL review seats and 12 CN review seats. Since 11 <= 12 in both subjects, it is mathematically possible for every teacher to review each subject at least once.
- **S5 Lower Bound >= 1.0**: In subject CN, all 12 panels require exactly 1 reviewer, and Teacher 12 (Thầy Nghĩa) is the unique forced setter. Only Teachers 1..11 can review CN. By the Pigeonhole Principle, assigning 12 seats among 11 teachers requires at least one teacher to review CN at least ceil(12/11) = 2 times. Each repeated review of Thầy Nghĩa incurs max(0, count - 1) >= 1 unit of penalty. Hence S5 >= 1.0.
