//! ExamPanel Tauri Application Shell
//!
//! Exposes IPC commands from Rust to the React frontend.
//! Calls into `exam_panel_core` and `exam_panel_storage`.

pub mod commands;

/// Application identifier as a constant for easy modification.
pub const APP_IDENTIFIER: &str = "vn.exampanel.app";

/// Display name of the application.
pub const APP_NAME: &str = "ExamPanel";

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![commands::ping])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ping_returns_expected_format() {
        let msg = commands::ping();
        assert!(msg.starts_with("pong from ExamPanel"));
    }
}
