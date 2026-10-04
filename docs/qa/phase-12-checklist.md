# Quy trình Kiểm thử và Bảng kiểm QA (Manual & Automated QA Checklist) - Phase 12

Tài liệu này cung cấp các kịch bản kiểm thử toàn diện trên ứng dụng Desktop thực tế (`pnpm tauri dev`), Web Mock (`pnpm -C ui dev`) và các công cụ dòng lệnh cho các tính năng trọng tâm của Phase 12:
- Bảng phân công dạng lưới mẫu tổ (Q-Style Grid) tại `/assignments`.
- Xuất bản Excel và Trang in PDF chuẩn mẫu tổ chuyên môn trường phổ thông.
- Trình nhập bảng phân công có sẵn (Import Existing Plan Wizard) từ Excel / TSV Clipboard.
- Biểu mẫu nhập liệu v2: Nhập danh mục Môn thi, Ma trận chuyên môn và Giáo viên nâng cao.
- Kiểm định hiệu năng thuật toán và tính đúng đắn trên bộ dữ liệu mẫu tổ Q-shaped.

---

## Bảng tổng hợp trạng thái các phần (Phase 12 Parts Checklist)

| Phần | Nội dung chính | Trạng thái | Minh chứng / Thao tác kiểm thử |
| :--- | :--- | :---: | :--- |
| **Part A** | Xử lý tồn đọng Phase 11.1 & Hạ tầng | [x] Đạt | Script `scripts/test-inventory.mjs`, Cận dưới S6 đa môn, Cảnh báo H3 `h3_reviewer_pool_reduced`, Warm vs Cold audit |
| **Part B** | Giao diện Lưới mẫu tổ (Q-Style Grid) | [x] Đạt | Chuyển đổi "Bảng tổ" / "Chi tiết", Bảng tổng hợp bên phải, Hover làm nổi bật, Thao tác chỉnh sửa đầy đủ |
| **Part C** | Xuất Excel và Trang in mẫu tổ | [x] Đạt | Sheet "Bảng phân công (mẫu tổ)", Route `/print/plan/:id`, Khớp calamine golden, Xuất PDF A4 nằm ngang |
| **Part D** | Trình nhập bảng phân công có sẵn | [x] Đạt | Nhập file .xlsx mẫu tổ & dán TSV, So khớp tên không dấu NFC/NFD, Xem trước vi phạm/điểm số, Nguồn 'manual' |
| **Part E** | Biểu mẫu nhập v2 (Môn & Chuyên môn) | [x] Đạt | Sheet "Môn" & "Môn đảm nhiệm", Mô phỏng khả thi có H3/cố định, Tự động sao lưu dữ liệu trước khi áp dụng |
| **Part F** | Đánh giá dữ liệu mẫu tổ & Hiệu năng | [x] Đạt | Benchmark `solve_hard` < 50ms (0.78ms), Đọc calamine < 1ms (0.50ms), Bảng `comparison.md` 12 cột chuẩn xác |
| **Part G** | Hoàn thiện tài liệu, ADR-0043 & QA | [x] Đạt | Cập nhật SPEC, ARCHITECTURE, DECISIONS (ADR-0043), CHANGELOG, Check-all sạch 100% |

---

## Chuẩn bị môi trường thử nghiệm
1. Khởi chạy ứng dụng:
   ```powershell
   pnpm tauri dev
   ```
   *(Hoặc kiểm thử nhanh trên trình duyệt: `pnpm -C ui dev`)*
2. Tạo một năm học mới hoặc nạp dữ liệu mẫu Q-shaped (12 giáo viên, 2 môn VL/CN, 4 đợt thi, 3 khối).

---

## Các kịch bản kiểm thử chi tiết (Step-by-Step QA Scenarios)

### Kịch bản 1: Trải nghiệm Lưới phân công mẫu tổ (Q-Style Grid) tại `/assignments`
- **Các bước thực hiện:**
  1. Vào trang **Phân công** (`/assignments`) và chọn một kế hoạch đã giải.
  2. Quan sát thanh chuyển đổi góc trên: Nút **"Bảng tổ"** (mặc định) và **"Chi tiết"**.
  3. Kiểm tra bố cục lưới dạng tổ:
     - Góc trên cùng bên trái: Ô tiêu đề `"Kì thi/khối"`.
     - Cột: Gom nhóm theo Khối (`Khối 10`, `Khối 11`, `Khối 12`), bên dưới là các cột Môn (`VL`, `CN`) kèm màu nhận diện môn.
     - Hàng: Gom khối theo từng Kỳ thi (`GK1`, `CK1`, `GK2`, `CK2`), chia các hàng vai trò `"Đề"` và `"P.Biện"`. Đối với môn CN chỉ có 1 người ra đề, ô Đề thứ hai được để trống tự nhiên.
     - Ô hiển thị: Ưu tiên cách gọi của giáo viên (ví dụ: `C Quí`, `T Nghĩa`), mật độ hiển thị vừa vặn trên màn hình chuẩn 1280x720.
  4. Quan sát **Bảng tổng hợp gắn kèm bên phải** (Totals panel):
     - Gồm các cột: `GV`, `Tổng lượt`, `Đề`, `PB`, `GK1`, `CK1`, `GK2`, `CK2`.
     - Giáo viên có nhiệm vụ cố định (T Nghĩa) có nhãn `Cố định`.
     - Các ô có từ 2 nhiệm vụ trong 1 kỳ trở lên có huy hiệu cảnh báo mật độ; từ 3 nhiệm vụ trở lên có cảnh báo đậm.
     - Hàng tổng cộng (Total row) ở cuối bảng hiển thị tổng số lượt phân công (chuẩn 60 lượt).
  5. Thử rê chuột (Hover) hoặc bấm vào tên giáo viên trên lưới hoặc bảng tổng hợp:
     - Tất cả các vị trí được phân công của giáo viên đó trên lưới sáng đồng thời.
     - Tooltip hiển thị: Số lượt/Chỉ tiêu, danh sách vai trò và các kỳ thi tham gia.
  6. Thử chỉnh sửa trên lưới:
     - Bấm hoặc ấn `Enter` vào một ô: Hộp thoại chọn ứng viên hiện ra, sắp xếp theo độ chênh điểm số ($\Delta$ score), các ứng viên vi phạm bị vô hiệu hóa kèm lý do rõ ràng.
     - Kéo thả để hoán đổi 2 vị trí (Drag-to-swap).
     - Phím tắt Hoàn tác (`Ctrl+Z`) và Làm lại (`Ctrl+Y`).
     - Bấm chuột phải mở menu ngữ cảnh: Ghim vị trí (Pin), Cấm vị trí (Forbid).
     - Thử chọn ô có nhiệm vụ cố định (T Nghĩa): Kiểm tra vị trí hoàn toàn bất khả xâm phạm (Immovable).
- **Kết quả mong đợi:** [x] Giao diện hiển thị đúng chuẩn tài liệu của tổ, chuyển đổi mượt mà, thao tác chỉnh sửa tức thời và cập nhật điểm số chính xác.

---

### Kịch bản 2: Xuất Excel và In ấn chuẩn mẫu tổ
- **Các bước thực hiện:**
  1. Tại trang chi tiết kế hoạch, bấm nút **Xuất Excel** và mở tệp tải về.
  2. Quan sát cấu trúc trang tính:
     - Trang tính đầu tiên có tên: **"Bảng phân công (mẫu tổ)"**.
     - Khung thông tin trường/tổ bộ môn ở trên cùng; kế hoạch nháp có đóng dấu `"BẢN NHÁP"`.
     - Tiêu đề gộp ô `"Kì thi/khối"`, khối gộp trên các môn, nền màu phân biệt kỳ thi và môn thi rõ ràng.
     - Sau bảng lưới là cột cách trống (spacer), tiếp đến bảng tổng hợp `GV`, `Tổng lượt n.vụ`, `Đề`, `PB`, `GK1`..`CK2`.
     - Dưới cùng có phần ký duyệt: Ngày tháng, chức danh người lập/ký và tên người ký.
     - Bản in đọc rõ ràng trong chế độ đơn sắc trắng đen (grayscale).
  3. Bấm nút **In ấn / Xuất PDF** hoặc truy cập URL `/print/plan/:id`:
     - Trang in hiển thị trọn vẹn toàn bộ bảng mẫu tổ trên 1 trang khổ ngang A4 (Landscape).
- **Kết quả mong đợi:** [x] Tệp Excel và trang in PDF khớp 100% định dạng biểu mẫu tổ của trường học.

---

### Kịch bản 3: Nhập bảng phân công có sẵn (Import Plan Wizard)
- **Các bước thực hiện:**
  1. Vào menu **Kế hoạch** (`/plans`) hoặc nút công cụ, chọn **"Nhập bảng phân công có sẵn"**.
  2. Chọn phương thức nhập:
     - Cách 1: Tải lên tệp `.xlsx` mẫu tổ vừa xuất ở Kịch bản 2.
     - Cách 2: Sao chép vùng dữ liệu dạng bảng từ Excel và dán (Paste TSV) vào khung văn bản.
  3. Hệ thống tự động nhận diện:
     - Các cột kỳ thi, khối lớp, môn thi, vai trò ra đề/phản biện.
     - Nhận diện giáo viên theo: Cách gọi -> Họ và tên -> Tên bỏ dấu không phân biệt chữ hoa/thường (hỗ trợ cả Unicode NFC và NFD).
  4. Xem màn hình **Xem trước (Preview)**:
     - Bảng ma trận được ánh xạ, bảng tổng hợp số lượt từng giáo viên.
     - Kiểm tra cảnh báo nếu có ô thiếu người hoặc số tổng cộng lệch so với file gốc.
     - Báo cáo số vi phạm ràng buộc cứng và điểm mục tiêu mềm tính theo bộ quy tắc hiện hành.
  5. Bấm **"Áp dụng và lưu kế hoạch"**:
     - Kế hoạch mới được tạo với nguồn gốc `manual`, thông số `run_params_json.origin = "import"`.
     - Kế hoạch sau khi nhập có thể chỉnh sửa, so sánh và khóa chốt như mọi kế hoạch khác.
- **Kết quả mong đợi:** [x] Nhập tròn vẹn (round-trip) tạo ra đúng 60 nhiệm vụ, tổng số lượt chuẩn xác từng giáo viên.

---

### Kịch bản 4: Biểu mẫu nhập liệu v2 — Môn thi, Chuyên môn & Cố định nâng cao
- **Các bước thực hiện:**
  1. Vào trang **Cài đặt** (`/settings`) hoặc menu Nhập liệu, chọn tải mẫu **"Biểu mẫu v2 (Kèm môn & chuyên môn)"**.
  2. Mở file mẫu Excel và kiểm tra các sheet:
     - Sheet **"Môn"**: Khai báo danh mục môn, số lượng ra đề, phản biện, phân hiệu tối thiểu, màu sắc.
     - Sheet **"Môn đảm nhiệm"**: Phân công vai trò (Ra đề / Phản biện / Cả hai) và phạm vi (Theo khối dạy / Mọi khối) cho từng giáo viên.
     - Sheet **"Giáo viên"**: Có thêm cột `Cách gọi`, `Chỉ tiêu riêng`, `Số việc tối đa mỗi kỳ`.
  3. Tải tệp dữ liệu mẫu `template-v2-q-sample.xlsx` vào hệ thống.
  4. Quan sát màn hình mô phỏng tính khả thi (Feasibility Simulation):
     - Hệ thống cảnh báo nếu có giáo viên chưa được gán chuyên môn (`teacher_no_competency`).
     - Cảnh báo H3 nếu quy định phân hiệu làm giảm nguồn cán bộ phản biện (`h3_reviewer_pool_reduced`).
  5. Bấm **"Áp dụng"**:
     - Hệ thống tự động tạo bản sao lưu dữ liệu (Automatic backup).
     - Toàn bộ danh mục môn, chuyên môn và giáo viên được cập nhật nguyên tử (Atomic).
- **Kết quả mong đợi:** [x] Dữ liệu nạp đầy đủ, nhất quán; cơ sở dữ liệu được sao lưu an toàn trước khi thay đổi.

---

### Kịch bản 5: Kiểm tra tự động bằng lệnh (Automated CLI Verification)
- **Các lệnh kiểm tra nghiệm thu:**
  ```powershell
  # 1. Kiểm tra toàn bộ mã nguồn, lint, test và build
  pnpm check-all

  # 2. Kiểm kê số lượng bài kiểm thử
  node scripts/test-inventory.mjs

  # 3. Đo lường hiệu năng và so sánh giải thuật
  cargo test -p exam_panel_service --test phase12_real_data_validation -- --nocapture
  ```
- **Kết quả nghiệm thu:**
  - `cargo fmt --check`: Sạch sẽ, không lệch định dạng.
  - `cargo clippy -- -D warnings`: Không có cảnh báo.
  - `cargo test`: 202 bài kiểm tra backend vượt qua (0 thất bại).
  - `pnpm -C ui lint`: Không có cảnh báo lint.
  - `pnpm -C ui typecheck`: 0 lỗi TypeScript.
  - `pnpm -C ui test`: 77 bài kiểm tra frontend Vitest vượt qua.
  - `pnpm -C ui build`: Biên dịch production thành công.
