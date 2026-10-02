    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.16s
     Running unittests src\lib.rs (target\debug\deps\exam_panel_core-2f9eaf9f6c8d622c.exe)

running 1 test

=== TABLE 1: BEFORE / AFTER SCORE PER RULE ===
| Rule | Name | Weight | Hard Solve Units | Hard Penalty | Opt Units | Opt Penalty | Lower Bound |
|---|---|---|---|---|---|---|---|
| S1 | Reviewer count | 10.0 | 5.00 | 50.00 | 0.00 | 0.00 | 0.00 |
| S2 | Role balance | 3.0 | 5.00 | 15.00 | 0.00 | 0.00 | 0.00 |
| S3 | Independent reviewer | 4.0 | 2.00 | 8.00 | 0.00 | 0.00 | 0.00 |
| S4 | Repeated setter pair | 6.0 | 2.00 | 12.00 | 0.00 | 0.00 | 0.00 |
| S5 | Repeated review relation | 6.0 | 5.00 | 30.00 | 0.00 | 0.00 | 0.00 |
| S6 | Consecutive setting | 2.0 | 11.00 | 22.00 | 6.00 | 12.00 | 2.00 |
| S7 | Grade rotation | 1.0 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 |
| S8 | Load deviation | 8.0 | 2.60 | 20.80 | 2.60 | 20.80 | 2.60 |
| **TOTAL** | | | | **157.80** | | **32.80** |

=== TABLE 2: 4x3 ASSIGNMENT SCHEDULE (EXAM x GRADE) ===
| Exam | Grade | Setters (Campus) | Reviewer (Campus) | Campuses Distinct |
|---|---|---|---|---|
| GK1 | Khoi 10 | Le Hoang Cuong (CS2), Hoang Thu Giang (CS3) | Bui Thi Lan (CS4) | Yes |
| GK1 | Khoi 11 | Pham Minh Duc (CS2), Nguyen Van An (CS1) | Do Tuan Minh (CS4) | Yes |
| GK1 | Khoi 12 | Ngo Phuong Nam (CS1), Tran Thi Binh (CS1) | Dang Quoc Hung (CS3) | Yes |
| CK1 | Khoi 10 | Le Hoang Cuong (CS2), Bui Thi Lan (CS4) | Hoang Thu Giang (CS3) | Yes |
| CK1 | Khoi 11 | Do Tuan Minh (CS4), Pham Minh Duc (CS2) | Tran Thi Binh (CS1) | Yes |
| CK1 | Khoi 12 | Duong Thuy Trang (CS2), Dang Quoc Hung (CS3) | Ngo Phuong Nam (CS1) | Yes |
| GK2 | Khoi 10 | Nguyen Van An (CS1), Hoang Thu Giang (CS3) | Le Hoang Cuong (CS2) | Yes |
| GK2 | Khoi 11 | Do Tuan Minh (CS4), Tran Thi Binh (CS1) | Pham Minh Duc (CS2) | Yes |
| GK2 | Khoi 12 | Ngo Phuong Nam (CS1), Dang Quoc Hung (CS3) | Duong Thuy Trang (CS2) | Yes |
| CK2 | Khoi 10 | Bui Thi Lan (CS4), Hoang Thu Giang (CS3) | Nguyen Van An (CS1) | Yes |
| CK2 | Khoi 11 | Tran Thi Binh (CS1), Pham Minh Duc (CS2) | Vu Hai Ha (CS3) | Yes |
| CK2 | Khoi 12 | Duong Thuy Trang (CS2), Le Hoang Cuong (CS2) | Dang Quoc Hung (CS3) | Yes |

=== TABLE 3: PER-TEACHER WORKLOAD TABLE ===
| ID | Teacher Name | Quota q_t | Total Tasks | Setter Tasks | Reviewer Tasks | Grades Assigned |
|---|---|---|---|---|---|---|
| 1 | Nguyen Van An | 3.47 | 3 | 2 | 1 | 10, 11 |
| 2 | Tran Thi Binh | 3.47 | 4 | 3 | 1 | 11, 12 |
| 3 | Le Hoang Cuong | 3.47 | 4 | 3 | 1 | 10, 12 |
| 4 | Pham Minh Duc | 3.47 | 4 | 3 | 1 | 11 |
| 5 | Hoang Thu Giang | 3.47 | 4 | 3 | 1 | 10 |
| 6 | Vu Hai Ha | 1.30 | 1 | 0 | 1 | 11 |
| 7 | Dang Quoc Hung | 3.47 | 4 | 2 | 2 | 12 |
| 8 | Bui Thi Lan | 3.47 | 3 | 2 | 1 | 10 |
| 9 | Do Tuan Minh | 3.47 | 3 | 2 | 1 | 11 |
| 10 | Ngo Phuong Nam | 3.47 | 3 | 2 | 1 | 12 |
| 11 | Duong Thuy Trang | 3.47 | 3 | 2 | 1 | 12 |

=== TABLE 4: REMAINING SOFT VIOLATIONS ===
- Rule S6 (code: `setter_consecutive`): panel None, teachers [TeacherId(2)], params: {"exam1": "GK2", "exam2": "CK2", "teacher": "Tran Thi Binh"}
- Rule S6 (code: `setter_consecutive`): panel None, teachers [TeacherId(3)], params: {"exam1": "GK1", "exam2": "CK1", "teacher": "Le Hoang Cuong"}
- Rule S6 (code: `setter_consecutive`): panel None, teachers [TeacherId(4)], params: {"exam1": "GK1", "exam2": "CK1", "teacher": "Pham Minh Duc"}
- Rule S6 (code: `setter_consecutive`): panel None, teachers [TeacherId(5)], params: {"exam1": "GK2", "exam2": "CK2", "teacher": "Hoang Thu Giang"}
- Rule S6 (code: `setter_consecutive`): panel None, teachers [TeacherId(7)], params: {"exam1": "CK1", "exam2": "GK2", "teacher": "Dang Quoc Hung"}
- Rule S6 (code: `setter_consecutive`): panel None, teachers [TeacherId(9)], params: {"exam1": "CK1", "exam2": "GK2", "teacher": "Do Tuan Minh"}
- Rule S8 (code: `load_deviation`): panel None, teachers [TeacherId(1)], params: {"count": "3", "deviation": "-0.47", "quota": "3.47", "teacher": "Nguyen Van An"}
- Rule S8 (code: `load_deviation`): panel None, teachers [TeacherId(2)], params: {"count": "4", "deviation": "+0.53", "quota": "3.47", "teacher": "Tran Thi Binh"}
- Rule S8 (code: `load_deviation`): panel None, teachers [TeacherId(3)], params: {"count": "4", "deviation": "+0.53", "quota": "3.47", "teacher": "Le Hoang Cuong"}
- Rule S8 (code: `load_deviation`): panel None, teachers [TeacherId(4)], params: {"count": "4", "deviation": "+0.53", "quota": "3.47", "teacher": "Pham Minh Duc"}
- Rule S8 (code: `load_deviation`): panel None, teachers [TeacherId(5)], params: {"count": "4", "deviation": "+0.53", "quota": "3.47", "teacher": "Hoang Thu Giang"}
- Rule S8 (code: `load_deviation`): panel None, teachers [TeacherId(6)], params: {"count": "1", "deviation": "-0.30", "quota": "1.30", "teacher": "Vu Hai Ha"}
- Rule S8 (code: `load_deviation`): panel None, teachers [TeacherId(7)], params: {"count": "4", "deviation": "+0.53", "quota": "3.47", "teacher": "Dang Quoc Hung"}
- Rule S8 (code: `load_deviation`): panel None, teachers [TeacherId(8)], params: {"count": "3", "deviation": "-0.47", "quota": "3.47", "teacher": "Bui Thi Lan"}
- Rule S8 (code: `load_deviation`): panel None, teachers [TeacherId(9)], params: {"count": "3", "deviation": "-0.47", "quota": "3.47", "teacher": "Do Tuan Minh"}
- Rule S8 (code: `load_deviation`): panel None, teachers [TeacherId(10)], params: {"count": "3", "deviation": "-0.47", "quota": "3.47", "teacher": "Ngo Phuong Nam"}
- Rule S8 (code: `load_deviation`): panel None, teachers [TeacherId(11)], params: {"count": "3", "deviation": "-0.47", "quota": "3.47", "teacher": "Duong Thuy Trang"}

=== TABLE 5: PAIRWISE DISTANCES BETWEEN RETURNED PLANS ===
Number of plans returned: 3
Distance between Plan 1 (score 32.80) and Plan 2 (score 32.80): 0.722 (72.2%)
Distance between Plan 1 (score 32.80) and Plan 3 (score 35.50): 0.583 (58.3%)
Distance between Plan 2 (score 32.80) and Plan 3 (score 35.50): 0.611 (61.1%)
test optimize::tests::print_report_demo_data ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 72 filtered out; finished in 2.05s

     Running unittests src\lib.rs (target\debug\deps\exam_panel_service-91405ffa6edb88dc.exe)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running unittests src\lib.rs (target\debug\deps\exam_panel_storage-126170342425c528.exe)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 15 filtered out; finished in 0.00s

