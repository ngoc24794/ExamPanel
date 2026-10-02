# Sao Lưu & Khôi Phục Dữ Liệu

ExamPanel coi trọng tính an toàn tuyệt đối của dữ liệu phân công nhà trường. Ứng dụng cung cấp cả hai cơ chế: sao lưu tự động theo chu kỳ và sao lưu thủ công theo yêu cầu của Thầy/Cô.

---

## 1. Cơ Chế Sao Lưu Tự Động (Automatic Backups)

Mỗi khi Thầy/Cô thực hiện các thao tác quan trọng (nhập dữ liệu mới, chạy phân công tự động, chốt chính thức phương án), ExamPanel tự động tạo một bản sao lưu an toàn:
- **Vị trí lưu trữ:** Nằm trong thư mục `backups/` bên cạnh tệp dữ liệu chính của ứng dụng.
- **Quy tắc xoay vòng (Retention Policy):** Hệ thống tự động duy trì các bản sao lưu gần nhất (mặc định 10 bản) và xóa bỏ các bản sao cũ hơn để tiết kiệm dung lượng ổ đĩa.
- **Định dạng tệp:** `exampanel-backup-YYYYMMDD-HHMMSS.db` kèm theo mã kiểm tra toàn vẹn dữ liệu.

---

## 2. Sao Lưu Thủ Công Xuất Ra Tệp (Manual Export)

Trước khi thực hiện các thay đổi lớn hoặc chuyển dữ liệu sang máy tính khác, Thầy/Cô nên chủ động xuất bản sao lưu:
1. Mở menu **Cài đặt** (Settings) ở góc trái dưới cùng.
2. Chọn mục **Quản lý dữ liệu** ➔ **Sao lưu dữ liệu**.
3. Chọn vị trí an toàn trên máy tính hoặc thẻ nhớ USB để lưu tệp sao lưu (định dạng `.db` hoặc `.bak`).
4. Ứng dụng hiển thị thông báo "Sao lưu dữ liệu thành công" kèm ngày giờ cụ thể.

---

## 3. Khôi Phục Dữ Liệu An Toàn (Safe Restore)

Khi cần khôi phục lại dữ liệu từ một thời điểm trong quá khứ hoặc chuyển dữ liệu từ máy tính khác sang:
1. Mở **Cài đặt** ➔ **Quản lý dữ liệu** ➔ **Khôi phục dữ liệu**.
2. Thầy/Cô có thể:
   - Chọn một trong các bản sao lưu tự động có sẵn trong danh sách hiển thị trên màn hình.
   - Hoặc nhấp **Chọn tệp sao lưu khác...** để duyệt đến tệp `.db` đã lưu trước đó.
3. **Cơ chế bảo vệ trước khi khôi phục:**
   - Hệ thống sẽ tự động tạo một bản sao lưu an toàn ngay tại thời điểm hiện tại trước khi ghi đè, đảm bảo không bao giờ bị mất mát dữ liệu do sơ suất.
   - Hệ thống hiển thị hộp thoại xác nhận cảnh báo các thay đổi chưa lưu.
4. **Tự động làm mới toàn diện:**
   - Sau khi khôi phục xong, ExamPanel tự động đóng và kết nối lại tệp dữ liệu mới.
   - Giao diện người dùng tự động làm mới toàn bộ bộ nhớ đệm (cache), tải lại danh sách năm học, phương án phân công và lịch sử thao tác mà Thầy/Cô không cần phải khởi động lại ứng dụng.
