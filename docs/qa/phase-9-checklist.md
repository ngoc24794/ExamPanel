# Quy trình Kiểm thử Thủ công (Manual QA Checklist) - Phase 9

Tài liệu này cung cấp các bước kiểm thử tuần tự trên ứng dụng Desktop thực tế (`pnpm tauri dev`) cho các tính năng Phase 9: Nhập/Xuất Excel, In ấn & Giấy báo phân công, Sao lưu & Phục hồi dữ liệu SQLite.

---

## Chuẩn bị môi trường
1. Khởi chạy ứng dụng:
   ```powershell
   pnpm tauri dev
   ```
2. Vào mục **Cài đặt** (`/settings`):
   - Đảm bảo dữ liệu demo đã được nạp (hoặc nạp mới qua Dev Tools).
   - Thiết lập "Thông tin đơn vị":
     - Tên trường: `TRƯỜNG THPT CHUYÊN HÀ NỘI - AMSTERDAM`
     - Tổ chuyên môn: `TỔ TOÁN - TIN`
     - Chức vụ người ký: `TỔ TRƯỞNG CHUYÊN MÔN`
     - Họ tên người ký: `Nguyễn Văn An`
     - Địa danh: `Hà Nội`
     - Bấm **Lưu thông tin đơn vị**.

---

## Các kịch bản kiểm thử (Numbered QA Steps)

### Kịch bản 1: Tải tệp mẫu Excel (`mau-nhap-du-lieu.xlsx`)
- **Thao tác:**
  1. Vào menu **Giáo viên** (`/teachers`).
  2. Bấm nút **Tải file mẫu** ở góc trên bên phải.
  3. Chọn thư mục lưu tệp `mau-nhap-du-lieu.xlsx` trong hộp thoại lưu file của hệ điều hành.
  4. Mở tệp vừa tải bằng Microsoft Excel.
- **Kết quả mong đợi:**
  - Tệp có 4 trang tính: "Hướng dẫn", "Phân hiệu", "Giáo viên", "Lịch vắng".
  - Dòng tiêu đề được cố định (freeze panes) và định dạng đậm.
  - Các cột có Data Validation danh sách thả xuống: Mã phân hiệu, Khối dạy, Hệ số tải (0, 0.25, 0.5, 0.75, 1), Đang dạy (Có, Không), Mã kỳ thi (GK1, CK1, GK2, CK2).

### Kịch bản 2: Nhập dữ liệu từ Excel - Chế độ "Thêm và cập nhật" (Upsert)
- **Thao tác:**
  1. Trong Excel, thêm 1 giáo viên mới vào sheet "Giáo viên": Mã `GV999`, Họ tên `Trần Văn Thử Nghiệm`, Phân hiệu `CS1`, Khối dạy `10, 11`, Hệ số tải `1`, Đang dạy `Có`.
  2. Lưu tệp Excel.
  3. Trong ExamPanel, tại trang **Giáo viên**, bấm nút **Nhập từ Excel**.
  4. Bấm nút **Duyệt tệp** và chọn file Excel vừa lưu.
  5. Chọn chế độ: **Thêm và cập nhật**.
  6. Bấm **Xem trước kết quả**.
  7. Kiểm tra các tab: "Giáo viên", "Phân hiệu", "Lịch vắng", bấm thử các bộ lọc trạng thái (Mới, Cập nhật, Không đổi, Lỗi).
  8. Bấm **Xác nhận & Áp dụng**.
- **Kết quả mong đợi:**
  - Bảng xem trước hiển thị dòng giáo viên `GV999` với huy hiệu xanh **Mới**.
  - Không có lỗi đỏ chặn áp dụng (`can_apply = true`).
  - Hệ thống tự động sao lưu cơ sở dữ liệu trước khi ghi nhận (`data/backups/`).
  - Thông báo thành công hiển thị số lượng giáo viên tạo mới.
  - Danh sách giáo viên ngoài màn hình tự động làm mới và xuất hiện giáo viên mới.

### Kịch bản 3: Nhập dữ liệu từ Excel - Chế độ "Đồng bộ danh sách giáo viên" (Sync)
- **Thao tác:**
  1. Mở lại hộp thoại **Nhập từ Excel**, chọn file Excel.
  2. Chọn chế độ **Đồng bộ danh sách giáo viên**.
  3. Bấm **Xem trước kết quả**.
  4. Chuyển sang tab **Ngừng hoạt động**.
  5. Kiểm tra danh sách các giáo viên có trong hệ thống nhưng không có trong file Excel.
  6. Bấm **Xác nhận & Áp dụng**.
- **Kết quả mong đợi:**
  - Tab "Ngừng hoạt động" hiển thị danh sách các giáo viên sẽ bị chuyển trạng thái `active = false`.
  - Sau khi áp dụng, các giáo viên này chuyển sang trạng thái "Tạm dừng" trong bảng giáo viên mà không bị xóa mất dữ liệu lịch sử phân công.

### Kịch bản 4: Xuất sổ bảng tính phân công ra Excel (`.xlsx`)
- **Thao tác:**
  1. Vào menu **Phân công** (`/assignments`), chọn một phương án phân công (VD: Kế hoạch #1).
  2. Bấm nút **Xuất Excel (.xlsx)** trên thanh công cụ của phương án.
  3. Chọn đường dẫn lưu tệp trong hộp thoại Desktop.
  4. Mở tệp vừa xuất bằng Microsoft Excel.
- **Kết quả mong đợi:**
  - Tên tệp mặc định theo chuẩn: `phan-cong-<năm học>-<tên phương án>.xlsx`.
  - Sheet 1 "Phân công": Trình bày dạng ma trận theo kỳ thi × khối lớp, có tiêu đề trường, tổ chuyên môn, định dạng A4 ngang, vừa khít chiều rộng 1 trang giấy, có khối chữ ký của tổ trưởng/hiệu trưởng.
  - Nếu là bản nháp, có tiền tố `[BẢN NHÁP]` rõ ràng.
  - Sheet 2 "Theo giáo viên": Danh sách từng lượt phân công kèm thông tin các giáo viên cùng ban đề.
  - Sheet 3 "Thống kê": Thống kê số lượt ra đề, phản biện, chỉ tiêu và độ lệch của từng giáo viên.
  - Sheet 4 "Tiêu chí": Chi tiết điểm phạt, chỉ tiêu cận dưới và trọng số các luật mềm.

### Kịch bản 5: In ma trận phân công và Xuất PDF
- **Thao tác:**
  1. Tại màn hình **Phân công**, bấm nút **In / Xuất PDF** $\rightarrow$ chọn **In bảng phân công**.
  2. Giao diện chuyển sang trang `/print/plan/:id` không chứa thanh điều hướng hoặc app chrome.
  3. Bấm nút **In / Xuất PDF** (hoặc nhấn Ctrl+P).
  4. Trong hộp thoại in của hệ điều hành, chọn máy in **Microsoft Print to PDF**, hướng giấy Ngang (Landscape).
  5. Lưu thành file PDF và mở kiểm tra.
- **Kết quả mong đợi:**
  - Trang in định dạng khổ A4 ngang chuẩn xác.
  - Hiển thị đầy đủ quốc hiệu, tiêu ngữ, thông tin trường/tổ, ma trận phân công và chữ ký.
  - Nếu là phương án chưa khóa chính thức, hiển thị watermark chìm `BẢN NHÁP`.

### Kịch bản 6: In giấy báo phân công từng giáo viên
- **Thao tác:**
  1. Tại màn hình **Phân công**, bấm nút **In / Xuất PDF** $\rightarrow$ chọn **In giấy báo phân công**.
  2. Giao diện chuyển sang trang `/print/notices/:id`.
  3. Thử nghiệm bộ lọc: "Chỉ giáo viên có nhiệm vụ" và "Tất cả giáo viên".
  4. Bấm nút **In / Xuất PDF** (hoặc Ctrl+P), chọn khổ Dọc (Portrait).
  5. Kiểm tra bản xem trước in.
- **Kết quả mong đợi:**
  - Mỗi giáo viên được ngắt trang riêng biệt (`break-after: page`), không bị tràn dòng hay cắt đôi bảng nhiệm vụ sang trang khác.
  - Thể hiện rõ tên giáo viên, phân hiệu, bảng chi tiết các kỳ thi và khối lớp được giao, ghi chú bảo mật đề thi và phần chữ ký xác nhận.

### Kịch bản 7: Sao lưu cơ sở dữ liệu SQLite tức thời
- **Thao tác:**
  1. Vào menu **Cài đặt** (`/settings`).
  2. Cuộn tới phần **Sao lưu & Phục hồi dữ liệu**.
  3. Bấm nút **Sao lưu ngay**.
  4. Chọn vị trí lưu tệp (tên mặc định dạng `exampanel-backup-YYYYMMDD-HHmm.db`).
- **Kết quả mong đợi:**
  - Quá trình SQLite Online Backup API diễn ra an toàn không khóa giao dịch đọc.
  - Thông báo thành công hiển thị.
  - Tệp `.db` được tạo có dung lượng hợp lệ (> 100 KB).

### Kịch bản 8: Phục hồi cơ sở dữ liệu từ tệp sao lưu
- **Thao tác:**
  1. Trong phần **Sao lưu & Phục hồi dữ liệu**, bấm nút **Phục hồi từ tệp**.
  2. Chọn tệp sao lưu đã tạo ở Kịch bản 7 (hoặc chọn 1 file giả/hỏng để thử nghiệm lỗi).
  3. Hộp thoại **Kiểm tra tệp sao lưu** xuất hiện.
  4. Xem bảng thông tin kiểm tra toàn vẹn: số năm học, số giáo viên, số phương án, phiên bản schema.
  5. Bấm nút **Xác nhận phục hồi**.
- **Kết quả mong đợi:**
  - Hệ thống tự động tạo 1 bản sao lưu dự phòng cho cơ sở dữ liệu hiện tại trước khi thực hiện ghi đè.
  - Cơ sở dữ liệu được khôi phục nguyên vẹn và các trang tự động nạp lại dữ liệu mới nhất.
  - Nếu chọn file hỏng hoặc phiên bản schema không tương thích, nút phục hồi bị vô hiệu hóa kèm thông báo lỗi rõ ràng.

### Kịch bản 9: Phục hồi từ danh sách sao lưu tự động
- **Thao tác:**
  1. Quan sát bảng **Danh sách bản sao lưu tự động** trong phần Sao lưu.
  2. Bấm nút **Phục hồi** tại một bản sao lưu trong danh sách (tối đa lưu 10 bản gần nhất).
  3. Xác nhận phục hồi.
- **Kết quả mong đợi:**
  - Hộp thoại kiểm tra hiển thị thông số tệp.
  - Sau khi xác nhận, dữ liệu được hoàn nguyên chính xác về thời điểm tạo bản sao lưu đó.
