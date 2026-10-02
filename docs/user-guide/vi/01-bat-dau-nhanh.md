# Hướng Dẫn Nhanh (Quick Start) — ExamPanel v0.1.0

Chào mừng Thầy/Cô và Quý nhà trường đến với **ExamPanel** — giải pháp phần mềm chuyên dụng hỗ trợ lập kế hoạch và tối ưu hóa phân công cán bộ ra đề và phản biện đề kiểm tra định kỳ tại các trường trung học phổ thông.

---

## 1. Quy Trình 5 Bước Cơ Bản

```
[1. Nhập Dữ Liệu] ➔ [2. Kiểm Tra Khả Thi] ➔ [3. Tối Ưu Tự Động] ➔ [4. Xem & Chỉnh Sửa] ➔ [5. Xuất Excel & In Ấn]
```

1. **Bước 1: Chuẩn bị & Nhập danh sách giáo viên**
   - Vào mục **Cài đặt** (Settings) để nhập tên Trường và Tổ chuyên môn.
   - Tải biểu mẫu Excel mẫu tại trang **Giáo viên** ➔ Nhấn nút **Tải biểu mẫu Excel**.
   - Điền danh sách giáo viên, phân hiệu trực thuộc, các khối lớp giảng dạy và hệ số tải trọng (1.0 là đủ tải, 0.5 là nửa tải, 0.0 là tạm nghỉ).
   - Nhấn **Nhập Excel** để nạp dữ liệu vào hệ thống.

2. **Bước 2: Khai báo kỳ thi & Lịch bận giáo viên**
   - Vào mục **Kỳ thi** để kiểm tra 4 kỳ thi mặc định trong năm học (GK1, CK1, GK2, CK2) hoặc thêm mới theo kế hoạch năm học.
   - Vào mục **Lịch bận** để đánh dấu những giáo viên bận đột xuất hoặc nghỉ sinh/nghỉ công tác trong các kỳ thi cụ thể.

3. **Bước 3: Kiểm tra tính khả thi**
   - Quan sát đèn báo **Tính khả thi** trên thanh công cụ góc trên bên phải màn hình:
     - **Màu xanh (Khả thi):** Dữ liệu hoàn toàn hợp lệ, số lượng giáo viên và năng lực đáp ứng tốt bài toán.
     - **Màu vàng (Có cảnh báo):** Hệ thống phát hiện cảnh báo nghẽn (ví dụ: một phân hiệu có quá ít giáo viên), nhưng vẫn có thể giải được.
     - **Màu đỏ (Không khả thi):** Thiếu giáo viên trầm trọng hoặc có xung đột khóa cấm/ghim. Nhấn vào đèn báo để xem chi tiết hướng dẫn khắc phục.

4. **Bước 4: Chạy tối ưu hóa phân công**
   - Chuyển sang mục **Phân công** (Assignments).
   - Nhấn nút **Chạy tối ưu**. Chọn mức độ nỗ lực (Nhanh, Chuẩn, hoặc Kỹ) và số phương án mong muốn (1 đến 5 phương án).
   - Hệ thống áp dụng thuật toán tối ưu toán học (Random Walk + Simulated Annealing) để tính toán cân bằng tải, phân phối công bằng số lượt ra đề/phản biện, tránh trùng lặp đối tác và kiểm soát tối đa các tiêu chí chuyên môn.

5. **Bước 5: Xem xét, Tinh chỉnh & Xuất tài liệu**
   - Chọn phương án tốt nhất trong bảng danh sách.
   - Xem ma trận phân công chi tiết. Nếu cần đổi người tại bất kỳ vị trí nào: Nhấn **Tạo bản sao chỉnh sửa**, chọn ô cần đổi và chọn giáo viên gợi ý từ danh sách ứng viên hợp lệ.
   - Nhấn **Đánh dấu chính thức** khi phương án đã hoàn thiện.
   - Xuất dữ liệu:
     - Nhấn **Xuất Excel** để lưu bảng phân công ma trận và bảng tổng hợp công việc gửi Ban Giám hiệu.
     - Nhấn **In bảng phân công** để xem trang in văn bản hành chính khổ A4 chuẩn mực.
     - Nhấn **In giấy báo** để in từng phiếu thông báo phân công cá nhân gửi riêng cho từng Thầy/Cô.

---

## 2. Lưu Ý Quan Trọng
- **Bảo mật dữ liệu:** ExamPanel là phần mềm chạy độc lập (offline). Toàn bộ dữ liệu nằm trên máy tính của Quý trường và không bao giờ gửi qua Internet.
- **Bản di động (USB):** Tệp `ExamPanel.portable` cho phép Thầy/Cô mang theo phần mềm trên USB và cắm chạy trên bất kỳ máy tính Windows nào của nhà trường mà không lo mất dữ liệu.
