// This file is generated from Rust DTOs in crates/service/src/dto.rs and crates/core/src/domain/.
// Do not edit manually. Verify with `cargo test -p exam-panel-service --test generate_types`.

export type Role = 'setter' | 'reviewer'

export type LockKind = 'pin' | 'forbid'

export type RuleKey =
  | 'h1'
  | 'h2'
  | 'h3'
  | 'h4'
  | 'h5'
  | 'h6'
  | 'h7'
  | 's1'
  | 's2'
  | 's3'
  | 's4'
  | 's5'
  | 's6'
  | 's7'
  | 's8'

export interface AppInfo {
  version: string
  data_dir: string
  is_portable: boolean
  db_path: string
  name?: string
  identifier?: string
  mode?: 'tauri' | 'mock'
}

export interface AppSettings {
  theme: string
  language: string
  current_school_year_id: number | null
}

export interface CreateCampusInput {
  code: string
  name: string
  color: string
}

export interface Campus {
  id: number
  code: string
  name: string
  color: string
}

export interface CreateGradeInput {
  code: number
  name: string
  sort_order: number
}

export interface Grade {
  id: number
  code: number
  name: string
  sort_order: number
}

export interface CreateTeacherInput {
  full_name: string
  campus_id: number
  load_weight: number
  active: boolean
  note?: string | null
}

export interface Teacher {
  id: number
  full_name: string
  campus_id: number
  load_weight: number
  active: boolean
  note: string | null
}

export interface TeacherWithGrades {
  teacher: Teacher
  grade_ids: number[]
}

export interface CreateSchoolYearInput {
  name: string
  is_current?: boolean
  copy_grades_from?: number | null
}

export interface SchoolYear {
  id: number
  name: string
  is_current: boolean
}

export interface Exam {
  id: number
  school_year_id: number
  code: string
  name: string
  sort_order: number
}

export interface Unavailability {
  teacher_id: number
  exam_id: number
  reason?: string | null
}

export interface CreateLockInput {
  exam_id: number
  grade_id: number
  teacher_id: number
  role?: Role | null
  kind: LockKind
}

export interface Lock {
  id: number
  school_year_id: number
  exam_id: number
  grade_id: number
  teacher_id: number
  role: Role | null
  kind: LockKind
}

export interface RuleSetting {
  key: RuleKey
  enabled: boolean
  weight: number
  params: Record<string, unknown>
}

export interface PanelKey {
  exam_id: number
  grade_id: number
}

export interface Violation {
  rule: RuleKey
  code: string
  panel?: PanelKey | null
  teacher?: number | null
  params: Record<string, unknown>
}

export interface TeacherQuota {
  teacher_id: number
  quota: number
  lo: number
  hi: number
}

export interface DiagnosticCapacity {
  target_slots: number
  active_teachers: number
  total_weight: number
}

export interface FeasibilityReport {
  is_feasible: boolean
  errors: Violation[]
  warnings: Violation[]
  capacity: DiagnosticCapacity
}

export interface FeasibilityReportWithQuotas {
  report: FeasibilityReport
  quotas: TeacherQuota[]
}

export interface Assignment {
  plan_id: number
  exam_id: number
  grade_id: number
  teacher_id: number
  role: Role
}

export interface SoftViolation {
  rule: RuleKey
  code: string
  panel?: PanelKey | null
  teachers: number[]
  params: Record<string, string>
}

export interface TeacherStats {
  teacher_id: number
  count: number
  setter_count: number
  reviewer_count: number
  target_quota: number
  delta: number
  cost: number
}

export interface RuleScore {
  rule: RuleKey
  enabled: boolean
  weight: number
  units: number
  penalty: number
  lower_bound: number
}

export interface ScoreReport {
  total: number
  by_rule: RuleScore[]
  violations: SoftViolation[]
  per_teacher: TeacherStats[]
}

export interface EvaluationOutcome {
  hard_violations: Violation[]
  score_report: ScoreReport
}

export type OptimizeBudget =
  { type: 'Iterations'; value: number } | { type: 'TimeMs'; value: number }

export interface OptimizeRequest {
  base_seed?: number | null
  runs: number
  budget: OptimizeBudget
  k: number
  diversity_threshold?: number | null
}

export interface Progress {
  run: number
  iteration: number
  best_score: number
  current_score: number
  elapsed_ms: number
}

export interface RankedPlan {
  rank: number
  seed: number
  assignments: Assignment[]
  report: ScoreReport
}

export interface OptimizeStats {
  total_runs: number
  total_iterations: number
  elapsed_ms: number
}

export interface RuleBound {
  rule: RuleKey
  units_lower_bound: number
  method: string
}

export interface OptimizeOutcome {
  school_year_id: number
  plans: RankedPlan[]
  initial_report: ScoreReport
  stats: OptimizeStats
  lower_bounds: RuleBound[]
  run_params_json: string
}

export interface PlanSummary {
  id: number
  name: string
  rank?: number | null
  score?: number | null
  created_at: string
  is_final: boolean
  source: string
}

export interface Plan {
  id: number
  school_year_id: number
  name: string
  created_at: string
  seed: number
  score?: number | null
  is_final: boolean
  rank?: number | null
  score_report_json?: string | null
  run_params_json?: string | null
  source: string
}

export interface PlanDetails {
  plan: Plan
  assignments: Assignment[]
  score_report?: ScoreReport | null
}

export interface AppError {
  code: string
  params?: Record<string, unknown>
}
