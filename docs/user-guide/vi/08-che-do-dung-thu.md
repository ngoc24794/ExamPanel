# Chế Độ Dùng Thử Dữ Liệu Mẫu (Trial Mode Sandbox)

Để giúp Thầy/Cô và cán bộ quản lý dễ dàng làm quen, tìm hiểu mọi tính năng của ExamPanel mà không sợ làm ảnh hưởng đến dữ liệu phân công thực tế của trường, ExamPanel tích hợp sẵn **Chế độ dùng thử với dữ liệu mẫu**.

---

## 1. Dữ Liệu Mẫu Bao Gồm Những Gì?

Khi kích hoạt chế độ dùng thử, hệ thống tự động khởi tạo môi trường dữ liệu mẫu hoàn chỉnh:
- **Tổ chuyên môn:** Tổ Toán - Tin học trường phổ thông tiêu chuẩn.
- **Quy mô:** Gồm 2 phân hiệu trường, 18 giáo viên với đầy đủ chuyên môn, hạn ngạch số lần ra đề/phản biện, giáo viên kiêm nhiệm làm đề trắc nghiệm chuyên biệt, và các cặp tránh làm chung ban đề.
- **Kỳ thi & Khối lớp:** 4 kỳ thi chuẩn (Giữa kỳ 1, Cuối kỳ 1, Giữa kỳ 2, Cuối kỳ 2) qua 3 khối lớp 10, 11 và 12.
- **Phương án mẫu:** Bao gồm phương án đã giải sẵn, có điểm số công bằng tối ưu và lịch sử đánh giá chi tiết.

---

## 2. Cách Bật Chế Độ Dùng Thử

1. Mở menu **Cài đặt** (Settings).
2. Tại mục **Chế độ hoạt động**, nhấp vào nút **Dùng thử với dữ liệu mẫu**.
3. Hệ thống sẽ chuyển kết nối sang tệp dữ liệu thử nghiệm độc lập (`data/demo.db`).
4. Khi đang ở chế độ dùng thử:
   - Một dải biểu ngữ màu hổ phách (amber banner) luôn xuất hiện nổi bật trên đầu màn hình: **"Đang ở chế độ dùng thử với dữ liệu mẫu - Dữ liệu thật được bảo vệ an toàn"**.
   - Thầy/Cô có thể tự do thêm, sửa, xóa giáo viên, thay đổi tiêu chí, chạy lại thuật toán phân công tự động hoặc chỉnh sửa thủ công.
   - Mọi thao tác này chỉ lưu vào tệp dữ liệu thử nghiệm `demo.db` và **hoàn toàn không chạm vào dữ liệu thật**.

---

## 3. Thoát Khỏi Chế Độ Dùng Thử

Khi Thầy/Cô đã trải nghiệm xong và muốn quay trở lại làm việc với dữ liệu thật của trường:
1. Nhấp trực tiếp vào nút **Thoát chế độ dùng thử** ngay trên dải biểu ngữ đầu trang (hoặc vào **Cài đặt** ➔ chọn **Quay lại dữ liệu thật**).
2. Hệ thống tự động chuyển kết nối trở lại tệp dữ liệu chính của nhà trường (`exampanel.db`).
3. Toàn bộ màn hình làm việc sẽ được nạp lại tức thì với dữ liệu phân công thực tế.
