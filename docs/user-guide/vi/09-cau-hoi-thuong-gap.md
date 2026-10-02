# Câu Hỏi Thường Gặp & Xử Lý Sự Cố (FAQ & Troubleshooting)

---

## 1. Các Câu Hỏi Thường Gặp (FAQ)

### Q1: Tôi có thể chạy ExamPanel trên máy tính không có kết nối Internet không?
**Trả lời:** Hoàn toàn được. ExamPanel là ứng dụng chạy cục bộ trên máy tính (offline desktop application), không gửi dữ liệu ra máy chủ bên ngoài và hoạt động 100% khi không có Internet.

### Q2: Tôi có thể copy thư mục ứng dụng vào USB và mang sang máy tính khác dùng được không?
**Trả lời:** Có. ExamPanel hỗ trợ phiên bản Portable (di động). Trong phiên bản này có tệp đánh dấu `ExamPanel.portable`. Toàn bộ dữ liệu của trường được lưu tại thư mục `data/` ngay cạnh tệp `ExamPanel.exe` trên USB. Thầy/Cô chỉ cần cắm USB vào bất kỳ máy tính Windows nào (Windows 10, Windows 11) là có thể tiếp tục làm việc ngay lập tức.

### Q3: Nếu hai giáo viên có nguyện vọng không làm chung ban đề thì cấu hình ở đâu?
**Trả lời:** Tại mục **Quy tắc phân công** ➔ thẻ **Tránh ghép cặp (Avoid Pairs)**, Thầy/Cô chọn Giáo viên A và Giáo viên B rồi chọn mức độ:
- **Bắt buộc (Hard constraint):** Thuật toán tuyệt đối không bao giờ xếp 2 người vào cùng một ban đề kiểm tra.
- **Ưu tiên (Soft penalty):** Thuật toán hạn chế tối đa việc xếp chung, trừ trường hợp không còn nhân sự nào khác thay thế.

### Q4: Thuật toán có phân công đều việc giữa các học kỳ không?
**Trả lời:** Có. Tiêu chí phân bố tải theo thời gian (temporal load spreading) là một trong những tiêu chuẩn tối ưu quan trọng của ExamPanel, giúp tránh tình trạng một giáo viên phải gánh nhiệm vụ dồn dập trong 2 kỳ thi liên tiếp.

---

## 2. Xử Lý Sự Cố (Troubleshooting)

### Sự cố 1: Windows hiển thị cảnh báo "Windows protected your PC" (SmartScreen)
- **Nguyên nhân:** Do ExamPanel là phần mềm nội bộ chuyên ngành giáo dục, phát hành dưới dạng mã nguồn mở và chưa mua chứng chỉ số thương mại đắt tiền từ Microsoft (EV Code Signing Certificate).
- **Cách xử lý:** Thầy/Cô nhấp chuột vào dòng chữ nhỏ **"More info"** (hoặc **"Thông tin khác"**), sau đó chọn nút **"Run anyway"** (hoặc **"Vẫn chạy"**). Ứng dụng sẽ khởi chạy bình thường.

### Sự cố 2: Thông báo lỗi "Cần cài đặt Microsoft Edge WebView2 Runtime"
- **Nguyên nhân:** ExamPanel sử dụng công nghệ WebView2 chuẩn của Microsoft để hiển thị giao diện đồ họa. Hầu hết Windows 10 (bản cập nhật mới) và Windows 11 đã có sẵn. Một số máy Windows cũ có thể chưa có.
- **Cách xử lý:** 
  1. Nếu máy tính có Internet, bộ cài đặt NSIS của ExamPanel sẽ tự động tải và cài đặt trong 1 phút.
  2. Nếu dùng bản Portable trên máy tính không có Internet, Thầy/Cô tải gói cài đặt chính thức của Microsoft tại: `https://go.microsoft.com/fwlink/p/?LinkId=2124703` (Microsoft Edge WebView2 Evergreen Standalone Installer), chép vào USB và cài đặt trước khi chạy ExamPanel.

### Sự cố 3: Ổ đĩa USB bị khóa chống ghi (Read-Only)
- **Nguyên nhân:** USB bị gạt nút khóa chống ghi vật lý hoặc do chính sách bảo mật máy tính trường học cấm ghi USB.
- **Cách xử lý:** Khi khởi chạy bản di động trên USB bị khóa, ExamPanel sẽ hiển thị thông báo tiếng Việt rõ ràng:
  - Chọn **"Chuyển sang lưu trên máy tính"**: ExamPanel sẽ tạm thời lưu dữ liệu vào thư mục cá nhân trên máy tính (`%APPDATA%`).
  - Chọn **"Thoát"**: Thầy/Cô tháo USB, kiểm tra lại lẫy khóa chống ghi và cắm lại.

### Sự cố 4: Tra cứu tệp nhật ký hoạt động (Logs) khi gặp lỗi
Khi cần gửi nhật ký hoạt động cho ban kỹ thuật hỗ trợ:
1. Mở ExamPanel ➔ vào menu **Cài đặt** (Settings).
2. Cuộn xuống mục **Nhật ký hoạt động (Logs)** ➔ nhấp nút **Mở thư mục nhật ký**.
3. Thư mục chứa các tệp nhật ký `exampanel.log` sẽ tự động mở ra trên Windows Explorer.
4. Thầy/Cô có thể đính kèm tệp này để kỹ thuật viên phân tích và hỗ trợ nhanh chóng nhất.
