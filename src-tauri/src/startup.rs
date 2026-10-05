//! Startup sequence of the desktop shell.
//!
//! The checks that may need a modal dialog (read-only portable folder, database newer than
//! the app, damaged database) run on a dedicated thread. `blocking_show()` must never run
//! on the event-loop thread: inside `setup()` it parks the loop that has to service the
//! dialog request, which left Linux users with a blank window forever (RA-001). The main
//! window is therefore created only after the checks passed (`"create": false` in
//! `tauri.conf.json`) so the frontend can never call a command before the service exists.

use exam_panel_service::error::AppError;
use exam_panel_service::service::AppService;
use exam_panel_storage::paths::DataLocationStatus;
use std::path::Path;
use std::sync::Arc;
use tauri::{AppHandle, Manager};
use tauri_plugin_dialog::{DialogExt, MessageDialogButtons};

/// What went wrong while opening the database at startup.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OpenFailure {
    NewerDatabase { version: String },
    DamagedDatabase { detail: String },
    Other { message: String },
}

/// Maps the service error returned by `AppService::open_default` to a user-facing failure kind.
#[must_use]
pub fn classify_open_error(err: &AppError) -> OpenFailure {
    match err.code.as_str() {
        "unsupported_database_version" => OpenFailure::NewerDatabase {
            version: err
                .params
                .get("version")
                .map(param_text)
                .unwrap_or_default(),
        },
        "database_corrupted" | "database_error" => OpenFailure::DamagedDatabase {
            detail: err.params.get("detail").map(param_text).unwrap_or_default(),
        },
        _ => OpenFailure::Other {
            message: err.to_string(),
        },
    }
}

fn param_text(v: &serde_json::Value) -> String {
    v.as_str().map_or_else(|| v.to_string(), str::to_string)
}

#[must_use]
pub fn read_only_message(requested: &Path, fallback: &Path) -> String {
    format!(
        "Chế độ di động (ExamPanel.portable) được kích hoạt nhưng thư mục không có quyền ghi dữ liệu:\n{}\n\nBạn có muốn chuyển sang sử dụng thư mục dữ liệu trên máy tính ({}) không?",
        requested.display(),
        fallback.display()
    )
}

#[must_use]
pub fn newer_database_message(version: &str) -> String {
    format!(
        "Cơ sở dữ liệu có phiên bản mới hơn (v{version}) không tương thích với phiên bản ExamPanel này.\n\nVui lòng cập nhật ExamPanel lên phiên bản mới nhất."
    )
}

#[must_use]
pub fn damaged_database_message(detail: &str) -> String {
    format!(
        "Cơ sở dữ liệu bị lỗi hoặc hư hỏng:\n{detail}\n\nBạn có muốn tự động khôi phục từ bản sao lưu gần nhất không?"
    )
}

fn confirm(app: &AppHandle, title: &str, message: String) -> bool {
    app.dialog()
        .message(message)
        .title(title)
        .buttons(MessageDialogButtons::OkCancel)
        .blocking_show()
}

fn notify(app: &AppHandle, title: &str, message: String) {
    let _ = app.dialog().message(message).title(title).blocking_show();
}

/// Starts the startup sequence on its own thread; returns immediately so the event loop keeps
/// running and can service the dialogs.
pub fn spawn(app: AppHandle) {
    let spawned = std::thread::Builder::new()
        .name("exampanel-startup".into())
        .spawn({
            let app = app.clone();
            move || {
                if let Err(code) = run(&app) {
                    log::error!("startup aborted with exit code {code}");
                    app.exit(code);
                }
            }
        });
    if let Err(e) = spawned {
        log::error!("cannot spawn startup thread: {e}");
        app.exit(1);
    }
}

/// Runs the checks, opens the service, then creates the main window. `Err(code)` = exit code.
fn run(app: &AppHandle) -> Result<(), i32> {
    #[cfg(target_os = "windows")]
    if tauri::webview_version().is_err() {
        notify(
            app,
            "ExamPanel - Thiếu WebView2",
            "Không tìm thấy Microsoft Edge WebView2 Runtime trên hệ thống.\nỨng dụng ExamPanel cần WebView2 để hiển thị giao diện.\n\nVui lòng cài đặt WebView2 Runtime từ Microsoft rồi khởi động lại ứng dụng."
                .to_string(),
        );
        return Err(1);
    }

    if let DataLocationStatus::PortableReadOnly {
        requested,
        fallback,
    } = exam_panel_storage::paths::inspect_data_location()
    {
        log::warn!(
            "portable folder {} is read-only; offering fallback {}",
            requested.display(),
            fallback.display()
        );
        if !confirm(
            app,
            "ExamPanel - Lỗi ghi dữ liệu di động",
            read_only_message(&requested, &fallback),
        ) {
            return Err(0);
        }
    }

    let service = match AppService::open_default() {
        Ok(s) => s,
        Err(err) => {
            log::error!("cannot open database: {err}");
            recover_or_exit(app, &err)?
        }
    };

    if let Ok(info) = service.get_app_info() {
        log::info!(
            "ExamPanel {} started (mode {}, data dir {}, database {})",
            info.version,
            info.mode.as_deref().unwrap_or("?"),
            info.data_dir,
            info.db_path
        );
    }
    app.manage(Arc::new(service));
    create_main_window(app)
}

/// Dialogs for a failed open; returns a service when the user restored a backup successfully.
fn recover_or_exit(app: &AppHandle, err: &AppError) -> Result<AppService, i32> {
    match classify_open_error(err) {
        OpenFailure::NewerDatabase { version } => {
            notify(
                app,
                "ExamPanel - Phiên bản không tương thích",
                newer_database_message(&version),
            );
            Err(1)
        }
        OpenFailure::DamagedDatabase { detail } => {
            if !confirm(
                app,
                "ExamPanel - Lỗi cơ sở dữ liệu",
                damaged_database_message(&detail),
            ) {
                return Err(1);
            }
            let Some(backup) = exam_panel_storage::backup::list_automatic_backups()
                .ok()
                .and_then(|list| list.into_iter().next())
            else {
                notify(
                    app,
                    "ExamPanel - Không có bản sao lưu",
                    "Không tìm thấy bản sao lưu nào để khôi phục.".to_string(),
                );
                return Err(1);
            };
            let db_path = exam_panel_storage::paths::resolve_database_path();
            if let Err(restore_err) = std::fs::copy(&backup.path, &db_path) {
                log::error!("restore from {} failed: {restore_err}", backup.path);
                notify(
                    app,
                    "ExamPanel - Lỗi khôi phục",
                    format!("Khôi phục từ bản sao lưu thất bại: {restore_err}"),
                );
                return Err(1);
            }
            log::info!("restored database from {}", backup.path);
            AppService::open_default().map_err(|retry_err| {
                log::error!("cannot open restored database: {retry_err}");
                notify(
                    app,
                    "ExamPanel - Lỗi khởi động",
                    format!("Không thể mở cơ sở dữ liệu sau khi khôi phục: {retry_err}"),
                );
                1
            })
        }
        OpenFailure::Other { message } => {
            notify(
                app,
                "ExamPanel - Lỗi khởi động",
                format!("Lỗi khởi tạo cơ sở dữ liệu: {message}"),
            );
            Err(1)
        }
    }
}

/// Creates the main window from `tauri.conf.json` on the event-loop thread.
fn create_main_window(app: &AppHandle) -> Result<(), i32> {
    let handle = app.clone();
    app.run_on_main_thread(move || {
        let Some(config) = handle.config().app.windows.first().cloned() else {
            log::error!("no window configured in tauri.conf.json");
            handle.exit(1);
            return;
        };
        if let Err(e) = tauri::WebviewWindowBuilder::from_config(&handle, &config)
            .and_then(tauri::WebviewWindowBuilder::build)
        {
            log::error!("cannot create the main window: {e}");
            handle.exit(1);
        }
    })
    .map_err(|e| {
        log::error!("cannot schedule window creation: {e}");
        1
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn newer_database_is_classified_with_its_version() {
        let err = AppError::new("unsupported_database_version").with_param("version", "99");
        assert_eq!(
            classify_open_error(&err),
            OpenFailure::NewerDatabase {
                version: "99".into()
            }
        );
        assert!(newer_database_message("99").contains("v99"));
    }

    #[test]
    fn damaged_database_offers_restore() {
        for code in ["database_corrupted", "database_error"] {
            let err = AppError::new(code).with_param("detail", "malformed");
            assert_eq!(
                classify_open_error(&err),
                OpenFailure::DamagedDatabase {
                    detail: "malformed".into()
                }
            );
        }
        assert!(damaged_database_message("x").contains("khôi phục"));
    }

    #[test]
    fn other_errors_fall_through() {
        let err = AppError::new("io_error");
        assert!(matches!(
            classify_open_error(&err),
            OpenFailure::Other { .. }
        ));
    }

    /// The main window must not exist before the startup thread has opened the service.
    #[test]
    fn main_window_is_created_by_the_startup_sequence_only() {
        let conf: serde_json::Value =
            serde_json::from_str(include_str!("../tauri.conf.json")).unwrap();
        assert_eq!(
            conf["app"]["windows"][0]["create"],
            serde_json::json!(false)
        );
    }

    /// RA-001: no dialog may be shown with `blocking_show` from `setup()` (event-loop thread).
    #[test]
    fn setup_does_not_block_the_event_loop_with_dialogs() {
        let lib = include_str!("lib.rs");
        let setup = lib
            .split(".setup(")
            .nth(1)
            .and_then(|s| s.split(".invoke_handler(").next())
            .expect("setup closure");
        assert!(
            !setup.contains("blocking_show"),
            "dialogs in setup() deadlock the main thread"
        );
    }
}
