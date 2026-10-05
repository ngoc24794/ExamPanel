//! Domain entities, enums, and configuration constants.

use super::ids::{CampusId, ExamId, GradeId, LockId, PlanId, SchoolYearId, SubjectId, TeacherId};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// A physical school branch or campus location.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
pub struct Campus {
    pub id: CampusId,
    pub code: String,
    pub name: String,
    pub color: String,
}

/// A student cohort grade level (e.g. 10, 11, 12).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
pub struct Grade {
    pub id: GradeId,
    pub code: i32,
    pub name: String,
    pub sort_order: i32,
}

/// An academic subject taught at the school (e.g. Physics 'VL', Technology 'CN').
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
pub struct Subject {
    pub id: SubjectId,
    pub code: String,
    pub name: String,
    pub color: String,
    pub sort_order: u32,
    pub setters: u8,
    pub reviewers: u8,
    pub min_campuses: u8,
}

/// A teaching staff member eligible for assignment.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ts_rs::TS)]
pub struct Teacher {
    pub id: TeacherId,
    pub full_name: String,
    pub campus_id: CampusId,
    pub load_weight: f64,
    pub active: bool,
    pub note: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub code: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub display_name: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub quota_override: Option<u32>,
    #[serde(default)]
    #[ts(optional)]
    pub max_tasks_per_exam_override: Option<u32>,
}

/// An academic year under management (e.g., "2026-2027").
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
pub struct SchoolYear {
    pub id: SchoolYearId,
    pub name: String,
    pub is_current: bool,
}

/// An exam term during a school year (e.g., GK1, CK1, GK2, CK2).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
pub struct Exam {
    pub id: ExamId,
    pub school_year_id: SchoolYearId,
    pub code: String,
    pub name: String,
    pub sort_order: i32,
}

/// Grade qualifications taught by a teacher in a specific school year.
/// Note: Grades taught can change every school year.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, ts_rs::TS)]
pub struct TeacherGrade {
    pub teacher_id: TeacherId,
    pub school_year_id: SchoolYearId,
    pub grade_id: GradeId,
}

/// An exam term for which a teacher is unavailable.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, ts_rs::TS)]
pub struct Unavailability {
    pub teacher_id: TeacherId,
    pub exam_id: ExamId,
    pub reason: Option<String>,
}

/// Role on an exam panel: Setter (Ra đề) or Reviewer (Phản biện).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ts_rs::TS)]
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

/// Scope of grades a teacher is qualified for in a competency.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
pub enum GradeScope {
    Taught,
    Any,
}

impl GradeScope {
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Taught => "taught",
            Self::Any => "any",
        }
    }
}

impl fmt::Display for GradeScope {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for GradeScope {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "taught" => Ok(Self::Taught),
            "any" => Ok(Self::Any),
            other => Err(format!("unknown grade scope: {other}")),
        }
    }
}

/// Teacher qualification for a specific subject and role with grade scope.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, ts_rs::TS)]
pub struct Competency {
    pub teacher_id: TeacherId,
    pub subject_id: SubjectId,
    pub role: Role,
    pub grade_scope: GradeScope,
}

/// Lock override kind: mandatory PIN or prohibited FORBID.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ts_rs::TS)]
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
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
pub struct Lock {
    pub id: LockId,
    pub exam_id: ExamId,
    pub grade_id: GradeId,
    pub subject_id: SubjectId,
    pub teacher_id: TeacherId,
    pub role: Option<Role>,
    pub kind: LockKind,
}

/// Constraint rule key identifying hard constraint parameters and soft constraint objectives.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RuleKey {
    #[serde(rename = "h1", alias = "h1_panel_composition")]
    H1,
    #[serde(rename = "h2", alias = "h2_grade_qualification")]
    H2,
    #[serde(rename = "h3", alias = "h3_multi_campus_diversity")]
    H3,
    #[serde(rename = "h4", alias = "h4_single_panel_per_exam")]
    H4,
    #[serde(rename = "h5", alias = "h5_exam_availability")]
    H5,
    #[serde(rename = "h6", alias = "h6_lock_compliance")]
    H6,
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
    #[serde(
        rename = "s8",
        alias = "s8_load_balance",
        alias = "s8_workload_balance"
    )]
    S8,
    #[serde(rename = "s9", alias = "s9_exam_crowding")]
    S9,
    #[serde(rename = "s10", alias = "s10_review_subject_missing")]
    S10,
}

impl ts_rs::TS for RuleKey {
    type WithoutGenerics = Self;
    type OptionInnerType = Self;
    fn name(_: &ts_rs::Config) -> String {
        "RuleKey".to_string()
    }
    fn decl(_: &ts_rs::Config) -> String {
        "type RuleKey = \"h1\" | \"h2\" | \"h3\" | \"h4\" | \"h5\" | \"h6\" | \"h7\" | \"s1\" | \"s2\" | \"s3\" | \"s4\" | \"s5\" | \"s6\" | \"s7\" | \"s8\" | \"s9\" | \"s10\";".to_string()
    }
    fn inline(_: &ts_rs::Config) -> String {
        "RuleKey".to_string()
    }
    fn dependencies(_: &ts_rs::Config) -> Vec<ts_rs::Dependency> {
        vec![]
    }
}

impl RuleKey {
    /// All 17 recognized rule keys (H1..H7 and S1..S10).
    pub const ALL: [Self; 17] = [
        Self::H1,
        Self::H2,
        Self::H3,
        Self::H4,
        Self::H5,
        Self::H6,
        Self::H7,
        Self::S1,
        Self::S2,
        Self::S3,
        Self::S4,
        Self::S5,
        Self::S6,
        Self::S7,
        Self::S8,
        Self::S9,
        Self::S10,
    ];

    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::H1 => "h1",
            Self::H2 => "h2",
            Self::H3 => "h3",
            Self::H4 => "h4",
            Self::H5 => "h5",
            Self::H6 => "h6",
            Self::H7 => "h7",
            Self::S1 => "s1",
            Self::S2 => "s2",
            Self::S3 => "s3",
            Self::S4 => "s4",
            Self::S5 => "s5",
            Self::S6 => "s6",
            Self::S7 => "s7",
            Self::S8 => "s8",
            Self::S9 => "s9",
            Self::S10 => "s10",
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
            "h1" | "h1_panel_composition" => Ok(Self::H1),
            "h2" | "h2_grade_qualification" => Ok(Self::H2),
            "h3" | "h3_multi_campus_diversity" => Ok(Self::H3),
            "h4" | "h4_single_panel_per_exam" | "h4_task_limits_per_exam" => Ok(Self::H4),
            "h5" | "h5_exam_availability" => Ok(Self::H5),
            "h6" | "h6_lock_compliance" => Ok(Self::H6),
            "h7" | "h7_workload_quota" => Ok(Self::H7),
            "s1" | "s1_reviewer_frequency" => Ok(Self::S1),
            "s2" | "s2_role_ratio_balance" => Ok(Self::S2),
            "s3" | "s3_campus_reviewer_independence" => Ok(Self::S3),
            "s4" | "s4_setter_pair_diversity" => Ok(Self::S4),
            "s5" | "s5_reciprocal_review_avoidance" => Ok(Self::S5),
            "s6" | "s6_consecutive_exam_relief" => Ok(Self::S6),
            "s7" | "s7_multi_grade_rotation" => Ok(Self::S7),
            "s8" | "s8_load_balance" | "s8_workload_balance" => Ok(Self::S8),
            "s9" | "s9_exam_crowding" => Ok(Self::S9),
            "s10" | "s10_review_subject_missing" => Ok(Self::S10),
            other => Err(format!("unknown rule key: {other}")),
        }
    }
}

/// Cap on reviewer assignments in soft rule S1.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ReviewerCap {
    #[default]
    Auto,
    Fixed(u32),
}

impl ts_rs::TS for ReviewerCap {
    type WithoutGenerics = Self;
    type OptionInnerType = Self;
    fn name(_: &ts_rs::Config) -> String {
        "ReviewerCap".to_string()
    }
    fn decl(_: &ts_rs::Config) -> String {
        "type ReviewerCap = \"auto\" | number | { type: \"auto\" } | { type: \"fixed\", value: number };".to_string()
    }
    fn inline(_: &ts_rs::Config) -> String {
        "ReviewerCap".to_string()
    }
    fn dependencies(_: &ts_rs::Config) -> Vec<ts_rs::Dependency> {
        vec![]
    }
}

impl Serialize for ReviewerCap {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Self::Auto => serializer.serialize_str("auto"),
            Self::Fixed(n) => serializer.serialize_u32(*n),
        }
    }
}

impl<'de> Deserialize<'de> for ReviewerCap {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde_json::Value;
        let v = Value::deserialize(deserializer)?;
        match v {
            Value::String(s) if s.eq_ignore_ascii_case("auto") => Ok(Self::Auto),
            Value::Number(n) => {
                if let Some(u) = n.as_u64() {
                    Ok(Self::Fixed(u as u32))
                } else {
                    Err(serde::de::Error::custom("invalid reviewer cap number"))
                }
            }
            Value::Object(map) => {
                if let Some(t) = map.get("type").and_then(Value::as_str) {
                    if t.eq_ignore_ascii_case("auto") {
                        return Ok(Self::Auto);
                    }
                    if t.eq_ignore_ascii_case("fixed") {
                        if let Some(val) = map.get("value").and_then(Value::as_u64) {
                            return Ok(Self::Fixed(val as u32));
                        }
                    }
                }
                Err(serde::de::Error::custom("invalid reviewer cap object"))
            }
            _ => Err(serde::de::Error::custom(
                "expected auto or number for reviewer cap",
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
pub struct H3Params {
    pub enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
pub struct H4Params {
    pub max_tasks_per_exam: u32,
    pub max_setter_per_exam: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
pub struct H7Params {
    pub tolerance: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
pub struct S1Params {
    #[serde(default = "default_s1_min")]
    pub min: u32,
    #[serde(default)]
    pub max: ReviewerCap,
}

const fn default_s1_min() -> u32 {
    1
}

/// Configuration settings for a specific constraint rule.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ts_rs::TS)]
pub struct RuleSetting {
    pub key: RuleKey,
    pub enabled: bool,
    pub weight: f64,
    #[ts(type = "Record<string, unknown>")]
    pub params: serde_json::Value,
}

impl RuleSetting {
    /// Returns canonical default rule settings used across the application.
    #[must_use]
    pub fn default_settings() -> Vec<Self> {
        vec![
            Self {
                key: RuleKey::H3,
                enabled: true,
                weight: 100.0,
                params: serde_json::json!({ "enabled": true }),
            },
            Self {
                key: RuleKey::H4,
                enabled: true,
                weight: 100.0,
                params: serde_json::json!({ "max_tasks_per_exam": 2, "max_setter_per_exam": 1 }),
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
                params: serde_json::json!({ "min": 1, "max": "auto" }),
            },
            Self {
                key: RuleKey::S2,
                enabled: true,
                weight: 3.0,
                params: serde_json::json!({}),
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
            Self {
                key: RuleKey::S8,
                enabled: true,
                weight: 8.0,
                params: serde_json::json!({}),
            },
            Self {
                key: RuleKey::S9,
                enabled: true,
                weight: 5.0,
                params: serde_json::json!({}),
            },
            Self {
                key: RuleKey::S10,
                enabled: true,
                weight: 4.0,
                params: serde_json::json!({}),
            },
        ]
    }
}

/// Canonical rule weight presets for soft constraint prioritization.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RulePreset {
    Balanced,
    WorkloadFairness,
    TeamDiversity,
    AllowTaskCrowding,
}

impl RulePreset {
    #[must_use]
    pub fn all() -> [Self; 4] {
        [
            Self::Balanced,
            Self::WorkloadFairness,
            Self::TeamDiversity,
            Self::AllowTaskCrowding,
        ]
    }

    #[must_use]
    pub fn id(&self) -> &'static str {
        match self {
            Self::Balanced => "balanced",
            Self::WorkloadFairness => "workload_fairness",
            Self::TeamDiversity => "team_diversity",
            Self::AllowTaskCrowding => "allow_task_crowding",
        }
    }

    #[must_use]
    pub fn settings(&self) -> Vec<RuleSetting> {
        match self {
            Self::Balanced => RuleSetting::default_settings(),
            Self::WorkloadFairness => {
                let mut s = RuleSetting::default_settings();
                for rule in &mut s {
                    match rule.key {
                        RuleKey::S1 => rule.weight = 12.0,
                        RuleKey::S8 => rule.weight = 16.0,
                        RuleKey::S9 => rule.weight = 8.0,
                        _ => {}
                    }
                }
                s
            }
            Self::TeamDiversity => {
                let mut s = RuleSetting::default_settings();
                for rule in &mut s {
                    match rule.key {
                        RuleKey::S3 => rule.weight = 8.0,
                        RuleKey::S4 => rule.weight = 12.0,
                        RuleKey::S5 => rule.weight = 12.0,
                        RuleKey::S8 => rule.weight = 6.0,
                        RuleKey::S10 => rule.weight = 6.0,
                        _ => {}
                    }
                }
                s
            }
            Self::AllowTaskCrowding => {
                let mut s = RuleSetting::default_settings();
                for rule in &mut s {
                    if rule.key == RuleKey::S9 {
                        rule.enabled = false;
                        rule.weight = 0.0;
                    }
                }
                s
            }
        }
    }
}

fn default_plan_source() -> String {
    "optimizer".to_string()
}

/// A full generated schedule across all panels for a school year.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ts_rs::TS)]
pub struct Plan {
    pub id: PlanId,
    pub school_year_id: SchoolYearId,
    pub name: String,
    pub created_at: String,
    #[ts(type = "number")]
    pub seed: u64,
    #[ts(optional)]
    pub score: Option<f64>,
    pub is_final: bool,
    #[serde(default)]
    #[ts(optional)]
    pub rank: Option<u32>,
    #[serde(default)]
    #[ts(optional)]
    pub score_report_json: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub run_params_json: Option<String>,
    #[serde(default = "default_plan_source")]
    pub source: String,
    #[serde(default)]
    #[ts(optional)]
    pub data_hash: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub rules_hash: Option<String>,
}

/// Summary record for plan listing.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ts_rs::TS)]
pub struct PlanSummary {
    pub id: PlanId,
    pub name: String,
    #[ts(optional)]
    pub rank: Option<u32>,
    #[ts(optional)]
    pub score: Option<f64>,
    pub created_at: String,
    pub is_final: bool,
    pub source: String,
    #[serde(default)]
    #[ts(optional)]
    pub data_hash: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub rules_hash: Option<String>,
    #[serde(default)]
    pub is_stale: bool,
}

/// A teacher along with their assigned grade qualifications for a specific school year.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ts_rs::TS)]
pub struct TeacherWithGrades {
    pub teacher: Teacher,
    pub grade_ids: Vec<GradeId>,
}

/// An individual assignment record within a Plan.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, ts_rs::TS)]
pub struct Assignment {
    #[serde(default)]
    pub plan_id: PlanId,
    pub exam_id: ExamId,
    pub grade_id: GradeId,
    pub subject_id: SubjectId,
    pub teacher_id: TeacherId,
    pub role: Role,
    #[serde(default)]
    pub position: usize,
}

impl Assignment {
    #[must_use]
    pub const fn new(
        exam_id: ExamId,
        grade_id: GradeId,
        subject_id: SubjectId,
        teacher_id: TeacherId,
        role: Role,
        position: usize,
    ) -> Self {
        Self {
            plan_id: PlanId(0),
            exam_id,
            grade_id,
            subject_id,
            teacher_id,
            role,
            position,
        }
    }
}

/// Identifying coordinate for an Exam Panel (Exam × Grade × Subject).
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, ts_rs::TS,
)]
pub struct PanelKey {
    pub exam_id: ExamId,
    pub grade_id: GradeId,
    pub subject_id: SubjectId,
}

impl PanelKey {
    #[inline]
    #[must_use]
    pub const fn new(exam_id: ExamId, grade_id: GradeId, subject_id: SubjectId) -> Self {
        Self {
            exam_id,
            grade_id,
            subject_id,
        }
    }
}

/// A forced or assigned seat placement in a panel.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ts_rs::TS)]
pub struct Placement {
    pub panel: PanelKey,
    pub role: Role,
    pub position: usize,
    pub teacher_id: TeacherId,
}

impl Placement {
    #[inline]
    #[must_use]
    pub const fn new(panel: PanelKey, role: Role, position: usize, teacher_id: TeacherId) -> Self {
        Self {
            panel,
            role,
            position,
            teacher_id,
        }
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

    #[test]
    fn test_rule_presets() {
        let presets = RulePreset::all();
        assert_eq!(presets.len(), 4);

        let balanced = RulePreset::Balanced.settings();
        let fairness = RulePreset::WorkloadFairness.settings();
        let diversity = RulePreset::TeamDiversity.settings();
        let crowding = RulePreset::AllowTaskCrowding.settings();

        let find_weight = |settings: &[RuleSetting], key: RuleKey| {
            settings.iter().find(|s| s.key == key).unwrap().weight
        };

        // Balanced defaults
        assert_eq!(find_weight(&balanced, RuleKey::S8), 8.0);
        assert_eq!(find_weight(&crowding, RuleKey::S9), 0.0);
        assert!(
            !crowding
                .iter()
                .find(|s| s.key == RuleKey::S9)
                .unwrap()
                .enabled
        );
        assert_eq!(find_weight(&balanced, RuleKey::S1), 10.0);
        assert_eq!(find_weight(&balanced, RuleKey::S4), 6.0);

        // Workload fairness raises S8 and S1
        assert!(find_weight(&fairness, RuleKey::S8) > find_weight(&balanced, RuleKey::S8));
        assert!(find_weight(&fairness, RuleKey::S1) > find_weight(&balanced, RuleKey::S1));

        // Team diversity raises S4, S5, S3
        assert!(find_weight(&diversity, RuleKey::S4) > find_weight(&balanced, RuleKey::S4));
        assert!(find_weight(&diversity, RuleKey::S5) > find_weight(&balanced, RuleKey::S5));
        assert!(find_weight(&diversity, RuleKey::S3) > find_weight(&balanced, RuleKey::S3));
    }
}
