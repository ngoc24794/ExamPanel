# Quy trình Kiểm thử Thủ công (Manual QA Checklist) - Phase 11 & Phase 11.1

Tài liệu này cung cấp các kịch bản kiểm thử tuần tự trên ứng dụng Desktop thực tế (`pnpm tauri dev`) cho các tính năng Phase 11 và sửa đổi Phase 11.1: Quản lý nhiều môn thi (Multi-Subject Model), Ma trận chuyên môn (/competencies), Cố định nhiệm vụ (Forced Placements) và Huy hiệu chỉ tiêu, Ràng buộc H3/H4 và Hàm mục tiêu S1-S10, In ấn & Xuất Excel có cột Môn, Nâng cấp di trú cơ sở dữ liệu v4→v5.

---

## Chuẩn bị môi trường
1. Khởi chạy ứng dụng:
   ```powershell
   pnpm tauri dev
   ```
2. Nạp dữ liệu mẫu thử nghiệm hoặc nạp dữ liệu Q-shaped demo:
   - Dữ liệu demo mặc định có 4 phân hiệu chuẩn ("Phân hiệu 1-4"), môn Chung và môn chuyên biệt.
3. Kiểm tra thông tin phiên bản tại **Cài đặt** (`/settings`):
   - Đảm bảo cơ sở dữ liệu hiển thị phiên bản schema v5.

---

## Các kịch bản kiểm thử (Numbered QA Steps)

### Kịch bản 1: Quản lý danh mục Môn thi (`/subjects`)
- **Thao tác:**
  1. Vào menu **Môn thi** (`/subjects`).
  2. Bấm nút **Thêm môn thi** (`create-subject-btn`).
  3. Nhập:
     - Mã môn thi: `VL`
     - Tên môn thi: `Vật lí`
     - Số cán bộ ra đề: `2`
     - Số cán bộ phản biện: `1`
     - Tối thiểu phân hiệu: `2`
     - Chọn màu sắc đại diện (palette color).
  4. Bấm **Lưu**.
  5. Thử tạo tiếp môn thi thứ hai: Mã `CN`, Tên `Công nghệ`, Ra đề `1`, Phản biện `1`, Tối thiểu phân hiệu `1`.
- **Kết quả mong đợi:**
  - Cả 2 thẻ môn thi `VL` và `CN` xuất hiện trong lưới môn thi với thông số định mức chính xác.
  - Hệ thống kiểm tra tính duy nhất của mã môn thi (không cho phép tạo trùng mã).

---

### Kịch bản 2: Ma trận phân công Chuyên môn (`/competencies`)
- **Thao tác:**
  1. Vào menu **Chuyên môn** (`/competencies`) từ thanh điều hướng bên trái hoặc từ nút "Chuyên môn" tại trang Giáo viên.
  2. Quan sát ma trận: Hàng là danh sách giáo viên đang hoạt động, Cột là các môn thi (CHUNG, VL, CN).
  3. Tại ô giao của một giáo viên và môn `VL`:
     - Bấm nút **Đề:** để chuyển trạng thái xoay vòng:
       - Lần 1: Chuyển sang **Theo khối dạy** (màu xanh dương).
       - Lần 2: Chuyển sang **Mọi khối** (màu xanh ngọc).
       - Lần 3: Chuyển về **Tắt** (màu xám).
     - Bấm nút **PB:** để chuyển đổi chuyên môn phản biện tương tự.
  4. Sử dụng thanh công cụ lọc:
     - Gõ tên giáo viên vào ô tìm kiếm.
     - Lọc theo phân hiệu hoặc khối dạy.
  5. Thử nút thao tác hàng loạt:
     - Bấm **Gán toàn bộ (Mọi khối)** và xác nhận.
     - Bấm **Xóa toàn bộ** và xác nhận.
  6. Quan sát thanh thống kê dưới chân trang (Footer):
     - Hiển thị tổng số giáo viên đủ điều kiện ra đề và phản biện cho từng môn thi.
- **Kết quả mong đợi:**
  - Các thao tác chuyển trạng thái lưu ngay lập tức vào cơ sở dữ liệu.
  - Số liệu footer cập nhật tức thời theo số lượng giáo viên được phân quyền chuyên môn.

---

### Kịch bản 3: Giáo viên — "Cách gọi", Chỉ tiêu cố định & Huy hiệu nhiệm vụ cố định (`/teachers`)
- **Thao tác:**
  1. Vào trang **Giáo viên** (`/teachers`).
  2. Bấm nút **Thêm giáo viên** (hoặc chỉnh sửa một giáo viên hiện có):
     - Họ tên: `Lê Văn Tám`
     - Cách gọi (display_name): `Thầy Tám (Toán)`
     - Mở rộng mục **Cấu hình nâng cao (Advanced)**:
       - Chỉ tiêu nhiệm vụ cố định (quota_override): `6`
       - Tối đa nhiệm vụ / đợt thi (max_tasks_per_exam_override): `1`
  3. Bấm **Lưu**.
  4. Xem thông tin giáo viên trên bảng:
     - Dưới họ tên hiển thị dòng phụ: `Cách gọi: Thầy Tám (Toán)`.
  5. Đối với giáo viên có phân công cố định (Forced Placement, ví dụ thầy T Nghĩa trong dữ liệu mẫu):
     - Cạnh họ tên xuất hiện huy hiệu nổi bật: `Cố định: 12`.
- **Kết quả mong đợi:**
  - Trường cách gọi hiển thị trực quan và lưu trữ thành công.
  - Huy hiệu cố định phản ánh đúng số ghế được ghim trước vào bài toán.

---

### Kịch bản 4: Cấu hình Quy tắc — H3 Toggle, H4 Limits, S1 Auto & S9/S10 (`/rules`)
- **Thao tác:**
  1. Vào menu **Quy tắc** (`/rules`).
  2. Tại tab **Ràng buộc cứng (Hard Rules)**:
     - Kiểm tra thẻ **H3 - Đa dạng phân hiệu trong hội đồng**: Có công tắc Bật/Tắt (`toggle-h3`). Khi tắt, hiển thị khung cảnh báo màu vàng.
     - Kiểm tra thẻ **H4 - Giới hạn nhiệm vụ trong cùng kỳ thi**: Khi bật, hiển thị 2 trường số cho phép nhập:
       - "Tối đa nhiệm vụ / đợt" (mặc định 2).
       - "Tối đa ra đề / đợt" (mặc định 1).
  3. Chuyển sang tab **Mục tiêu mềm (Soft Rules)**:
     - Kiểm tra thẻ **S1**: Có nút chọn chế độ **Tự động (Auto)** hoặc nhập số nguyên cụ thể.
     - Kiểm tra sự xuất hiện của hai quy tắc mới:
       - **S9**: Tránh tập trung nhiệm vụ dồn vào cùng kỳ thi (Trừ bù dồn ép tránh được).
       - **S10**: Cân đối phân công phản biện chéo môn.
     - Thử kéo thanh trượt trọng số (Weight slider) của S9 và S10.
  4. Thử bấm các bộ cấu hình mẫu (Presets):
     - **Cân bằng (Balanced)**: Trọng số phân bổ chuẩn.
     - **Công bằng khối lượng (Workload Fairness)**: S1, S8, S9 được tăng trọng số.
     - **Đa dạng phân hiệu & Hội đồng (Team Diversity)**: S3, S4, S5, S10 được tăng trọng số.
  5. Bấm **Lưu thay đổi** (`save-rules-btn`).
- **Kết quả mong đợi:**
  - Quy tắc lưu thành công, thông báo toast xác nhận. Khi tải lại trang, các giá trị tùy chỉnh được giữ nguyên.

---

### Kịch bản 5: Bảng kiểm tra tính khả thi & Thông tin phân công cố định (`FeasibilitySheet`)
- **Thao tác:**
  1. Trên thanh tiêu đề ứng dụng, bấm nút **Tính khả thi** (Feasibility Indicator).
  2. Ngăn trượt (Sheet) mở ra từ cạnh phải màn hình.
  3. Kiểm tra mục **Phân công cố định (Forced Placements)**:
     - Liệt kê chi tiết các nhiệm vụ cố định: Tên giáo viên, Kỳ thi, Khối, Môn, và Vai trò (Ra đề / Phản biện).
  4. Nếu có lỗi về chuyên môn (ví dụ xóa hết chuyên môn của giáo viên hoặc cố định vượt định mức), kiểm tra các mã chẩn đoán:
     - `teacher_no_competency`, `no_competencies`: Nút "Đi đến sửa" chuyển thẳng đến trang `/competencies`.
     - `forced_tasks_over_limit`, `forced_setter_over_limit`: Nút "Đi đến sửa" chuyển đến `/teachers`.
- **Kết quả mong đợi:**
  - Thông tin phân công cố định hiển thị rõ ràng, giúp người điều hành nắm bắt toàn bộ ghế đã ghim trước khi chạy thuật toán.

---

### Kịch bản 6: Không gian Phân công — Khối môn học trong ô, Thao tác thay thế & So sánh phương án (`/assignments`)
- **Thao tác:**
  1. Vào trang **Phân công** (`/assignments`).
  2. Bấm nút **Tự động phân công (Run Optimizer)**, chọn cấu hình nỗ lực và bấm chạy.
  3. Quan sát ma trận phân công:
     - Mỗi ô phân công (Kỳ thi × Khối) hiển thị rõ các khối môn học tương ứng.
     - Mỗi vị trí có huy hiệu vai trò `Đề` (màu trung tính) hoặc `PB` (màu chính với biểu tượng con mắt).
     - Hiển thị chấm màu phân hiệu công tác của giáo viên.
     - Hiển thị cách gọi (hoặc họ tên) của giáo viên.
  4. Thử bộ lọc môn học trên thanh điều khiển:
     - Chuyển giữa "Tất cả các môn" và từng môn riêng lẻ.
  5. Thao tác điều chỉnh thủ công:
     - Bấm vào một vị trí giáo viên trên ma trận. Hộp thoại **Chọn cán bộ thay thế** (CandidateSelectModal) xuất hiện.
     - Danh sách ứng viên lọc đúng theo chuyên môn (Chỉ hiện giáo viên có quyền ra đề hoặc phản biện môn đó).
     - Chọn giáo viên mới và áp dụng.
  6. Bấm nút **So sánh phương án** (`compare-plans-button`):
     - Hộp thoại so sánh hiển thị điểm phạt phân rã theo 10 quy tắc (S1→S10) cho từng phương án.
- **Kết quả mong đợi:**
  - Ma trận hiển thị đầy đủ thông tin đa môn học mà không bị vỡ bố cục.
  - Bộ giải tôn trọng 100% chuyên môn và các vị trí cố định.

---

### Kịch bản 7: In ấn và Xuất Excel có cột "Môn"
- **Thao tác:**
  1. Tại trang **Phân công**, bấm nút **In ấn & Xuất dữ liệu**:
     - Chọn **In bảng phân công tổng hợp**: Màn hình xem trước in hiển thị bảng ma trận với cột **Môn**, phân rã theo (Kỳ thi, Khối, Môn).
     - Chọn **In giấy báo phân công cá nhân**: Mỗi giáo viên có 1 trang in, bảng nhiệm vụ có cột **Môn** và danh sách "Thành viên cùng ban đề" lọc đúng theo cùng môn thi.
     - Chọn **Xuất tệp Excel (.xlsx)**: Mở tệp Excel vừa xuất, kiểm tra sheet "Theo giáo viên" (Sheet 2) có cột **Môn**.
  2. Tại trang **Giáo viên**, bấm **Tải file mẫu**:
     - Mở tệp `mau-nhap-du-lieu.xlsx`, kiểm tra sheet "Giáo viên" có thêm cột **Cách gọi**.
- **Kết quả mong đợi:**
  - Mọi báo cáo in ấn và tệp xuất bản đều có đầy đủ thông tin phân loại môn học và cách gọi thân thiện.

---

### Kịch bản 8: Sao lưu & Phục hồi — Tương thích ngược Migration v4→v5 (`/settings`)
- **Thao tác:**
  1. Vào menu **Cài đặt** (`/settings`), phần **Sao lưu & Phục hồi**.
  2. Bấm nút **Phục hồi từ file** (`restore-file-btn`).
  3. Chọn một tệp cơ sở dữ liệu backup từ Phase 4 (hoặc file v4 fixture).
  4. Xác nhận phục hồi.
- **Kết quả mong đợi:**
  - Hệ thống tự động phát hiện cơ sở dữ liệu phiên bản cũ, tự động chạy di trú 0005 (`up.sql`).
  - Toàn bộ giáo viên cũ được tự động cấp quyền chuyên môn cho môn mặc định `"CHUNG"`.
  - Toàn bộ phương án và lịch phân công cũ giữ nguyên tính hợp lệ cứng (Hard validity) và điểm phạt (Score).
