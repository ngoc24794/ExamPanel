//! Domain entities and models for ExamPanel.
//!
//! Pure mathematical and business domain models representing campuses,
//! teachers, panels, exams, constraints, plans, and the single-year problem snapshot.

pub mod entities;
pub mod forced;
pub mod ids;
pub mod problem;
pub mod quota;

pub use entities::*;
pub use forced::*;
pub use ids::*;
pub use problem::*;
pub use quota::*;
