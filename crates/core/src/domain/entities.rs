//! Domain entities, enums, and configuration constants.

use super::ids::{CampusId, ExamId, GradeId, LockId, PlanId, SchoolYearId, TeacherId};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// A physical school branch or campus location.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Campus {
    pub id: CampusId,
    pub code: String,
    pub name: String,
    pub color: String,
}

/// A student cohort grade level (e.g. 10, 11, 12).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Grade {
    pub id: GradeId,
    pub code: i32,
    pub name: String,
    pub sort_order: i32,
}

/// A teaching staff member eligible for assignment.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Teacher {
    pub id: TeacherId,
    pub full_name: String,
    pub campus_id: CampusId,
    pub load_weight: f64,
    pub active: bool,
    pub note: Option<String>,
}

/// An academic year under management (e.g., "2026-2027").
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SchoolYear {
    pub id: SchoolYearId,
    pub name: String,
    pub is_current: bool,
}

/// An exam term during a school year (e.g., GK1, CK1, GK2, CK2).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Exam {
    pub id: ExamId,
    pub school_year_id: SchoolYearId,
    pub code: String,
    pub name: String,
    pub sort_order: i32,
}

/// Grade qualifications taught by a teacher in a specific school year.
/// Note: Grades taught can change every school year.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TeacherGrade {
    pub teacher_id: TeacherId,
    pub school_year_id: SchoolYearId,
    pub grade_id: GradeId,
}

/// An exam term for which a teacher is unavailable.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Unavailability {
    pub teacher_id: TeacherId,
    pub exam_id: ExamId,
    pub reason: Option<String>,
}

/// Role on an exam panel: Setter (Ra đề) or Reviewer (Phản biện).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    Setter,
    Reviewer,
}

impl Role {
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Setter => "setter",
            Self::Reviewer => "reviewer",
        }
    }
}

impl fmt::Display for Role {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for Role {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "setter" => Ok(Self::Setter),
            "reviewer" => Ok(Self::Reviewer),
            other => Err(format!("unknown role: {other}")),
        }
    }
}

/// Lock override kind: mandatory PIN or prohibited FORBID.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LockKind {
    Pin,
    Forbid,
}

impl LockKind {
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Pin => "pin",
            Self::Forbid => "forbid",
        }
    }
}

impl fmt::Display for LockKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for LockKind {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "pin" => Ok(Self::Pin),
            "forbid" => Ok(Self::Forbid),
            other => Err(format!("unknown lock kind: {other}")),
        }
    }
}

/// A manual lock override on an exam panel.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Lock {
    pub id: LockId,
    pub exam_id: ExamId,
    pub grade_id: GradeId,
    pub teacher_id: TeacherId,
    pub role: Option<Role>,
    pub kind: LockKind,
}

/// Constraint rule key identifying hard constraint parameters and soft constraint objectives.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RuleKey {
    #[serde(rename = "h4", alias = "h4_single_panel_per_exam")]
    H4,
    #[serde(rename = "h7", alias = "h7_workload_quota")]
    H7,
    #[serde(rename = "s1", alias = "s1_reviewer_frequency")]
    S1,
    #[serde(rename = "s2", alias = "s2_role_ratio_balance")]
    S2,
    #[serde(rename = "s3", alias = "s3_campus_reviewer_independence")]
    S3,
    #[serde(rename = "s4", alias = "s4_setter_pair_diversity")]
    S4,
    #[serde(rename = "s5", alias = "s5_reciprocal_review_avoidance")]
    S5,
    #[serde(rename = "s6", alias = "s6_consecutive_exam_relief")]
    S6,
    #[serde(rename = "s7", alias = "s7_multi_grade_rotation")]
    S7,
}

impl RuleKey {
    /// All 9 recognized rule keys.
    pub const ALL: [Self; 9] = [
        Self::H4,
        Self::H7,
        Self::S1,
        Self::S2,
        Self::S3,
        Self::S4,
        Self::S5,
        Self::S6,
        Self::S7,
    ];

    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::H4 => "h4",
            Self::H7 => "h7",
            Self::S1 => "s1",
            Self::S2 => "s2",
            Self::S3 => "s3",
            Self::S4 => "s4",
            Self::S5 => "s5",
            Self::S6 => "s6",
            Self::S7 => "s7",
        }
    }
}

impl fmt::Display for RuleKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for RuleKey {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "h4" | "h4_single_panel_per_exam" => Ok(Self::H4),
            "h7" | "h7_workload_quota" => Ok(Self::H7),
            "s1" | "s1_reviewer_frequency" => Ok(Self::S1),
            "s2" | "s2_role_ratio_balance" => Ok(Self::S2),
            "s3" | "s3_campus_reviewer_independence" => Ok(Self::S3),
            "s4" | "s4_setter_pair_diversity" => Ok(Self::S4),
            "s5" | "s5_reciprocal_review_avoidance" => Ok(Self::S5),
            "s6" | "s6_consecutive_exam_relief" => Ok(Self::S6),
            "s7" | "s7_multi_grade_rotation" => Ok(Self::S7),
            other => Err(format!("unknown rule key: {other}")),
        }
    }
}

/// Configuration settings for a specific constraint rule.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RuleSetting {
    pub key: RuleKey,
    pub enabled: bool,
    pub weight: f64,
    pub params: serde_json::Value,
}

impl RuleSetting {
    /// Returns canonical default rule settings used across the application.
    #[must_use]
    pub fn default_settings() -> Vec<Self> {
        vec![
            Self {
                key: RuleKey::H4,
                enabled: true,
                weight: 100.0,
                params: serde_json::json!({}),
            },
            Self {
                key: RuleKey::H7,
                enabled: true,
                weight: 100.0,
                params: serde_json::json!({ "tolerance": 1 }),
            },
            Self {
                key: RuleKey::S1,
                enabled: true,
                weight: 10.0,
                params: serde_json::json!({ "min": 1, "max": 2 }),
            },
            Self {
                key: RuleKey::S2,
                enabled: true,
                weight: 5.0,
                params: serde_json::json!({ "ratio": [2, 1] }),
            },
            Self {
                key: RuleKey::S3,
                enabled: true,
                weight: 4.0,
                params: serde_json::json!({}),
            },
            Self {
                key: RuleKey::S4,
                enabled: true,
                weight: 6.0,
                params: serde_json::json!({}),
            },
            Self {
                key: RuleKey::S5,
                enabled: true,
                weight: 6.0,
                params: serde_json::json!({}),
            },
            Self {
                key: RuleKey::S6,
                enabled: true,
                weight: 2.0,
                params: serde_json::json!({}),
            },
            Self {
                key: RuleKey::S7,
                enabled: true,
                weight: 1.0,
                params: serde_json::json!({}),
            },
        ]
    }
}

/// A full generated schedule across all panels for a school year.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Plan {
    pub id: PlanId,
    pub school_year_id: SchoolYearId,
    pub name: String,
    pub created_at: String,
    pub seed: u64,
    pub score: Option<f64>,
    pub is_final: bool,
}

/// An individual assignment record within a Plan.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Assignment {
    pub plan_id: PlanId,
    pub exam_id: ExamId,
    pub grade_id: GradeId,
    pub teacher_id: TeacherId,
    pub role: Role,
}

/// Identifying coordinate for an Exam Panel (Exam × Grade).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PanelKey {
    pub exam_id: ExamId,
    pub grade_id: GradeId,
}

impl PanelKey {
    #[inline]
    #[must_use]
    pub const fn new(exam_id: ExamId, grade_id: GradeId) -> Self {
        Self { exam_id, grade_id }
    }
}

/// Panel composition rules and structural constants.
pub struct PanelComposition;

impl PanelComposition {
    /// Number of setters required per panel.
    pub const SETTERS: usize = 2;
    /// Number of reviewers required per panel.
    pub const REVIEWERS: usize = 1;
    /// Total assigned teachers per panel.
    pub const TOTAL_PER_PANEL: usize = Self::SETTERS + Self::REVIEWERS;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_role_string_and_serde() {
        assert_eq!(Role::Setter.as_str(), "setter");
        assert_eq!(Role::Reviewer.as_str(), "reviewer");
        assert_eq!("setter".parse::<Role>().unwrap(), Role::Setter);
        assert_eq!("reviewer".parse::<Role>().unwrap(), Role::Reviewer);
        assert_eq!(serde_json::to_string(&Role::Setter).unwrap(), "\"setter\"");
        assert_eq!(
            serde_json::from_str::<Role>("\"reviewer\"").unwrap(),
            Role::Reviewer
        );
    }

    #[test]
    fn test_lock_kind_string_and_serde() {
        assert_eq!(LockKind::Pin.as_str(), "pin");
        assert_eq!(LockKind::Forbid.as_str(), "forbid");
        assert_eq!("pin".parse::<LockKind>().unwrap(), LockKind::Pin);
        assert_eq!("forbid".parse::<LockKind>().unwrap(), LockKind::Forbid);
        assert_eq!(serde_json::to_string(&LockKind::Pin).unwrap(), "\"pin\"");
    }

    #[test]
    fn test_rule_key_strings_and_serde() {
        for key in RuleKey::ALL {
            let s = key.as_str();
            assert_eq!(s.parse::<RuleKey>().unwrap(), key);
            let json = serde_json::to_string(&key).unwrap();
            assert_eq!(json, format!("\"{s}\""));
            let parsed: RuleKey = serde_json::from_str(&json).unwrap();
            assert_eq!(parsed, key);
        }
        // Alias test
        assert_eq!(
            serde_json::from_str::<RuleKey>("\"h4_single_panel_per_exam\"").unwrap(),
            RuleKey::H4
        );
        assert_eq!(
            "s1_reviewer_frequency".parse::<RuleKey>().unwrap(),
            RuleKey::S1
        );
    }

    #[test]
    fn test_panel_composition_constants() {
        assert_eq!(PanelComposition::SETTERS, 2);
        assert_eq!(PanelComposition::REVIEWERS, 1);
        assert_eq!(PanelComposition::TOTAL_PER_PANEL, 3);
    }
}
