//! Application error contract and conversions.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use thiserror::Error;

/// Standard serializable application error contract.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Error, ts_rs::TS)]
#[error("AppError {code}: {params:?}")]
pub struct AppError {
    pub code: String,
    #[serde(default)]
    #[ts(type = "Record<string, unknown>")]
    pub params: BTreeMap<String, serde_json::Value>,
}

impl AppError {
    #[must_use]
    pub fn new(code: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            params: BTreeMap::new(),
        }
    }

    #[must_use]
    pub fn with_param(
        mut self,
        key: impl Into<String>,
        value: impl Into<serde_json::Value>,
    ) -> Self {
        self.params.insert(key.into(), value.into());
        self
    }
}

impl From<exam_panel_storage::StorageError> for AppError {
    fn from(err: exam_panel_storage::StorageError) -> Self {
        match err {
            exam_panel_storage::StorageError::NotFound(msg) => {
                Self::new("not_found").with_param("message", msg)
            }
            exam_panel_storage::StorageError::Constraint(ref detail) => {
                if detail.contains("teacher_in_use") {
                    Self::new("teacher_in_use")
                } else if detail.contains("exam_in_use") {
                    Self::new("exam_in_use")
                } else if detail.contains("grade_in_use") {
                    Self::new("grade_in_use")
                } else if detail.contains("campus_in_use") {
                    Self::new("campus_in_use")
                } else if detail.contains("UNIQUE constraint") || detail.contains("duplicate") {
                    Self::new("duplicate_entry").with_param("detail", detail.clone())
                } else if detail.contains("FOREIGN KEY") {
                    Self::new("foreign_key_violation").with_param("detail", detail.clone())
                } else {
                    Self::new("storage_constraint").with_param("detail", detail.clone())
                }
            }
            exam_panel_storage::StorageError::Sqlite(e) => {
                Self::new("database_error").with_param("detail", e.to_string())
            }
            exam_panel_storage::StorageError::Io(e) => {
                Self::new("io_error").with_param("detail", e.to_string())
            }
            exam_panel_storage::StorageError::Serialization(e) => {
                Self::new("serialization_error").with_param("detail", e.to_string())
            }
            exam_panel_storage::StorageError::Other(msg) => {
                Self::new("unknown_error").with_param("detail", msg)
            }
        }
    }
}

impl From<exam_panel_core::solver::SolveError> for AppError {
    fn from(err: exam_panel_core::solver::SolveError) -> Self {
        match err {
            exam_panel_core::solver::SolveError::Infeasible { report } => {
                Self::new("problem_infeasible").with_param("error_count", report.errors.len())
            }
            exam_panel_core::solver::SolveError::LimitReached { .. } => Self::new("solver_timeout"),
            exam_panel_core::solver::SolveError::Exhausted { .. } => Self::new("solver_exhausted"),
        }
    }
}

impl From<serde_json::Error> for AppError {
    fn from(err: serde_json::Error) -> Self {
        Self::new("serialization_error").with_param("detail", err.to_string())
    }
}
