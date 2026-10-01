//! ExamPanel Core Library
//!
//! Pure domain models, feasibility checks, and solver algorithms for ExamPanel.
//! This crate does NOT depend on Tauri or SQLite.

pub mod domain;
pub mod feasibility;
pub mod optimize;
pub mod score;
pub mod solver;
pub mod validate;

pub use feasibility::*;
pub use optimize::*;
pub use score::*;
pub use solver::*;
pub use validate::*;

/// Returns core engine version information.
#[must_use]
pub fn core_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_core_version_returns_package_version() {
        assert_eq!(core_version(), env!("CARGO_PKG_VERSION"));
    }
}
