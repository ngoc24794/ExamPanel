# Quy trình Kiểm thử Thủ công (Manual QA Checklist) - Phase 8

Tài liệu này cung cấp các bước kiểm thử tuần tự trên ứng dụng Desktop thực tế (`pnpm tauri dev`) sử dụng dữ liệu mẫu (demo seed).

---

## Chuẩn bị môi trường
1. Khởi chạy ứng dụng:
   ```powershell
   pnpm tauri dev
   ```
2. Vào mục **Cài đặt** (`/settings`):
   - Đảm bảo dữ liệu demo đã được nạp (nhấn **Nạp dữ liệu mẫu** nếu cần).
   - Kiểm tra chuyển đổi giao diện Sáng / Tối và ngôn ngữ Tiếng Việt / English.

---

## Các kịch bản kiểm thử (Numbered QA Steps)

### Bước 1: Chạy Tối ưu hóa Phân công (Run Optimizer)
- **Thao tác:**
  1. Vào menu **Phân công** (`/assignments`).
  2. Nhấp nút **Chạy tối ưu** ở góc trên bên phải.
  3. Chọn mức độ nỗ lực: **Nhanh** (hoặc Chuẩn/Kỹ). Số lượng phương án: 3.
  4. Nhấp nút **Bắt đầu tối ưu**.
- **Kết quả mong đợi:**
  - Hộp thoại hiển thị thanh tiến trình (% hoàn thành), số bước lặp, điểm số tốt nhất hiện tại và thời gian đã trôi qua.
  - Sau khi hoàn thành, 3 phương án được lưu vào cơ sở dữ liệu dưới dạng bản nháp (`optimizer`). Hộp thoại đóng và mở ma trận phương án đầu tiên.

### Bước 2: Hủy tiến trình tối ưu (Cancel Optimizer)
- **Thao tác:**
  1. Mở lại hộp thoại **Chạy tối ưu**, chọn mức **Kỹ** (nhiều vòng lặp hơn).
  2. Bấm **Bắt đầu tối ưu**.
  3. Khi tiến trình đang chạy, bấm nút **Hủy bỏ**.
- **Kết quả mong đợi:**
  - Tiến trình dừng ngay lập tức.
  - Thông báo hiển thị "Đã hủy tiến trình tối ưu". Trạng thái trả về bình thường, không làm treo ứng dụng.

### Bước 3: So sánh hai phương án (Compare Plans)
- **Thao tác:**
  1. Nhấp nút **So sánh phương án** trên thanh công cụ.
  2. Chọn Phương án A (Kế hoạch #1) và Phương án B (Kế hoạch #2).
- **Kết quả mong đợi:**
  - Hộp thoại hiển thị khoảng cách giải pháp (Distance), bảng đối sánh chi tiết từng luật mềm giữa 2 phương án.
  - Ma trận so sánh trực quan tô màu nổi bật các ô có sự thay đổi giáo viên giữa 2 phương án.

### Bước 4: Nhân bản để chỉnh sửa thủ công (Duplicate & Edit)
- **Thao tác:**
  1. Trên thanh thông tin phương án của một phương án máy tạo (`optimizer`), nhấp nút **Tạo bản chỉnh sửa**.
  2. Không có hộp thoại đặt tên: bản sao được tạo ngay với tên `<tên phương án gốc> (Chỉnh sửa)` và tự động được chọn.
- **Kết quả mong đợi:**
  - Bản sao được tạo với nguồn gốc `duplicate`.
  - Chế độ chỉnh sửa được kích hoạt: xuất hiện các nút **Hoàn tác (Undo)**, **Làm lại (Redo)**, **Hủy thay đổi (Discard)** và **Lưu (Save)**.

### Bước 5: Đổi chỗ hai vị trí bằng kéo thả (Drag & Drop Swap)
- **Thao tác:**
  1. Kéo một thẻ giáo viên (ví dụ: Ra đề 1 ở ô Giữa kỳ 1 - Khối 10) thả vào một thẻ giáo viên khác trên ma trận.
- **Kết quả mong đợi:**
  - Hai giáo viên hoán đổi vị trí cho nhau.
  - Điểm số tổng và phân rã các vi phạm cập nhật tức thời (live feedback).
  - Nút **Hoàn tác** và **Lưu** sáng lên.

### Bước 6: Thay thế giáo viên bằng bàn phím (Keyboard Replace)
- **Thao tác:**
  1. Di chuột vào một ô phân công, bấm menu dấu ba chấm (hoặc phím Enter/Click) chọn **Thay thế bằng người khác**.
  2. Danh sách ứng viên mở ra, sắp xếp theo mức độ cải thiện điểm số (Δ Score).
  3. Dùng phím mũi tên `↓` / `↑` để di chuyển qua các ứng viên.
  4. Quan sát các ứng viên không hợp lệ bị làm mờ (disabled) kèm lý do vi phạm ràng buộc cứng.
  5. Chọn một ứng viên hợp lệ và nhấn **Enter**.
- **Kết quả mong đợi:**
  - Giáo viên mới thay thế vị trí đã chọn.
  - Điểm phạt cập nhật theo delta đã tính toán.

### Bước 7: Thử nghiệm Hoàn tác và Làm lại (Undo / Redo)
- **Thao tác:**
  1. Nhấn nút **Hoàn tác** (hoặc tổ hợp phím `Ctrl+Z`).
  2. Nhấn nút **Làm lại** (hoặc tổ hợp phím `Ctrl+Y`).
- **Kết quả mong đợi:**
  - Trạng thái các ô phân công quay lại chính xác bước trước và sau khi thao tác.

### Bước 8: Ghim và Cấm trực tiếp từ ma trận (Pin / Forbid from Matrix)
- **Thao tác:**
  1. Mở menu ngữ cảnh tại một vị trí giáo viên trên ma trận.
  2. Chọn **Ghim người này ở ô này**.
  3. Mở menu ở một vị trí khác và chọn **Cấm người này ở ô này**.
- **Kết quả mong đợi:**
  - Vị trí được ghim xuất hiện biểu tượng Ghim (Pin).
  - Khóa được lưu vào bảng quản lý khóa (`locks`) của năm học.

### Bước 9: Giữ ô và Tối ưu lại phần còn lại (Re-optimize Around Kept Slots)
- **Thao tác:**
  1. Mở menu tại 2 ô phân công hài lòng và chọn **Giữ ô này khi tối ưu lại**.
  2. Biểu ngữ "Đã chọn giữ: 2 ô" xuất hiện phía trên bảng (cả ở chế độ Bảng tổ lẫn Chi tiết).
  3. Bấm **Tối ưu lại phần còn lại**.
  4. Chọn số phương án và bấm **Bắt đầu**.
- **Kết quả mong đợi:**
  - Thuật toán chạy tối ưu hóa với các ô đã giữ được cố định tạm thời.
  - Các phương án mới sinh ra vẫn giữ nguyên giáo viên tại 2 ô đã chọn, trong khi các ô còn lại được sắp xếp tối ưu hơn.

### Bước 10: Xác nhận chính thức phương án (Mark Final)
- **Thao tác:**
  1. Mở panel **Lịch sử phương án**.
  2. Chọn một phương án hợp lệ (0 vi phạm cứng) và bấm menu **Đặt làm chính thức**.
  3. Thử đặt làm chính thức một phương án có vi phạm cứng (nếu có).
- **Kết quả mong đợi:**
  - Phương án hợp lệ nhận huy hiệu **Chính thức (Final)** màu xanh lá.
  - Phương án không hợp lệ hoặc dữ liệu bài toán bị thay đổi sẽ báo lỗi chi tiết, không cho phép chốt chính thức.

### Bước 11: Kiểm tra cảnh báo lỗi thời (Stale Badge)
- **Thao tác:**
  1. Vào menu **Kỳ thi** (`/exams`) hoặc **Giáo viên** (`/teachers`), thực hiện thay đổi dữ liệu (ví dụ: sửa tên hoặc thêm kỳ thi/phân hiệu).
  2. Quay lại menu **Phân công** (`/assignments`).
- **Kết quả mong đợi:**
  - Phương án hiển thị huy hiệu **Dữ liệu đã đổi (Stale)** màu vàng cam.
  - Xuất hiện banner cảnh báo với danh sách các vi phạm cứng mới (nếu có) do dữ liệu bài toán thay đổi so với `problem_hash` lúc tạo phương án.

### Bước 12: Báo cáo & Thống kê chuyên sâu (Statistics)
- **Thao tác:**
  1. Vào menu **Thống kê** (`/statistics`).
  2. Chọn phương án vừa phân công ở menu lựa chọn phương án.
- **Kết quả mong đợi:**
  - **Biểu đồ khối lượng:** Hiển thị thanh so sánh giữa số lượt thực tế và chỉ tiêu định mức $q$ của từng giáo viên.
  - **Bảng chi tiết theo giáo viên:** Hiển thị số lượt ra đề, phản biện, độ lệch tải, khối và kỳ tham gia; hỗ trợ sắp xếp theo cột khi nhấn vào tiêu đề.
  - **Bản đồ đồng hành (Co-working Matrix):** Hiển thị tần suất làm việc chung giữa từng cặp giáo viên.
  - **Bản đồ phản biện (Review-Relation Matrix):** Hiển thị số lượt phản biện lẫn nhau (Người phản biện → Người ra đề).
  - **Cơ cấu phân hiệu:** Thống kê tỷ lệ ban đề có 2 phân hiệu hoặc 3 phân hiệu kết hợp.
