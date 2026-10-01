//! Tauri command handlers for ExamPanel.

use crate::APP_NAME;

/// Ping command to verify Rust-frontend IPC bridge.
#[tauri::command]
pub fn ping() -> String {
    format!(
        "pong from {} core v{}",
        APP_NAME,
        exam_panel_core::core_version()
    )
}
