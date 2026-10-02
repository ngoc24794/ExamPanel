//! ExamPanel Service Layer
//!
//! Provides the application service interface decoupling presentation (Tauri/IPC)
//! from persistence and domain solvers.

pub mod dto;
pub mod error;
pub mod service;

pub use dto::*;
pub use error::AppError;
pub use service::AppService;
