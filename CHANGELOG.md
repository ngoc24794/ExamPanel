# Changelog

All notable changes to ExamPanel will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [Unreleased]

### Tiếng Việt (Vietnamese)

#### Tính năng mới & Sửa lỗi (Added & Fixed)
- **Bảng phân công kiểu Tổ bộ môn & Xuất/Nhập theo định dạng trường (Phase 12):**
  - Giao diện bảng phân công "Bảng tổ" (`/assignments`) tái hiện định dạng bảng giấy thực tế của Thầy Q: cột khối lớp nhóm theo kỳ thi, dòng Đề / Phản biện, bảng thống kê phân công gắn liền bên phải với các huy hiệu cảnh báo mật độ nhiệm vụ ($\ge 2$ việc, $\ge 3$ việc).
  - Tích hợp đầy đủ thao tác chỉnh sửa tương tác trên Bảng tổ (gợi ý ứng viên xếp hạng theo $\Delta$ điểm, kéo thả hoán đổi, phím tắt Enter/Delete, khóa/cấm nhiệm vụ, cố định chỗ bắt buộc của Thầy Nghĩa).
  - Xuất bảng tính Excel chuẩn mẫu trường với sheet đầu tiên "Bảng phân công (mẫu tổ)" có tiêu đề cơ quan, khung chữ ký, cột đệm và định dạng in khổ A4 ngang đen trắng.
  - Hỗ trợ in trực quan qua route `/print/plan/:id` và xuất PDF tự động bằng Playwright.
  - Wizard nhập bảng phân công có sẵn từ tập tin Excel hoặc vùng dán clipboard TSV, nhận diện cấu trúc tự động, khớp tên giáo viên đa cấp (cách gọi, họ tên, chuẩn hóa NFC/NFD không dấu) và lưu trữ dưới nguồn `'manual'` với xuất xứ `'import'`.
  - Mẫu nhập liệu Excel v2 bổ sung các sheet "Môn", "Môn đảm nhiệm" và các cột "Cách gọi", "Chỉ tiêu riêng", "Số việc tối đa mỗi kỳ", kèm mô phỏng tính khả thi trước khi áp dụng.
  - Cập nhật thuật toán tính cận dưới toán học cho quy tắc S6 trên bài toán đa môn, đạt cận dưới chính xác 2.0 đơn vị trên bộ dữ liệu Thầy Q.
  - Bổ sung bản ghi quyết định kiến trúc ADR-0043.

- **Quy tắc phân công & Tối ưu hóa (Phase 11.1):**
  - Cập nhật công thức chuẩn cho tiêu chí S9 (Tránh dồn lịch thi): $\text{avoidable}_t = \max(0, \text{crowding}_t - \max(0, c_t - m_t))$, loại bỏ trừ phạt không công bằng do giới hạn cơ học số kỳ thi.
  - Tích hợp tiêu chí S10 (Đa dạng môn duyệt phản biện) và chế độ S1 Auto-Max cho ngưỡng phản biện.
  - Tùy biến giới hạn H4 qua hai tham số `max_tasks_per_exam` và `max_setter_per_exam`.
  - Nâng cấp bộ di chuyển cục bộ cho SA (Simulated Annealing) bao gồm role-swap, multi-subject candidate swaps và điều chuẩn nhiệt độ làm lạnh, đảm bảo optimizer đạt tối ưu toàn diện từ điểm xuất phát trắng (Cold Start).
  - Khôi phục bộ kiểm thử toàn vẹn (124 Rust tests), kiểm tra chéo đa môn $\ge 300$ trường hợp, kiểm tra tính bằng nhau giữa cập nhật điểm gia tăng và tính toán toàn phần sau $\ge 10,000$ bước.
- **Giao diện & Trải nghiệm người dùng:**
  - Trang ma trận năng lực chuyên môn (`/competencies`) hỗ trợ 3 trạng thái chuyển đổi (tắt, theo khối dạy, mọi khối), lọc theo môn/khối và thao tác hàng loạt.
  - Cập nhật ma trận phân công hiển thị trực quan theo khối môn ("Đề"/"PB") kèm huy hiệu phân hiệu làm việc.
  - Trang giáo viên bổ sung trường "Cách gọi" (`display_name`), huy hiệu số lượng chỗ cố định và cài đặt nâng cao.
  - Trang thiết lập quy tắc hỗ trợ bật/tắt H3 kèm cảnh báo, cấu hình giới hạn H4, S1 Auto và cấu hình trọng số S9, S10.
  - Hỗ trợ nhập và xuất Excel với cột "Cách gọi", cảnh báo giáo viên chưa gán môn chuyên môn.
  - Tự động di trú cơ sở dữ liệu từ bản sao lưu v4 sang v5 và khôi phục năng lực chuyên môn.
- **Tài liệu & Kiến trúc:**
  - Bổ sung 6 bản ghi quyết định kiến trúc từ ADR-0037 đến ADR-0042.

### English

#### Added & Fixed
- **Q-Style Department Grid & School-Format Export/Import (Phase 12):**
  - "Bảng tổ" view on `/assignments` reproducing Coordinator Q's authentic paper schedule: grade columns grouped under exam blocks, Setter/Reviewer rows, and an attached live workload table with task density badges ($\ge 2, \ge 3$ tasks).
  - Complete interactive editing parity on the Q-style grid (candidate ranking with $\Delta$ score, drag-and-drop swaps, keyboard navigation, pin/forbid overrides, immovable forced seats).
  - Excel export in official school format with first sheet "Bảng phân công (mẫu tổ)" containing department headers, signature blocks, spacer columns, and A4 landscape grayscale formatting.
  - Interactive printing via route `/print/plan/:id` and automated headless PDF generation via Playwright.
  - Existing plan import wizard supporting both Excel workbooks and clipboard TSV text, with structure detection, multi-tier name matching (display name, full name, accent-folded NFC/NFD), and atomic save under `source: 'manual'` with `origin: 'import'`.
  - Excel Template v2 adding sheets `"Môn"` and `"Môn đảm nhiệm"` and teacher columns `"Cách gọi"`, `"Chỉ tiêu riêng"`, and `"Số việc tối đa mỗi kỳ"`, backed by non-destructive feasibility simulation.
  - Multi-subject mathematical lower bound for S6 achieving exactly 2.0 units on the canonical Q dataset.
  - Added architecture decision record ADR-0043.

- **Scheduling Rules & Optimization Core (Phase 11.1):**
  - Corrected S9 avoidable crowding penalty formula: $\text{avoidable}_t = \max(0, \text{crowding}_t - \max(0, c_t - m_t))$, eliminating structural overflow penalties.
  - Integrated S10 review subject diversity penalty and dynamic S1 Auto-Max pacing mode.
  - Configurable H4 limits via `max_tasks_per_exam` and `max_setter_per_exam`.
  - Enhanced Simulated Annealing move operator repertoire (role swaps, multi-subject candidate swaps) and cooling calibration, guaranteeing cold-start plans dominate manual plans across all metrics.
  - Complete test integrity recovery (124 Rust tests), multi-subject exhaustive cross-check on $\ge 300$ instances, and incremental score verification over $\ge 10,000$ accepted moves.
- **UI & Interaction:**
  - New Subject Competencies matrix screen (`/competencies`) with 3-state cycling toggles (`off` -> `taught` -> `any`), filters, and bulk actions.
  - Assignment matrix with per-cell subject blocks ("Đề"/"PB") and campus indicators.
  - Teacher view with "Display Name" (`display_name`), forced placement badges, and advanced settings accordion.
  - Rules view with H3 multi-campus toggle and warning box, H4 configurable limits, S1 Auto, and S9/S10 soft rule cards.
  - Excel import/export template support for Display Name and unassigned competency warnings.
  - Automatic v4-to-v5 database migration upon backup restore with auto-populated competencies.
- **Architecture & Documentation:**
  - Added sequential architecture decision records ADR-0037 through ADR-0042.

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
