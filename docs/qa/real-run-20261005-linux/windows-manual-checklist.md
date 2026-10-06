# Windows manual checklist (items the Linux real-app run could NOT cover)

Everything below needs a real Windows 10/11 machine (WebView2, MSVC build, NSIS, Windows common dialogs). Use release artefacts built from the commit under test; use fictitious data only. Where a Linux finding (RA-xxx) suggests a likely Windows defect, the step says what to confirm.

Legend: **Expected** = result that means PASS. Record screenshots + `data/logs/app.log` for any deviation.

## W1 — WebView2 runtime missing (src-tauri/src/lib.rs setup(), `#[cfg(target_os = "windows")]`)
Source: v0.1.0 checklist "Môi trường kiểm thử" (Windows 10 chưa có WebView2), DOC-TOI §4.
1. On a clean Windows 10 VM **without** Edge WebView2 Evergreen (uninstall it from Apps & features, or use a VM image before April 2023 updates) run `ExamPanel.exe` (portable).
2. **Expected:** modal "ExamPanel - Thiếu WebView2" with the Vietnamese text, process exits with code 1 after OK. **Also confirm** (RA-001) that this `blocking_show()` dialog really appears on Windows and does not hang the main thread like it does on Linux.
3. Install via the NSIS installer instead: `webviewInstallMode = downloadBootstrapper` must download/install WebView2 (needs internet) and the app then starts.

## W2 — SmartScreen / unsigned executable
1. Download the portable ZIP in Edge, extract, run `ExamPanel.exe`.
2. **Expected:** "Windows protected your PC" → More info → Run anyway works as DOC-TOI §3 says; no other blocking (Defender quarantine = report).

## W3 — NSIS installer (`ExamPanel_0.1.0_x64-setup.exe`)
1. Run installer **as a standard user** (no admin). **Expected:** no UAC prompt; language selector shows Tiếng Việt (default) and English.
2. **Expected:** default install dir `%LOCALAPPDATA%\Programs\ExamPanel`; desktop + Start-menu shortcuts with the new vector icon; Start-menu launch works.
3. **Data location (docs mismatch RA-037):** installed mode has no `ExamPanel.portable` marker → code (`crates/storage/src/paths.rs`) writes to `%APPDATA%\ExamPanel\data\exam-panel.db`. v0.1.0 checklist claims `%APPDATA%\com.exampanel.desktop\`, and tauri.conf identifier is `vn.exampanel.app`. Record which folders actually exist (`dir %APPDATA%`) and where `.window-state.json` lands.
4. **Uninstall** from Settings → Apps. **Expected:** per checklist an option to keep user data; check what the NSIS uninstaller really offers and whether `%APPDATA%\ExamPanel\data` survives.

## W4 — Portable ZIP on Windows / USB (v0.1.0 checklist §1)
1. Extract `ExamPanel-0.1.0-windows-x64-portable.zip` to a USB stick (NTFS and FAT32 separately).
2. **Expected:** `ExamPanel.portable`, `DOC-TOI.txt`, `THIRD_PARTY_NOTICES`, `ExamPanel-HuongDan-0.1.0.pdf` present; first window within ~1.5 s on a normal PC (Linux sandbox median 1.45 s from process start to rendered UI — not comparable); `data\` created next to the exe containing `exam-panel.db` (NOT `exampanel.db`, RA-006), `logs\app.log`, later `backups\`.
3. Copy the whole folder to another PC; **Expected:** all plans visible unchanged.

## W5 — Read-only USB / write-protected folder (A4 on Linux: BLOCKED by RA-001)
1. Write-protect the stick (hardware switch) or deny Write on the folder for the current user; keep `ExamPanel.portable`.
2. Launch. **Expected:** Vietnamese dialog "ExamPanel - Lỗi ghi dữ liệu di động" with OK/Cancel. OK → data goes to `%APPDATA%\ExamPanel\data` and the app works; Cancel → clean exit (code 0). **If the window stays blank/hung → confirms RA-001 on Windows (S1).**
3. Repeat for a DB with `PRAGMA user_version = 99` (message "Cơ sở dữ liệu có phiên bản mới hơn…") and for a corrupted `exam-panel.db` with an automatic backup present (offer to restore latest backup; restore works).

## W6 — Executable icon and resources
1. Explorer → Properties → Details: product name, version 0.1.0, copyright; Properties → shortcut icon.
2. **Expected:** the new vector icon at 16/32/48/256 px (taskbar, title bar, Alt-Tab, installer, uninstaller entry); no default Tauri icon.

## W7 — Windows common dialogs (all native, tauri-plugin-dialog)
Run each with a destination path that contains spaces and diacritics (e.g. `D:\Tổ Toán\Phân công 2026`): template download (default name `mau-nhap-du-lieu.xlsx`), Excel export (`phan-cong-…xlsx`), backup now (`exampanel-backup-YYYYMMDD-HHmm.db`), restore from file, import Excel / plan Excel file pickers.
**Expected:** correct default name + extension filter, file created at the chosen path, cancel does nothing, Vietnamese path survives round trip (`dir` shows exact name).

## W8 — Display scaling 125 % / 150 % (and 100 %)
At 1920×1080 with 125 % (effective 1536×864) and 150 % (1280×720): open Tổng quan, Giáo viên, Chuyên môn, Quy tắc, Phân công (Bảng tổ), Thống kê, Cài đặt, all dialogs.
**Expected:** no clipped text, no horizontal page scroll; Q grid (RA-014) — whole 6-column grid + totals panel visible without sideways scrolling at 1280×720 effective; window minimum 1024×700 enforced.

## W9 — Real print dialog / Microsoft Print to PDF (RA-030, D2)
1. Phân công → In / Xuất PDF → "In bảng phân công" → button "In / Xuất PDF" (and Ctrl+P) → printer "Microsoft Print to PDF", **Landscape A4**, margins default, "Background graphics" ON.
2. **Expected:** exactly 1 page; Vietnamese diacritics intact; draft plan shows "[BẢN NHÁP]" title and diagonal watermark; final plan shows neither; org info + signer block from Settings (not the invented defaults, RA-029).
3. "In giấy báo phân công": Portrait A4, "Chỉ giáo viên có nhiệm vụ" vs "Tất cả giáo viên". **Expected:** one page per teacher (12 pages for the Q data), table never split across pages, "Thành viên cùng ban đề" lists only the same-subject panel.
4. Repeat with "Background graphics" OFF: confirm table borders/headers still readable in grayscale.
5. Confirm on WebView2 that `window.print()` works at all (it does nothing on WebKitGTK).

## W10 — Paths with spaces and Vietnamese diacritics in the user profile
Create a Windows user `Nguyễn Văn Q` (profile `C:\Users\Nguyễn Văn Q`) and run the **installed** build and a portable copy under `C:\Users\Nguyễn Văn Q\Tài liệu\ExamPanel`.
**Expected:** DB opens, backups/restore/export work, `logs\app.log` written, window state remembered; no mojibake in the data-path display / copy-path.

## W11 — Windows font rendering of Vietnamese
Check stacked diacritics (Ệ, Ặ, Ữ, Ợ, đ/Đ) in grid cells at 100/125/150 %, in the print preview and the printed PDF. **Expected:** no clipped tone marks in the 26 px grid rows, no fallback to a font without Vietnamese coverage.

## W12 — Dates and WebKit-specific items (webkitgtk-tagged findings)
- RA-024: Phân công → Lịch sử phương án shows a real date/time under WebView2 (expected; Linux shows "Invalid Date").
- RA-034: Cài đặt → "Mở thư mục dữ liệu" opens Explorer (expected) — on Linux it fails with an ACL error; capability `opener:default` is platform-independent so **expect the same failure**; "Mở thư mục nhật ký" should open `data\logs`.

## W13 — Single instance, shell integration
Start a second `ExamPanel.exe` while the first is open (same user). **Expected:** first window is restored/focused, second process exits silently (Linux: second exits in 49 ms, focus not verified).

## W14 — Native startup + fonts + keyboard on Windows narrator / high contrast
Optional: Narrator reads the sidebar, dialogs trap focus, high-contrast theme keeps all controls visible (Linux E1 found focus-ring gaps — see findings).

## W15 — Upgrade path (v0.1.0 checklist §7)
Copy a Phase-9 `exam-panel.db` (user_version 4 — fixture `crates/storage/fixtures/v4_synthetic.db` is equivalent) into `data\` of the new build. **Expected:** migration runs, all years/plans visible (verified on Linux A7: user_version 4→5, rows preserved). **Also note RA-007:** no automatic pre-migration backup exists.

---

# Addendum — checks for the fixes made after this run (branch `fix/qa-real-run-linux`)

These fixes were verified on Linux/WebKitGTK only (`docs/reports/qa-fix/`). The items below are **not verified on Windows**; do them on a real Windows machine.

## W16 — Startup dialogs after the RA-001 fix (ADR-0044)
Repeat W1 and W5, and additionally press **OK** in "Lỗi ghi dữ liệu di động" (expected: the main window opens and data is created under `%APPDATA%\ExamPanel\data`), press **Cancel** (expected: process exits with code 0, no window), press OK in the newer-database dialog (expected: exit code 1), and press OK in the damaged-database dialog with a backup present (expected: backup restored, app starts). Confirm no blank window ever appears before a dialog.

## W17 — Native print command (RA-030, ADR-0046)
In "In / Xuất PDF" pages the button now calls the Rust command `print_page` (WebView2 `ShowPrintUI`). Confirm the Windows print dialog opens from the button, that "Microsoft Print to PDF" produces a landscape A4 page for the plan and portrait pages for notices.

## W18 — Folder buttons after RA-034
"Mở thư mục dữ liệu" must open Explorer on `%APPDATA%\ExamPanel\data` (or the portable `data\`) without an ACL error; "Mở thư mục nhật ký" must open `data\logs`, which now contains `app.log` lines (startup, database opened, backups).

## W19 — Release build contains no dev tools (RA-036)
From the webview console of the release build `window.__TAURI_INTERNALS__.invoke('seed_demo')` must reject with `not_supported`; the database must stay unchanged.

## W20 — Backups (RA-031, RA-007)
After several imports/restores `data\backups` keeps the 10 newest files (newest listed first in Settings); opening a v4 database leaves an `exampanel-backup-pre-migration-<epoch>.db` copy.

## W21 — Grid at 125 % / 150 % scaling (RA-014)
At effective 1280×720 the Q grid (all 6 columns) is fully visible without sideways scrolling; the totals panel sits below the grid under 1440 px and to its right at 1440 px and above.
