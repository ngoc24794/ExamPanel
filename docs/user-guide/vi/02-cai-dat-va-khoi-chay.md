# Cài Đặt & Khởi Chạy ExamPanel

Tài liệu này hướng dẫn cách cài đặt và khởi chạy ExamPanel trên các hệ điều hành khác nhau, tập trung vào môi trường Windows phổ biến tại các trường trung học phổ thông.

---

## 1. Các Phiên Bản Phân Phối Trên Windows

ExamPanel cung cấp 2 hình thức đóng gói chính thức cho người dùng Windows:

| Phiên Bản | Tên Tệp Phân Phối | Mục Đích Sử Dụng |
| :--- | :--- | :--- |
| **Bản Di Động (Portable ZIP)** | `ExamPanel-0.1.0-windows-x64-portable.zip` | Dành cho Tổ trưởng chuyên môn mang đi dạy trên USB, không cần quyền quản trị máy tính (Administrator), không cần cài đặt. |
| **Bản Cài Đặt (NSIS Installer)** | `ExamPanel-0.1.0-windows-x64-setup.exe` | Dành cho máy tính văn phòng, tự động tạo lối tắt trên Desktop và Start Menu, hỗ trợ đa ngôn ngữ (Tiếng Việt / English). |

---

## 2. Hướng Dẫn Sử Dụng Bản Di Động (Portable USB)

1. Tải tệp `ExamPanel-0.1.0-windows-x64-portable.zip` về máy tính.
2. Nhấp chuột phải vào tệp ZIP và chọn **Extract All...** (Giải nén tất cả) vào thư mục trên máy tính hoặc thẳng vào ổ đĩa USB (ví dụ: `E:\ExamPanel\`).
3. Sau khi giải nén, thư mục chứa các tệp:
   - `ExamPanel.exe`: Tệp chạy chương trình chính.
   - `ExamPanel.portable`: Tệp đánh dấu kích hoạt chế độ di động. Khi có tệp này, ExamPanel sẽ lưu toàn bộ dữ liệu vào thư mục `data/` nằm ngay bên cạnh `ExamPanel.exe`.
   - `DOC-TOI.txt`: Hướng dẫn nhanh cho người dùng.
   - `THIRD_PARTY_NOTICES`: Bản quyền mã nguồn mở.
4. Nhấp đúp vào `ExamPanel.exe` để mở ứng dụng.

> [!TIP]
> **Khóa chống ghi trên USB:** Nếu ổ USB của bạn bị gạt sang nấc khóa chống ghi (Write-protected), ExamPanel sẽ tự động phát hiện và hiển thị hộp thoại cảnh báo thân thiện hỏi Thầy/Cô có muốn tạm thời chuyển sang lưu dữ liệu vào máy tính hay thoát ứng dụng.

---

## 3. Hướng Dẫn Cài Đặt Bản Cài Đặt (NSIS Setup)

1. Tải và nhấp đúp vào tệp `ExamPanel-0.1.0-windows-x64-setup.exe`.
2. Trình cài đặt sẽ hiển thị hộp thoại chọn ngôn ngữ: Chọn **Vietnamese** (Tiếng Việt).
3. Trình cài đặt thực hiện cài đặt theo người dùng hiện tại (`currentUser` trong AppData), không yêu cầu quyền Admin của trường.
4. Sau khi cài đặt hoàn tất, nhấp vào biểu tượng ExamPanel trên màn hình chính (Desktop) để sử dụng.
5. Khi gỡ cài đặt (Uninstall), trình gỡ bỏ mặc định luôn bảo toàn toàn bộ dữ liệu bài toán của nhà trường, trừ khi Thầy/Cô tích chọn xóa trắng dữ liệu.

---

## 4. Xử Lý Cảnh Báo Windows Defender SmartScreen

Khi tải phần mềm mới hoặc chạy lần đầu trên máy tính Windows, Thầy/Cô có thể gặp màn hình cảnh báo màu xanh dương:

> **Windows protected your PC**
> *Microsoft Defender SmartScreen prevented an unrecognized app from starting...*

### Cách khắc phục:
1. Nhấp chuột vào dòng chữ gạch chân: **"More info"** (Thông tin thêm).
2. Nút **"Run anyway"** (Vẫn chạy) sẽ xuất hiện ở góc dưới. Nhấp vào nút này.
3. Phần mềm sẽ khởi động bình thường và Windows sẽ ghi nhớ để không hỏi lại vào những lần sau.

*Lý do:* ExamPanel là phần mềm nội bộ chuyên ngành giáo dục, không chứa chữ ký số thương mại trả phí hàng năm của các tổ chức quốc tế. Ứng dụng hoàn toàn sạch, mã nguồn mở và an toàn 100%.

---

## 5. Yêu Cầu Microsoft Edge WebView2

ExamPanel sử dụng công nghệ WebView2 của Microsoft để kết xuất giao diện đồ họa hiện đại và nhẹ nhàng.
- Trên hầu hết máy tính Windows 10 (bản cập nhật gần đây) và toàn bộ Windows 11, WebView2 đã được Microsoft cài sẵn theo hệ điều hành.
- Nếu mở ứng dụng và nhận được thông báo "Thiếu WebView2", Thầy/Cô tải gói cài đặt chính thức từ Microsoft tại:
  `https://go.microsoft.com/fwlink/p/?LinkId=2124703` (Chạy tệp `MicrosoftEdgeWebview2Setup.exe` một lần duy nhất).

---

## 6. Chạy Trên macOS & Linux

- **Linux (AppImage / .deb):** Nhấp chuột phải vào tệp `.AppImage`, chọn Properties ➔ Permissions ➔ Tích chọn "Allow executing file as program", sau đó nhấp đúp để mở.
- **macOS (.app / .dmg):** Vì phần mềm chưa mua chứng chỉ Apple Developer, macOS Gatekeeper có thể chặn mở lần đầu. Thầy/Cô nhấp chuột phải vào `ExamPanel.app`, giữ phím `Option` và chọn **Open**, sau đó nhấn xác nhận **Open**.
