# Changelog

All notable changes to ExamPanel will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [0.1.0] - 2026-10-02

### Tiếng Việt (Vietnamese)

#### Tính năng mới (Added)
- **Thuật toán xếp lịch và tối ưu hóa phân công:**
  - Bộ giải ràng buộc cứng (H1–H7) bằng luồng cực đại (Dinic) và quay lui MRV có xét đến phân hiệu, khối lớp, hạn ngạch công việc và lịch vắng.
  - Tối ưu hóa đa mục tiêu bằng giải thuật Tôi luyện thép (Simulated Annealing) song song trên nhiều luồng CPU với hạt giống giả ngẫu nhiên xác định (PRNG ChaCha8).
  - Đánh giá thỏa mãn 8 tiêu chí mềm (S1–S8: tần suất phản biện, cân bằng vai trò ra đề/phản biện, tính độc lập phân hiệu, đa dạng cặp ra đề, đa dạng quan hệ duyệt chéo, giảm tải thi liên tiếp, xoay vòng khối lớp, cân bằng tải trọng số).
  - Xuất ra 3 phương án xếp lịch tối ưu có độ đa dạng khác biệt $\ge 20\%$.
- **Không gian làm việc & chỉnh sửa phương án:**
  - Hiển thị ma trận phân công 4 kỳ thi × 3 khối lớp trực quan theo bảng màu phân hiệu chuẩn WCAG AA.
  - Chế độ chỉnh sửa thủ công cho phép hoán đổi vai trò, thay thế giáo viên bằng bàn phím, tính điểm biến thiên tức thời ($\Delta\text{Score}$), hoàn tác/làm lại (Undo/Redo), và ghim vị trí để tối ưu hóa cục bộ.
  - So sánh trực quan sự khác biệt giữa hai phương án.
  - Báo cáo thống kê phân bổ khối lượng công việc, tỷ lệ ra đề/phản biện của từng giáo viên.
- **Nhập / Xuất dữ liệu & In ấn:**
  - Trợ lý nhập liệu từ tệp Excel chuẩn với các chế độ Hợp nhất (Merge) hoặc Đồng bộ (Sync), tự động phát hiện mã giáo viên, kiểm tra ràng buộc trước khi áp dụng.
  - Xuất toàn bộ phương án ra sổ tính Excel đa trang có định dạng chuyên nghiệp.
  - Chức năng in lịch phân công toàn trường và in phiếu thông báo phân công cá nhân gửi từng giáo viên kèm chữ ký.
- **Sao lưu & Phục hồi:**
  - Sao lưu trực tuyến SQLite với cơ chế xoay vòng tự động tối đa 10 bản sao lưu an toàn.
  - Kiểm tra tính toàn vẹn tệp sao lưu trước khi phục hồi, tự động tạo bản sao lưu dự phòng trước khi khôi phục, tự động nạp lại kết nối cơ sở dữ liệu và làm mới bộ nhớ đệm ứng dụng.
- **Kiến trúc & Vận hành:**
  - Chế độ chạy di động (Portable mode) kích hoạt thông qua tệp đánh dấu `ExamPanel.portable` bên cạnh tệp thực thi, lưu dữ liệu trực tiếp trong thư mục `data/` trên USB; tự động dự phòng thư mục AppData hệ điều hành khi chạy từ thư mục cài đặt.
  - Giao diện người dùng đa ngôn ngữ 100% (Tiếng Việt mặc định, Tiếng Anh tùy chọn), chế độ sáng/tối/theo hệ thống.
  - Kiểm tra tính nhất quán phiên bản tự động giữa Cargo và cấu hình ứng dụng.

---

### English

#### Added
- **Optimization & Scheduling Core Engine:**
  - Exact constructive solver for H1–H7 hard constraints using Dinic max-flow and MRV backtracking respecting campus diversity, grade qualifications, availability quotas, and pin/forbid locks.
  - Parallel multi-run Simulated Annealing local search with deterministic ChaCha8 PRNG seeding.
  - Scoring and evaluation of 8 soft rules (S1–S8: reviewer frequency, role ratio balance, campus reviewer independence, repeated setter pair diversity, reciprocal review diversity, consecutive setter relief, multi-grade rotation, and discrete load equity) with provable lower bounds.
  - Multi-plan output generating up to 3 candidate schedules satisfying a minimum 20% pairwise diversity threshold.
- **Workspace & Interactive Plan Editing:**
  - Matrix view of 4 exams × 3 grades with accessible semantic campus tokens.
  - Manual draft editing supporting candidate swap/replace, sub-millisecond delta score re-evaluation, multi-level undo/redo, slot pinning, and re-optimization.
  - Pairwise plan comparison modal highlighting differential assignments.
  - Faculty workload distribution statistics and setter vs. reviewer balance charts.
- **Data Import, Export & Printing:**
  - Excel template generator and import wizard with preview simulation and validation checks.
  - Multi-sheet Excel workbook export with professional typography and formatting.
  - Native print and PDF generation for the institutional assignment master table and individual teacher assignment notices with administrative signatures.
- **Safety, Backup & Recovery:**
  - SQLite online backup API with automatic pruning retaining up to 10 timestamped safety snapshots.
  - Pre-restore backup creation, backup file validation, and automatic connection reopening with UI query cache invalidation.
- **Runtime & Deployment:**
  - Portable USB runtime mode activated via `ExamPanel.portable` marker file next to the binary, falling back to OS app-data directories when marker is absent.
  - 100% internationalization (Vietnamese default, English secondary) with dark and light theme tokens.
  - Single source of truth versioning anchored in workspace `Cargo.toml`.
