# Phase 12 / 12.1 — A6 Status Verification Matrix

This document tracks the completeness and verification status of all A6 requirements from Phase 12 and Phase 12.1 fix-ups.

| STT | Hạng mục (Item) | Trạng thái (Status) | Tệp tin cài đặt (File) | Tên kiểm thử xác minh (Test Name) |
|---|---|---|---|---|
| 1 | Màn hình Giáo viên: Cột / trường "Cách gọi" (display_name) | done | `ui/src/pages/TeachersPage.tsx`, `crates/core/src/domain/entities.rs` | `TeachersPage > renders teacher roster and coverage panel matrix` |
| 2 | Màn hình Giáo viên: Chỉ định hạn ngạch (quota_override) | done | `ui/src/pages/TeachersPage.tsx`, `crates/core/src/domain/entities.rs` | `QPlanGrid Component Tests > renders forced-load teacher indicator and task count markers in totals table` |
| 3 | Màn hình Giáo viên: Giới hạn nhiệm vụ/kỳ (max_tasks_per_exam_override) | done | `ui/src/pages/TeachersPage.tsx`, `crates/core/src/domain/entities.rs` | `comparison_tests > test_task5_comparison_bounds_assertion` |
| 4 | Màn hình Giáo viên: Huy hiệu ghế cố định (forced-seat badge) | done | `ui/src/pages/TeachersPage.tsx` | `TeachersPage > displays coverage panel status badges derived from feasibility` |
| 5 | Màn hình Quy tắc: Bật/tắt H3 (H3 campus balance toggle) | done | `ui/src/pages/RulesPage.tsx`, `crates/core/src/domain/rules.rs` | `RulesPage > renders hard rules section with H3 and H4 toggles and limits` |
| 6 | Màn hình Quy tắc: Giới hạn H4 (H4 max tasks limits) | done | `ui/src/pages/RulesPage.tsx`, `crates/core/src/domain/rules.rs` | `RulesPage > renders hard rules section with H3 and H4 toggles and limits` |
| 7 | Màn hình Quy tắc: Chế độ S1 Tự động/thủ công (S1 Auto/number) | done | `ui/src/pages/RulesPage.tsx`, `crates/core/src/domain/rules.rs` | `RulesPage > navigates to soft rules tab, renders S1/S9/S10 and applies presets` |
| 8 | Màn hình Quy tắc: S9 & S10 (giới hạn nhiệm vụ kỳ & khoảng cách) | done | `ui/src/pages/RulesPage.tsx`, `crates/core/src/domain/rules.rs` | `RulesPage > navigates to soft rules tab, renders S1/S9/S10 and applies presets` |
| 9 | Màn hình Quy tắc: Cấu hình mẫu (Presets bao gồm "Cho phép dồn việc trong một kỳ") | done | `crates/core/src/domain/entities.rs`, `ui/src/pages/RulesPage.tsx` | `RulesPage > navigates to soft rules tab, renders S1/S9/S10 and applies presets` |
| 10 | Bảng Tính khả thi: Mã chẩn đoán mới & nhóm phân công bắt buộc (Forced placement group) | done | `crates/core/src/feasibility/mod.rs`, `ui/src/components/FeasibilitySheet.tsx` | `FeasibilityIndicator & FeasibilitySheet > opens sheet on click, lists diagnostics, and handles "Go to fix" navigation` |
| 11 | Quy trình Hướng dẫn ban đầu: Bước môn học & chuyên môn (Onboarding steps) | done | `ui/src/components/Onboarding.tsx` | `App > renders application brand, sidebar navigation, and ping response` |
| 12 | Tính đồng bộ Mock API: Đầy đủ 100% các phương thức ExamPanelApi | done | `ui/src/lib/api/mock.ts`, `ui/src/lib/api/types.ts` | `lib/api/api.test.ts > implements all ExamPanelApi methods in mock` |
| 13 | Xuất Excel & In ấn: Cột "Môn" trong bảng phân công và thông báo | done | `crates/service/src/excel/export.rs`, `ui/src/pages/print/AssignmentPrintPage.tsx` | `PrintPages > renders assignment print page with Môn column and teacher notices`, `plan_import_tests > test_plan_import_export_roundtrip` |
| 14 | Biểu mẫu nhập giáo viên: Cột "Cách gọi" | done | `crates/service/src/excel/import.rs`, `crates/service/src/excel/template.rs` | `import_tests > test_import_teachers_with_display_name_column` |
| 15 | Cảnh báo tính khả thi: `teacher_no_competency` | done | `crates/core/src/feasibility/mod.rs` | `feasibility_tests > test_diagnostics_teacher_no_competency` |
| 16 | Khôi phục bản sao lưu schema v4 (Restore of a v4 backup with auto pre-restore backup & migration) | done | `crates/storage/src/backup.rs`, `crates/storage/src/repository.rs` | `v4_migration_tests > test_v4_restore_with_auto_pre_restore_backup_and_migration` |
