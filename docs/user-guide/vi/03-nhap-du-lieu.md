# Hướng Dẫn Nhập Dữ Liệu Từ Excel

ExamPanel cung cấp tính năng nhập danh sách giáo viên, phân hiệu và lịch vắng trực tiếp từ bảng tính Excel (.xlsx) với cơ chế đối soát thông minh, kiểm tra lỗi tự động và sao lưu dữ liệu trước khi ghi nhận.

---

## 1. Tải Biểu Mẫu Nhập Liệu Chuẩn

1. Truy cập trang **Giáo viên** (Teachers) từ thanh điều hướng bên trái.
2. Nhấp vào nút **Tải biểu mẫu Excel** (Generate Template).
3. Chọn vị trí lưu tệp (ví dụ: `Mau_Nhap_Lieu_ExamPanel.xlsx`).

Biểu mẫu Excel bao gồm các trang tính (sheet) với định dạng màu sắc rõ ràng và hướng dẫn điền chi tiết:
- Sheet 1: **HUONG_DAN** — Hướng dẫn quy tắc điền cột.
- Sheet 2: **PHAN_HIEU** (Campuses) — Danh sách các cơ sở, phân hiệu trường.
- Sheet 3: **GIAO_VIEN** (Teachers) — Danh sách giáo viên tổ bộ môn.
- Sheet 4: **LICH_VANG** (Unavailabilities) — Các kỳ thi giáo viên bận không thể tham gia.

---

## 2. Chi Tiết Các Cột Trong Tệp Excel

### Bảng Phân Hiệu (PHAN_HIEU)
- **Mã phân hiệu (Code):** Ký hiệu ngắn viết tắt (ví dụ: `PH1`, `PH2`, `CS_A`, `CS_B`). Bắt buộc, không được trùng lặp.
- **Tên phân hiệu (Name):** Tên đầy đủ (ví dụ: `Phân hiệu 1 - Cơ sở chính`, `Phân hiệu 2 - Khu nội trú`).

### Bảng Giáo Viên (GIAO_VIEN)
- **Mã giáo viên (Code):** Mã định danh giáo viên (ví dụ: `GV01`, `GV02` hoặc mã công chức). Khuyến nghị nên có để tránh nhầm lẫn khi trùng họ tên.
- **Họ và tên (Full Name):** Họ và tên đầy đủ của Thầy/Cô. Bắt buộc.
- **Mã phân hiệu (Campus Code):** Phải khớp với mã phân hiệu đã khai báo ở sheet `PHAN_HIEU`.
- **Khối dạy (Grades):** Danh sách các khối lớp giáo viên có thể dạy và đủ năng lực ra đề, cách nhau bằng dấu phẩy (ví dụ: `10, 11` hoặc `10, 11, 12`).
- **Hệ số tải trọng (Load Weight):** Con số từ `0.0` đến `1.0`:
  - `1.0`: Giáo viên giảng dạy đủ số tiết định mức bình thường.
  - `0.5`: Giáo viên dạy nửa định mức hoặc kiêm nhiệm công tác khác.
  - `0.0`: Giáo viên tạm nghỉ cả năm học (nghỉ thai sản, đi học nâng cao).
- **Trạng thái hoạt động (Active):** Nhập `Có` / `1` / `True` (Đang hoạt động) hoặc `Không` / `0` / `False` (Tạm ngưng).

### Bảng Lịch Vắng (LICH_VANG)
- **Giáo viên (Teacher):** Nhập Mã giáo viên hoặc Họ và tên giáo viên.
- **Kỳ thi (Exam):** Mã kỳ thi (ví dụ: `GK1`, `CK1`, `GK2`, `CK2`).
- **Lý do (Reason):** Ghi chú ngắn gọn (ví dụ: `Trùng lịch chấm thi học sinh giỏi`, `Đi công tác`).

---

## 3. Các Chế Độ Nhập Liệu

Khi nhấp vào nút **Nhập dữ liệu** (Import Excel), hệ thống cho phép Thầy/Cô chọn giữa 2 chế độ:

1. **Thêm và Cập nhật (Upsert):**
   - Thêm giáo viên mới vào danh sách.
   - Cập nhật thông tin khối dạy, phân hiệu nếu giáo viên đã tồn tại.
   - **Giữ nguyên** toàn bộ các giáo viên khác trong hệ thống không có trong tệp Excel.
   - *Phù hợp khi:* Bổ sung một vài giáo viên mới hoặc cập nhật một nhóm nhỏ giáo viên.

2. **Đồng bộ danh sách (Sync):**
   - Thêm mới và cập nhật giáo viên có trong tệp Excel.
   - Tự động **chuyển trạng thái tạm ngưng (deactivate)** đối với các giáo viên có trong hệ thống nhưng không có tên trong tệp Excel.
   - *Phù hợp khi:* Đầu năm học mới khi có danh sách biên chế chính thức toàn tổ.

---

## 4. Kiểm Tra Trước Kết Quả (Import Preview)

Trước khi thực hiện bất kỳ thay đổi nào vào cơ sở dữ liệu, ExamPanel luôn hiển thị màn hình **Xem trước kết quả** (Import Preview):
- Phân loại rõ ràng từng dòng: **Mới (New)**, **Cập nhật (Update)**, **Không đổi (Unchanged)**, hoặc **Lỗi (Error)**.
- Đánh dấu chi tiết lỗi nếu phát hiện: mã phân hiệu không tồn tại, định dạng khối lớp không hợp lệ, hệ số tải ngoài khoảng `[0.0, 1.0]`.
- Cảnh báo tính khả thi: Tự động tính toán lại xem dữ liệu mới nhập vào có gây nghẽn bài toán phân công (thiếu người ra đề/phản biện) hay không.
- Nhấn **Xác nhận & Áp dụng** sau khi đã kiểm tra kỹ lưỡng.

> [!NOTE]
> **Tự động sao lưu an toàn:** Hệ thống luôn tự động tạo một bản sao lưu dữ liệu hoàn chỉnh (trong thư mục `data/backups/`) ngay trước thời điểm ghi nhận dữ liệu nhập. Nếu có bất kỳ sự cố nào, Thầy/Cô hoàn toàn có thể khôi phục lại trạng thái cũ ngay lập tức.
