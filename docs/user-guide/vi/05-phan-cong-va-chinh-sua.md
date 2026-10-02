# Không Gian Phân Công & Chỉnh Sửa Kế Hoạch

Không gian **Phân công** (Assignments) là trái tim của phần mềm ExamPanel, nơi Thầy/Cô sinh phương án tự động, so sánh các phương án và tinh chỉnh thủ công ma trận phân công.

---

## 1. Sinh Phương Án Bằng Thuật Toán Tối Ưu

1. Truy cập mục **Phân công** trên thanh điều hướng.
2. Nhấp vào nút **Chạy tối ưu** (Run Optimizer).
3. Hộp thoại cài đặt hiển thị các tùy chọn:
   - **Số lượng phương án:** Từ 1 đến 5 phương án khác nhau. ExamPanel sẽ tự động lọc các phương án có sự khác biệt về cấu trúc để Thầy/Cô có nhiều lựa chọn.
   - **Mức độ nỗ lực (Optimization Effort):**
     - *Nhanh (Fast):* 50.000 lượt duyệt / 4 lần chạy ngẫu nhiên (~0.5 - 1 giây).
     - *Chuẩn (Standard - Khuyến nghị):* 200.000 lượt duyệt / 8 lần chạy ngẫu nhiên (~1 - 2 giây).
     - *Kỹ (Thorough):* 500.000 lượt duyệt / 16 lần chạy ngẫu nhiên (~3 - 5 giây).
   - **Tùy chọn nâng cao:** Có thể chỉ định hạt giống ngẫu nhiên (Base Seed) để tái hiện lại kết quả chính xác 100%.
4. Nhấn **Bắt đầu tối ưu**. Quá trình chạy hiển thị tiến trình động và điểm phạt tốt nhất theo thời gian thực.

---

## 2. Quản Lý Danh Sách Phương Án

Sau khi hoàn tất, các phương án được lưu trữ an toàn trong cơ sở dữ liệu:
- **Tên phương án:** Mặc định được đặt là `Phương án #1`, `Phương án #2`... Thầy/Cô có thể nhấn **Đổi tên** (Rename) bất kỳ lúc nào.
- **Điểm phạt (Score) & Xếp hạng (Rank):** Phương án có điểm phạt thấp nhất sẽ được xếp hạng #1 (tốt nhất).
- **Cận dưới provable (Provable Lower Bound):** ExamPanel hiển thị ngưỡng cận dưới toán học. Nếu phương án đạt tới cận dưới (hiển thị nhãn xanh *"Tối ưu toàn cục"*), điều đó chứng minh về mặt toán học rằng không thể có bất kỳ phương án nào tốt hơn.
- **Nguồn gốc (Source):** Đánh dấu phương án do máy tạo (Optimizer), do người dùng chỉnh sửa tay (Manual), hay bản sao chép (Duplicate).
- **Trạng thái lỗi thời (Stale):** Nếu sau khi tạo phương án, Thầy/Cô vào sửa dữ liệu giáo viên hoặc lịch bận, hệ thống sẽ cảnh báo phương án bị lỗi thời và tính toán lại điểm phạt theo dữ liệu mới nhất.

---

## 3. Xem Ma Trận & Thẻ Giáo Viên

Màn hình chi tiết phương án hiển thị toàn bộ ma trận phân công của năm học:
- **Hàng:** Các kỳ thi (GK1, CK1, GK2, CK2) và các khối lớp (10, 11, 12).
- **Cột:** 2 vị trí Ra đề (Setter 1, Setter 2) và 1 vị trí Phản biện (Reviewer).
- **Thẻ giáo viên (Teacher Chip):** Hiển thị rõ họ tên, mã phân hiệu và mã màu sắc trực quan.
- **Lưới tương tác nhanh:** Nhấp vào bất kỳ thẻ giáo viên nào, hệ thống sẽ tự động làm nổi bật toàn bộ các vị trí mà Thầy/Cô đó được phân công trong cả năm học, giúp quan sát ngay phân bố khối lượng.

---

## 4. Cơ Chế Chỉnh Sửa Thủ Công An Toàn (Manual Edit)

Để đảm bảo tính toàn vẹn dữ liệu, các phương án do máy tạo ra có tính chất **bất biến (Immutable)**. Khi muốn thay đổi người ở bất kỳ vị trí nào:

1. Nhấn nút **Tạo bản sao chỉnh sửa** (Duplicate for Edit). Hệ thống tạo một bản sao độc lập mang nhãn *Chỉnh tay*.
2. Nhấp vào ô (vị trí) Thầy/Cô muốn thay đổi.
3. Hộp thoại **Chọn giáo viên thay thế** sẽ mở ra:
   - Danh sách giáo viên đủ điều kiện được sắp xếp tự động theo mức độ tối ưu (được tính toán trước delta điểm phạt).
   - Giáo viên nào nếu đưa vào sẽ gây vi phạm ràng buộc cứng (ví dụ: đang bận, không dạy khối này, hoặc trùng phân hiệu) sẽ bị khóa hoặc cảnh báo vi phạm rõ ràng.
4. Chọn giáo viên mong muốn và nhấn **Thay thế**.
5. Nhấn **Lưu thay đổi** (Save) để ghi lại phương án. Hệ thống hỗ trợ đầy đủ tính năng **Hoàn tác (Undo)** và **Làm lại (Redo)** nhiều bước.

---

## 5. Chốt Phương Án Chính Thức (Mark Final)

Khi đã lựa chọn được phương án ưng ý nhất:
1. Nhấn nút **Đánh dấu chính thức** (Mark as Final).
2. Phương án này sẽ nhận huy hiệu màu xanh **Chính thức**. Toàn bộ các báo cáo thống kê, lệnh in ấn và tệp xuất Excel sẽ mặc định lấy số liệu từ phương án chính thức này.
