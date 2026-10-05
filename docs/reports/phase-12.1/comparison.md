# Báo cáo so sánh và kiểm định thực tế (Phase 12 - Part F / Phase 12.1)

## 1. Thông số cấu hình & Nguồn dữ liệu kiểm định
- **Fixture variant:** `QVariant::NoCampus` (chính) & `QVariant::SyntheticCampuses` (kiểm tra phân hiệu)
- **Tham số quy tắc cứng H3 / H4 / H7:**
  * **H3 (Độc lập cơ sở ban đề):** min_campuses = 1 (trên NoCampus); min_campuses = 2 (trên SyntheticCampuses, áp dụng cho cả VL và CN).
  * **H4 (Giới hạn nhiệm vụ/kỳ):** Mặc định max_tasks_per_exam = 2. Cấu hình override: C Quí = 3 (cho phép 3 việc trong kỳ), T Nghĩa = 3 (do bắt buộc 3 ban CN mỗi kỳ).
  * **H7 (Giới hạn ra đề liên tiếp):** max_consecutive_setter = 2.
- **Trạng thái kích hoạt và trọng số của toàn bộ quy tắc:**
  * Quy tắc cứng: H1 (Chuyên môn) = ON, H2 (Khối lớp) = ON, H3 (Cơ sở) = ON (min_campuses tùy cấu hình), H4 (Giới hạn/kỳ) = ON, H5 (Trùng giờ) = ON, H6 (Bận) = ON, H7 (Liên tiếp) = ON.
  * Quy tắc mềm: S1 = 10.0, S2 = 3.0, S3 = 4.0, S4 = 6.0, S5 = 6.0, S6 = 2.0, S7 = 1.0, S8 = 8.0, S9 = 5.0, S10 = 4.0 (Tất cả đều ON).
- **Thông số tối ưu hoá:** Hạt giống base seed = 42 (dải seeds 42..49), Số luồng R = 8, Số bước lặp = 200,000 / luồng (tổng 1,600,000 bước lặp), Số phương án giữ lại K = 3.
- **Cận dưới lý thuyết Σ LB:**
  * S5 LB = 1.00 đơn vị (phạt: 6.00) [Nguyên lý Dirichlet trên CN: 12 ban đề / 11 GV phản biện]
  * S6 LB = 2.00 đơn vị (phạt: 4.00) [Cân bằng lượt ra đề 24 lượt VL / 11 GV: 9 người 2 lượt, 2 người 3 lượt => 2 x (2*3-5) = 2.0]
  * S8 LB = 2.5455 đơn vị (phạt: 20.36) [48 lượt / 11 GV, quota = 48/11: 4 người 5 việc, 7 người 4 việc => Σ(c-q)² = 28/11 ≈ 2.5455]
  * Các quy tắc mềm khác (S1, S2, S4, S7, S9, S10): Cận dưới = 0.00 đơn vị (phạt: 0.00)
  * **Tổng cận dưới Σ LB (Không tính S3):** **30.36** (S5: 6.00 + S6: 4.00 + S8: 20.36)
  * **Tổng cận dưới Σ LB (Tính S3 = 0.00 theo cận dưới độc lập):** **30.36**
  * **Tổng cận dưới Σ LB (Tính S3 đơn cơ sở = 36 đơn vị x 4.0 = 144.00):** **174.36**

### Bảng cấu hình giáo viên (Quota override & Max tasks override)
| Mã GV | Tên đầy đủ | Cách gọi | Cơ sở | Hệ số tải | Quota Override | Max Tasks/Exam Override | Ghi chú |
| :---: | :--- | :---: | :---: | :---: | :---: | :---: | :--- |
| HIEN  | Cô Hiền          | C Hiền   |   1   |    1.0    |      None      |          None           | Mặc định        |
| LAI   | Cô Lài           | C Lài    |   1   |    1.0    |      None      |          None           | Mặc định        |
| PHUC  | Thầy Phúc        | T Phúc   |   1   |    1.0    |      None      |          None           | Mặc định        |
| LOC   | Thầy Lộc         | T Lộc    |   1   |    1.0    |      None      |          None           | Mặc định        |
| THU   | Cô Thư           | C Thư    |   1   |    1.0    |      None      |          None           | Mặc định        |
| NA    | Cô Na            | C Na     |   1   |    1.0    |      None      |          None           | Mặc định        |
| BINH  | Cô Bình          | C Bình   |   1   |    1.0    |      None      |          None           | Mặc định        |
| QUI   | Cô Quí           | C Quí    |   1   |    1.0    |      None      |            3            | C Quí: max-tasks=3, quota=None |
| TU    | Cô Tú            | C Tú     |   1   |    1.0    |      None      |          None           | Mặc định        |
| NHU   | Cô Như           | C Như    |   1   |    1.0    |      None      |          None           | Mặc định        |
| LAN   | Cô Lan           | C Lan    |   1   |    1.0    |      None      |          None           | Mặc định        |
| NGHIA | Thầy Nghĩa       | T Nghĩa  |   1   |    1.0    |       12       |            3            | T Nghĩa: quota=12, max-tasks=3 |

## 2. Nguồn dữ liệu kiểm định
- Dữ liệu đầu vào: Người dùng chọn **'none'** (chưa cung cấp số liệu thực tế phân hiệu cá nhân).
- Thực hiện kiểm định chuẩn tắc trên bộ dữ liệu kiểm thử canonical Q fixture (`make_canonical_q_problem(QVariant::SyntheticCampuses)` và `QVariant::NoCampus`).
- Không có thông tin cá nhân thực tế nào được lưu trữ hoặc commit vào kho mã nguồn.

## 3. Kiểm định bảng phân công của Thầy Q
- Tổng số phân công: 60 chỗ (12 ban đề Vật lí x 3 = 36; 12 ban đề Công nghệ x 2 = 24).
- Điểm phạt mềm (nocampus): **260.36** (không tính S3: **116.36**).
- Điểm phạt mềm (synthetic campuses): **136.36**.
- Trạng thái thoả mãn ràng buộc cứng (Hard constraints): CÓ VI PHẠM RÀNG BUỘC CỨNG
- Danh sách ban đề không đạt H3 trên synthetic campuses (3 ban đề):
  | Kỳ thi | Khối | Môn | Phân hiệu thực tế | Yêu cầu | Ra đề (Cơ sở) | Phản biện (Cơ sở) | Lý do |
  | :---: | :---: | :---: | :---: | :---: | :--- | :--- | :--- |
  | GK1 | 10 | CN | 1 | 2 | T Nghĩa (PH1) | C Hiền (PH1) | Phản biện trùng cơ sở với người ra đề |
  | CK1 | 10 | CN | 1 | 2 | T Nghĩa (PH1) | C Tú (PH1) | Phản biện trùng cơ sở với người ra đề |
  | GK2 | 11 | CN | 1 | 2 | T Nghĩa (PH1) | C Thư (PH1) | Phản biện trùng cơ sở với người ra đề |

## 4. So sánh Tối ưu Hoá: Khởi động Lạnh (Cold) vs Khởi động Ấm (Warm)
- Tham số: Seed cơ sở = 42, Số luồng R = 8 (hạt giống 42..49), Số bước lặp = 200,000 / luồng, K = 3.
- Điểm khởi tạo của Warm Run (tính bởi cùng bộ đánh giá trên phương án của Thầy Q):
  * Bao gồm S3: **260.36**
  * Không tính S3: **116.36** (chính xác bằng điểm phương án Q: **116.36**)
- Điểm tối ưu nhất (Cold Start, winning seed 43): Có S3 = **176.36**, Không tính S3 = **32.36**
- Điểm tối ưu nhất (Warm Start, winning seed 43): Có S3 = **176.36**, Không tính S3 = **32.36**
- **Bằng chứng trùng khớp tuyệt đối:** Cả Cold Start và Warm Start đều hội tụ về điểm số và đơn vị vi phạm hoàn toàn đồng nhất trên tất cả 10 quy tắc mềm.

### Bảng so sánh chi tiết theo 10 quy tắc mềm (12 cột chuẩn)
| Quy tắc | Trọng số | Cận dưới (LB) | Q Đơn vị | Q Phạt | Q Chênh | Cold Đơn vị | Cold Phạt | Cold Chênh | Warm Đơn vị | Warm Phạt | Warm Chênh |
| :--- | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: |
| S1 Reviewer frequency        |     10.0 |          0.00 |     1.00 |  10.00 |    1.00 |        0.00 |      0.00 |       0.00 |        0.00 |      0.00 |       0.00 |
| S2 Role ratio balance        |      3.0 |          0.00 |     1.00 |   3.00 |    1.00 |        0.00 |      0.00 |       0.00 |        0.00 |      0.00 |       0.00 |
| S3 Campus independence       |      4.0 |          0.00 |    36.00 | 144.00 |   36.00 |       36.00 |    144.00 |      36.00 |       36.00 |    144.00 |      36.00 |
| S4 Setter pair diversity     |      6.0 |          0.00 |     0.00 |   0.00 |    0.00 |        0.00 |      0.00 |       0.00 |        0.00 |      0.00 |       0.00 |
| S5 Review relation diversity |      6.0 |          1.00 |     1.00 |   6.00 |    0.00 |        1.00 |      6.00 |       0.00 |        1.00 |      6.00 |       0.00 |
| S6 Setter consecutive        |      2.0 |          2.00 |     8.00 |  16.00 |    6.00 |        3.00 |      6.00 |       1.00 |        3.00 |      6.00 |       1.00 |
| S7 Grade rotation            |      1.0 |          0.00 |     0.00 |   0.00 |    0.00 |        0.00 |      0.00 |       0.00 |        0.00 |      0.00 |       0.00 |
| S8 Workload quota balance    |      8.0 |          2.55 |     4.55 |  36.36 |    2.00 |        2.55 |     20.36 |       0.00 |        2.55 |     20.36 |       0.00 |
| S9 Avoidable exam crowding   |      5.0 |          0.00 |     9.00 |  45.00 |    9.00 |        0.00 |      0.00 |       0.00 |        0.00 |      0.00 |       0.00 |
| S10 Review subject missing   |      4.0 |          0.00 |     0.00 |   0.00 |    0.00 |        0.00 |      0.00 |       0.00 |        0.00 |      0.00 |       0.00 |
| **Tổng cộng (Có S3)** | | | | **260.36** | | | **176.36** | | | **176.36** | |
| **Tổng cộng (Không S3)** | | | | **116.36** | | | **32.36** | | | **32.36** | |

## 5. Thống kê phân công theo giáo viên (Phương án tối ưu tốt nhất)
| Mã GV | Cách gọi | Phân hiệu | Chỉ tiêu q | Tổng lượt | Ra đề | Phản biện | GK1 | CK1 | GK2 | CK2 | Chênh lệch | Ghi chú |
| :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :--- |
| HIEN  | C Hiền   |    PH1    |     4      |     5     |   3   |     2     |  1  |  1  |  1  |  2  |     +1     |              |
| LAI   | C Lài    |    PH2    |     4      |     4     |   2   |     2     |  1  |  1  |  1  |  1  |     +0     |              |
| PHUC  | T Phúc   |    PH3    |     4      |     4     |   2   |     2     |  1  |  1  |  1  |  1  |     +0     |              |
| LOC   | T Lộc    |    PH4    |     4      |     4     |   2   |     2     |  1  |  1  |  1  |  1  |     +0     |              |
| THU   | C Thư    |    PH1    |     4      |     4     |   2   |     2     |  1  |  1  |  1  |  1  |     +0     |              |
| NA    | C Na     |    PH2    |     4      |     5     |   2   |     3     |  2  |  1  |  1  |  1  |     +1     |              |
| BINH  | C Bình   |    PH3    |     4      |     4     |   2   |     2     |  1  |  1  |  1  |  1  |     +0     |              |
| QUI   | C Quí    |    PH4    |     4      |     5     |   3   |     2     |  1  |  1  |  2  |  1  |     +1     | Tối đa 3 việc |
| TU    | C Tú     |    PH1    |     4      |     4     |   2   |     2     |  1  |  1  |  1  |  1  |     +0     |              |
| NHU   | C Như    |    PH2    |     4      |     4     |   2   |     2     |  1  |  1  |  1  |  1  |     +0     |              |
| LAN   | C Lan    |    PH3    |     4      |     5     |   2   |     3     |  1  |  2  |  1  |  1  |     +1     |              |
| NGHIA | T Nghĩa  |    PH1    |     12     |    12     |  12   |     0     |  3  |  3  |  3  |  3  |     +0     | Cố định CN   |

## 6. Bảng phân công phương án tối ưu (Dạng bảng tổ - Q-Style Grid)
| Kì thi | Vai trò | Khối 10 (VL) | Khối 10 (CN) | Khối 11 (VL) | Khối 11 (CN) | Khối 12 (VL) | Khối 12 (CN) |
| :---: | :---: | :---: | :---: | :---: | :---: | :---: | :---: |
| **GK1** | Đề 1 | C Như | T Nghĩa | C Hiền | T Nghĩa | C Lan | T Nghĩa |
| | Đề 2 | C Na | | C Bình | | C Quí | |
| | P.Biện | C Tú | T Phúc | C Thư | C Na | C Lài | T Lộc |
| **CK1** | Đề 1 | C Hiền | T Nghĩa | C Thư | T Nghĩa | T Phúc | T Nghĩa |
| | Đề 2 | C Tú | | T Lộc | | C Lài | |
| | P.Biện | C Na | C Bình | C Quí | C Như | C Lan | C Lan |
| **GK2** | Đề 1 | C Lài | T Nghĩa | C Quí | T Nghĩa | C Bình | T Nghĩa |
| | Đề 2 | C Như | | C Na | | C Lan | |
| | P.Biện | T Phúc | C Thư | T Lộc | C Quí | C Hiền | C Tú |
| **CK2** | Đề 1 | C Tú | T Nghĩa | C Thư | T Nghĩa | T Phúc | T Nghĩa |
| | Đề 2 | T Lộc | | C Quí | | C Hiền | |
| | P.Biện | C Như | C Lan | C Na | C Lài | C Bình | C Hiền |

## 7. Đánh giá Preset 'Cho phép dồn việc trong một kỳ' (Tắt S9)
- **Mục đích:** Kiểm tra khả năng đạt 0 điểm phạt mềm khi cho phép dồn việc (S9 = 0).
- Tổng điểm phạt với S9 tắt (nocampus): Có S3 = **176.36**, Không tính S3 = **32.36**
- Chi tiết các quy tắc còn điểm phạt:
  * **S5**: 1.00 đơn vị vi phạm (Phạt: 6.00)
  * **S6**: 3.00 đơn vị vi phạm (Phạt: 6.00)
  * **S8**: 2.55 đơn vị vi phạm (Phạt: 20.36)
- **Kết luận:** Khi tắt S9, optimizer có đạt 0 điểm phạt mềm không? -> **KHÔNG (Vẫn còn vi phạm cận dưới không thể giảm thêm của S5 và S6)**
  * Giải thích: Do S5 có cận dưới lý thuyết >= 1.0 (nguyên lý Dirichlet trên môn Công nghệ) và S6 có cận dưới lý thuyết = 2.0 (bài toán cân bằng lượt ra đề 24 lượt / 11 GV), nên ngay cả khi dồn việc tự do, tổng điểm phạt vẫn không thể đạt 0 tuyệt đối.

## 8. Giải trình cơ sở toán học của các cận dưới (Lower Bounds)
1. **Cận dưới S6 = 2.0:** Trong 4 kỳ thi, có 24 lượt giáo viên ra đề không bắt buộc (12 ban đề VL x 2) cần phân bổ cho 11 giáo viên đủ điều kiện (k_t <= 4, Thầy Nghĩa được loại trừ do 12 ghế ra đề CN của thầy là bắt buộc cố định). Để cực tiểu hoá tổng hàm lồi f(k_t) với f(k) = max(0, 2k - 5), thuật toán tham lam cân bằng tối ưu phân bổ 9 giáo viên nhận 2 lượt ra đề (f = 0) và 2 giáo viên nhận 3 lượt ra đề (f = 1 mỗi người), xác lập cận dưới chính xác tuyệt đối là 2.0 đơn vị.
2. **Cận dưới S10 = 0.0:** Cả 11 giáo viên (GV001..GV011) đều có chuyên môn phản biện cho cả 2 môn Vật lí (VL) và Công nghệ (CN). Toàn bộ năm học có 12 ghế phản biện VL và 12 ghế phản biện CN. Vì 11 <= 12 ở cả hai môn, về mặt toán học hoàn toàn tồn tại phân công để mỗi giáo viên đều được phản biện mỗi môn ít nhất 1 lần.
3. **Cận dưới S5 >= 1.0:** Ở môn Công nghệ, toàn bộ 12 ban đề đều do Thầy Nghĩa ra đề độc quyền, và chỉ có 11 giáo viên khác có thể làm phản biện. Theo nguyên lý Dirichlet (Pigeonhole Principle), phân phối 12 lượt phản biện cho 11 người chắc chắn sẽ có ít nhất một giáo viên phải phản biện cho Thầy Nghĩa ít nhất ceil(12/11) = 2 lần. Mỗi lượt phản biện lặp lại tạo ra max(0, count - 1) >= 1 đơn vị vi phạm. Do đó S5 >= 1.0.
4. **Giải trình S3 trên mô hình đơn cơ sở (nocampus) = 36.0 đơn vị:** Trên mô hình nocampus, tất cả giáo viên cùng thuộc Cơ sở 1. Đối với mỗi ban đề Vật lí (2 người ra đề, 1 người phản biện), người phản biện trùng cơ sở với 2 người ra đề, tạo ra 12 x 2 = 24 đơn vị vi phạm. Đối với mỗi ban đề Công nghệ (1 người ra đề, 1 người phản biện), người phản biện trùng cơ sở với 1 người ra đề, tạo ra 12 x 1 = 12 đơn vị vi phạm. Tổng số đơn vị vi phạm S3 trên nocampus = 24 + 12 = 36.0 đơn vị.
