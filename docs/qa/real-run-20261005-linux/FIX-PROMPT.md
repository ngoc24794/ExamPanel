# Prompt: fix all findings from the real-app QA run (RA-001..RA-043)

Copy everything below the line into a new session.

---

Nhiệm vụ: sửa toàn bộ lỗi đã tìm thấy trong đợt QA ứng dụng thật của ExamPanel (43 finding, RA-001..RA-043). Trả lời bằng tiếng Việt.

## Bối cảnh
- Repo `ngoc24794/ExamPanel`. Nguồn sự thật là thư mục `docs/qa/real-run-20261005-linux/` trên nhánh `qa/real-app-run-linux` (PR #1). Đọc trước: `REPORT.md` (mục 7 "Prioritised fix backlog" kèm test hồi quy đề xuất, mục 6 giới hạn Linux-vs-Windows), `findings.md`/`findings.json` (repro, expected/actual, evidence, suspected cause — các suspected cause chỉ là GIẢ THUYẾT, phải tự xác minh bằng code và test trước khi sửa), `matrix.md`.
- Bằng chứng nằm ở `extracts/<ID>.json`, `screenshots/`, `logs/`. Harness chạy app thật: `e2e-real/` và `scripts/qa/` (xem `README.md` trong thư mục QA để biết cách dựng stack Xvfb + tauri-driver).
- Đọc `AGENTS.md`, `docs/SPEC.md`, `docs/ARCHITECTURE.md`, `docs/DECISIONS.md` trước khi sửa.

## Quy trình
1. Tạo nhánh mới `fix/qa-real-run-linux` từ `qa/real-app-run-linux` (hoặc từ `main` nếu PR #1 đã merge). KHÔNG push lên nhánh khác; không tạo tag; không merge.
2. Chia việc theo nhóm, mỗi nhóm một hoặc vài commit Conventional Commits (`fix:`, `test:`, `docs:`), theo thứ tự ưu tiên dưới đây. Với MỖI finding: (a) viết test hồi quy tái hiện lỗi và xác nhận nó FAIL trước khi sửa, (b) sửa tối thiểu, (c) xác nhận test PASS, (d) với lỗi chỉ thấy ở app thật, chạy lại scenario tương ứng trong `e2e-real/` và lưu output vào `docs/reports/qa-fix/<ID>/` bằng redirect shell.
3. Ghi lại trong `docs/qa/real-run-20261005-linux/FIX-STATUS.md` một bảng: ID, trạng thái (fixed / wontfix / needs-decision / not-reproducible), commit, test hồi quy, bằng chứng. Mọi con số trong báo cáo phải do tool in ra từ lần chạy thật (Reporting Integrity Rule trong AGENTS.md); không gõ tay kết quả.
4. Trước khi báo hoàn thành chạy `pnpm check-all` (cargo fmt, clippy -D warnings, cargo test, ui lint/typecheck/test/build) và dán kết quả. Cập nhật `docs/DECISIONS.md` khi có quyết định kiến trúc.
5. Quy tắc cứng của repo: `crates/core` không phụ thuộc tauri/rusqlite/IO; thay đổi schema chỉ qua migration SQL tăng dần trong `crates/storage/migrations/`; chuỗi UI luôn qua `t('domain.key')` và cập nhật CẢ `ui/src/i18n/locales/vi.json` và `en.json`; màu chỉ dùng token ngữ nghĩa; mọi API mới phải cập nhật `ExamPanelApi` (`types.ts`), `mock.ts`, `tauri.ts`. Không đổi trạng thái máy toàn cục (cài toolchain, biến môi trường người dùng, `~/.cargo/config.toml`).
6. Không bao giờ bỏ qua, tắt hay làm yếu test để pass. Nếu một finding cần quyết định sản phẩm (ví dụ RA-017, RA-029, RA-037), dừng và hỏi người dùng thay vì tự chọn; ghi `needs-decision`.
7. Không tạo PR trừ khi người dùng yêu cầu. Chỉ push lên `fix/qa-real-run-linux` khi người dùng cho phép.

## Thứ tự ưu tiên (ID — vấn đề ngắn gọn; chi tiết đầy đủ trong findings.md)

**Nhóm 1 — S1/S2 gây mất dữ liệu hoặc sai kết quả**
- RA-001 (S1): hộp thoại lỗi khởi động chạy trong `setup()` trên main thread, deadlock trên Linux (cửa sổ trắng). Chuyển sang cơ chế không chặn (hiện dialog sau khi event loop chạy, hoặc dùng dialog async/emit lỗi lên UI).
- RA-019: Δ-score ứng viên/thay thế tính cho NHẦM vị trí người ra đề (thứ tự setter của `eval_edit.rs` khác vị trí trong DB).
- RA-020: điểm tổng và "Hợp lệ" không cập nhật sau khi sửa (replace/kéo-đổi); kéo-đổi không hợp lệ vẫn được nhận.
- RA-011: Hủy tối ưu vẫn lưu phương án dở và hiện đồng thời hai thông báo.
- RA-031: xoay vòng backup tự động và sắp xếp "mới nhất trước" theo TÊN file (tiền tố reason đứng trước timestamp) → xóa nhầm backup mới. Sắp xếp theo timestamp/mtime.
- RA-007: mở DB v4 migrate tại chỗ không có backup trước khi migrate.
- RA-036: bản release vẫn bật feature `dev-tools`, lệnh `seed_demo` ghi dữ liệu mẫu vào DB thật. Tắt feature khỏi release (xem `src-tauri/Cargo.toml`, `default = ["dev-tools"]`).
- RA-022: cờ "Dữ liệu đã đổi" (stale) không hiện sau khi đổi dữ liệu cho đến khi reload.
- RA-018: sau "Tạo bản chỉnh sửa" bản sao không được chọn.
- RA-033: ngôn ngữ chọn ở trang Cài đặt không được lưu.
- RA-034: "Mở thư mục dữ liệu" lỗi ACL (`opener:default` thiếu quyền `open_path`; sửa trong `src-tauri/capabilities/default.json`, giữ phạm vi quyền hẹp nhất).

**Nhóm 2 — S2 xuất/hiển thị sai (mô hình hai môn)**
- RA-013 compare dialog mù môn; RA-027 heatmap thống kê mù môn + cắt còn 11 giáo viên; RA-028 sheet Excel "Theo giáo viên" cột "Cùng ban với" lẫn ban của cả hai môn; RA-009 sheet "Tiêu chí" sai tên tiêu chí S3–S8, cắt số vi phạm S8.
- RA-014: lưới kiểu Q bị cắt ở 1280x720.

**Nhóm 3 — S3**
- RA-012 thanh tiến độ/điểm tốt nhất đi lùi; RA-017 một campus giả làm S3 cộng hằng 144 điểm; RA-023 H2 `unqualified_grade` thừa khi đánh dấu giáo viên vắng; RA-024 "Invalid Date" (WebKitGTK không parse `YYYY-MM-DD HH:MM:SS`); RA-026 tối ưu lại quanh ghế giữ lại không giữ thứ tự Đề 1/Đề 2; RA-029 chữ tổ chức bịa trong in/xuất; RA-030 In/Xuất PDF không làm gì trên Linux; RA-043 `app.log` luôn rỗng; RA-039 điều hướng bàn phím lưới (60 tab stop vô hình); RA-015 lưới Q lệch bản giấy.
- i18n: RA-002, RA-010, RA-021, RA-032, RA-040 (10 key thiếu cả hai locale), RA-042 (~20 chuỗi tiếng Việt cứng trong chế độ English). Thêm test tự động kiểm tra mọi key `t()` đều có trong cả `vi.json` và `en.json`.
- Tài liệu: RA-037 (SPEC §6/ADR mâu thuẫn với code về chế độ portable), RA-038 (checklist lỗi thời), RA-005 (môi trường test mock; chạy suite làm đổi 94 file đã track — làm cho suite không ghi vào file track).

**Nhóm 4 — S4**
- RA-003/RA-004/RA-006/RA-008/RA-016/RA-025/RA-035/RA-041.

## Điều kiện hoàn thành
- Mọi finding có trạng thái trong `FIX-STATUS.md`; các mục fixed đều có test hồi quy từng FAIL trước khi sửa.
- `pnpm check-all` xanh; CI trên GitHub xanh (kể cả Windows).
- Với lỗi chỉ có ở app thật, đã chạy lại scenario `e2e-real/` tương ứng trên bản build mới, và lưu output.
- Những gì KHÔNG xác minh được trên Linux (WebView2, installer Windows, hộp thoại file Windows, in thật) được liệt kê rõ là "not verified" kèm lý do, và dẫn tới `windows-manual-checklist.md`.
- Báo cáo cuối: số lượng fixed/wontfix/needs-decision/not-reproducible (lấy từ `FIX-STATUS.md`), các câu hỏi cần Q quyết định, và rủi ro hồi quy còn lại.
