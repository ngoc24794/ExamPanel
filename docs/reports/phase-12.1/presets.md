# Đánh giá 4 bộ cấu hình mẫu (Rule Presets Analysis)

Kiểm định trên tập dữ liệu canonical Q (`QVariant::NoCampus`):
- Thông số: Cold start, base seed = 42 (seeds 42..49), R = 8, Budget = 200,000 bước lặp / luồng, K = 3.
- Cận dưới: S5 LB = 1.00 (Dirichlet), S6 LB = 2.00 (Parity), S8 LB = 2.5455 (28/11), S3 = 0.00 (độc lập) / 36.00 (đơn cơ sở).

## Preset: Cân bằng (mặc định) (`balanced`)
| Quy tắc | Trạng thái | Trọng số | Cận dưới (LB) | Đơn vị vi phạm | Điểm phạt | Chênh lệch (Gap) |
| :--- | :---: | :---: | :---: | :---: | :---: | :---: |
| S1 Reviewer frequency        |    BẬT     |     10.0 |          0.00 |           0.00 |      0.00 |             0.00 |
| S2 Role ratio balance        |    BẬT     |      3.0 |          0.00 |           0.00 |      0.00 |             0.00 |
| S3 Campus independence       |    BẬT     |      4.0 |          0.00 |          36.00 |    144.00 |            36.00 |
| S4 Setter pair diversity     |    BẬT     |      6.0 |          0.00 |           0.00 |      0.00 |             0.00 |
| S5 Review relation diversity |    BẬT     |      6.0 |          1.00 |           1.00 |      6.00 |             0.00 |
| S6 Setter consecutive        |    BẬT     |      2.0 |          2.00 |           3.00 |      6.00 |             1.00 |
| S7 Grade rotation            |    BẬT     |      1.0 |          0.00 |           0.00 |      0.00 |             0.00 |
| S8 Workload quota balance    |    BẬT     |      8.0 |          2.55 |           2.55 |     20.36 |             0.00 |
| S9 Avoidable exam crowding   |    BẬT     |      5.0 |          0.00 |           0.00 |      0.00 |             0.00 |
| S10 Review subject missing   |    BẬT     |      4.0 |          0.00 |           0.00 |      0.00 |             0.00 |
| **Tổng cộng (Không tính S3)** | | | **30.36** | | **32.36** | **2.00** |
| **Tổng cộng (Có S3)** | | | **174.36** | | **176.36** | **2.00** |

## Preset: Ưu tiên công bằng khối lượng (`workload_fairness`)
| Quy tắc | Trạng thái | Trọng số | Cận dưới (LB) | Đơn vị vi phạm | Điểm phạt | Chênh lệch (Gap) |
| :--- | :---: | :---: | :---: | :---: | :---: | :---: |
| S1 Reviewer frequency        |    BẬT     |     12.0 |          0.00 |           0.00 |      0.00 |             0.00 |
| S2 Role ratio balance        |    BẬT     |      3.0 |          0.00 |           0.00 |      0.00 |             0.00 |
| S3 Campus independence       |    BẬT     |      4.0 |          0.00 |          36.00 |    144.00 |            36.00 |
| S4 Setter pair diversity     |    BẬT     |      6.0 |          0.00 |           0.00 |      0.00 |             0.00 |
| S5 Review relation diversity |    BẬT     |      6.0 |          1.00 |           1.00 |      6.00 |             0.00 |
| S6 Setter consecutive        |    BẬT     |      2.0 |          2.00 |           3.00 |      6.00 |             1.00 |
| S7 Grade rotation            |    BẬT     |      1.0 |          0.00 |           0.00 |      0.00 |             0.00 |
| S8 Workload quota balance    |    BẬT     |     16.0 |          2.55 |           2.55 |     40.73 |             0.00 |
| S9 Avoidable exam crowding   |    BẬT     |      8.0 |          0.00 |           0.00 |      0.00 |             0.00 |
| S10 Review subject missing   |    BẬT     |      4.0 |          0.00 |           0.00 |      0.00 |             0.00 |
| **Tổng cộng (Không tính S3)** | | | **50.73** | | **52.73** | **2.00** |
| **Tổng cộng (Có S3)** | | | **194.73** | | **196.73** | **2.00** |

## Preset: Ưu tiên đa dạng ê-kíp (`team_diversity`)
| Quy tắc | Trạng thái | Trọng số | Cận dưới (LB) | Đơn vị vi phạm | Điểm phạt | Chênh lệch (Gap) |
| :--- | :---: | :---: | :---: | :---: | :---: | :---: |
| S1 Reviewer frequency        |    BẬT     |     10.0 |          0.00 |           0.00 |      0.00 |             0.00 |
| S2 Role ratio balance        |    BẬT     |      3.0 |          0.00 |           0.00 |      0.00 |             0.00 |
| S3 Campus independence       |    BẬT     |      8.0 |          0.00 |          36.00 |    288.00 |            36.00 |
| S4 Setter pair diversity     |    BẬT     |     12.0 |          0.00 |           0.00 |      0.00 |             0.00 |
| S5 Review relation diversity |    BẬT     |     12.0 |          1.00 |           1.00 |     12.00 |             0.00 |
| S6 Setter consecutive        |    BẬT     |      2.0 |          2.00 |           3.00 |      6.00 |             1.00 |
| S7 Grade rotation            |    BẬT     |      1.0 |          0.00 |           0.00 |      0.00 |             0.00 |
| S8 Workload quota balance    |    BẬT     |      6.0 |          2.55 |           2.55 |     15.27 |             0.00 |
| S9 Avoidable exam crowding   |    BẬT     |      5.0 |          0.00 |           0.00 |      0.00 |             0.00 |
| S10 Review subject missing   |    BẬT     |      6.0 |          0.00 |           0.00 |      0.00 |             0.00 |
| **Tổng cộng (Không tính S3)** | | | **31.27** | | **33.27** | **2.00** |
| **Tổng cộng (Có S3)** | | | **319.27** | | **321.27** | **2.00** |

## Preset: Cho phép dồn việc trong một kỳ (`allow_task_crowding`)
| Quy tắc | Trạng thái | Trọng số | Cận dưới (LB) | Đơn vị vi phạm | Điểm phạt | Chênh lệch (Gap) |
| :--- | :---: | :---: | :---: | :---: | :---: | :---: |
| S1 Reviewer frequency        |    BẬT     |     10.0 |          0.00 |           0.00 |      0.00 |             0.00 |
| S2 Role ratio balance        |    BẬT     |      3.0 |          0.00 |           0.00 |      0.00 |             0.00 |
| S3 Campus independence       |    BẬT     |      4.0 |          0.00 |          36.00 |    144.00 |            36.00 |
| S4 Setter pair diversity     |    BẬT     |      6.0 |          0.00 |           0.00 |      0.00 |             0.00 |
| S5 Review relation diversity |    BẬT     |      6.0 |          1.00 |           1.00 |      6.00 |             0.00 |
| S6 Setter consecutive        |    BẬT     |      2.0 |          2.00 |           3.00 |      6.00 |             1.00 |
| S7 Grade rotation            |    BẬT     |      1.0 |          0.00 |           0.00 |      0.00 |             0.00 |
| S8 Workload quota balance    |    BẬT     |      8.0 |          2.55 |           2.55 |     20.36 |             0.00 |
| S9 Avoidable exam crowding   |    TẮT     |      0.0 |          0.00 |           6.00 |      0.00 |             6.00 |
| S10 Review subject missing   |    BẬT     |      4.0 |          0.00 |           0.00 |      0.00 |             0.00 |
| **Tổng cộng (Không tính S3)** | | | **30.36** | | **32.36** | **2.00** |
| **Tổng cộng (Có S3)** | | | **174.36** | | **176.36** | **2.00** |

