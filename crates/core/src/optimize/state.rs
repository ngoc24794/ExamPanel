//! Incremental state representation for local search soft-constraint evaluation.
//!
//! Maintains compact dense index arrays and fast O(1) counters for all S1–S8 terms.

use crate::domain::{
    calculate_quotas, Assignment, CampusId, ExamId, GradeId, LockKind, Problem, Role, RuleKey,
    TeacherId,
};
use crate::score::{evaluate, ScoreReport};
use std::collections::{HashMap, HashSet};

/// Panel slot identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SlotRole {
    Setter1,
    Setter2,
    Reviewer,
}

impl SlotRole {
    #[must_use]
    pub const fn to_role(self) -> Role {
        match self {
            Self::Setter1 | Self::Setter2 => Role::Setter,
            Self::Reviewer => Role::Reviewer,
        }
    }
}

/// A panel in the dense index representation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DensePanel {
    pub exam_idx: usize,
    pub grade_idx: usize,
    pub setter1: usize,
    pub setter2: usize,
    pub reviewer: usize,
    pub pinned_s1: bool,
    pub pinned_s2: bool,
    pub pinned_rev: bool,
}

impl DensePanel {
    #[must_use]
    pub fn get_slot(&self, slot: SlotRole) -> usize {
        match slot {
            SlotRole::Setter1 => self.setter1,
            SlotRole::Setter2 => self.setter2,
            SlotRole::Reviewer => self.reviewer,
        }
    }

    pub fn set_slot(&mut self, slot: SlotRole, teacher_idx: usize) {
        match slot {
            SlotRole::Setter1 => self.setter1 = teacher_idx,
            SlotRole::Setter2 => self.setter2 = teacher_idx,
            SlotRole::Reviewer => self.reviewer = teacher_idx,
        }
    }

    #[must_use]
    pub fn is_pinned(&self, slot: SlotRole) -> bool {
        match slot {
            SlotRole::Setter1 => self.pinned_s1,
            SlotRole::Setter2 => self.pinned_s2,
            SlotRole::Reviewer => self.pinned_rev,
        }
    }
}

/// Dense incremental state for evaluating soft rules S1–S8.
#[derive(Debug, Clone)]
pub struct IncrementalState {
    // Problem entity mapping
    pub teacher_ids: Vec<TeacherId>,
    pub teacher_map: HashMap<TeacherId, usize>,
    pub exam_ids: Vec<ExamId>,
    pub grade_ids: Vec<GradeId>,
    pub campuses: Vec<usize>, // teacher_idx -> campus_idx
    pub quotas: Vec<f64>,
    pub bounds: Vec<(usize, usize)>, // (lo, hi)
    pub reviewer_eligible: Vec<bool>,
    pub qualified_grades: Vec<Vec<bool>>, // [t][g] -> bool
    pub unavailabilities: Vec<Vec<bool>>, // [t][e] -> bool
    pub forbids_setter: Vec<Vec<HashSet<usize>>>, // [e][g] -> set of teacher_idx
    pub forbids_reviewer: Vec<Vec<HashSet<usize>>>, // [e][g] -> set of teacher_idx

    // Rule weights and enabled flags [0..7 for S1..S8]
    pub rule_enabled: [bool; 8],
    pub rule_weights: [f64; 8],

    // Panels state
    pub panels: Vec<DensePanel>, // length P = E * G

    // Dynamic counters
    pub teacher_count: Vec<usize>,
    pub teacher_setters: Vec<usize>,
    pub teacher_reviewers: Vec<usize>,
    pub teacher_exam_role: Vec<Vec<Option<Role>>>, // [t][e]
    pub teacher_grade_counts: Vec<Vec<usize>>,     // [t][g]
    pub teacher_distinct_grades: Vec<usize>,       // [t]
    pub setter_pairs: Vec<Vec<usize>>,             // [u][v] with u < v
    pub review_relations: Vec<Vec<usize>>,         // [rev][setter]

    // Sorted exam indices for S6 consecutive setting
    pub sorted_exam_indices: Vec<usize>,

    // Current cached score
    pub current_units: [f64; 8],
    pub current_penalty: f64,
}

impl IncrementalState {
    /// Builds incremental state from a Problem and complete assignments.
    #[must_use]
    pub fn new(problem: &Problem, assignments: &[Assignment]) -> Self {
        let num_teachers = problem.teachers.len();
        let num_exams = problem.exams.len();
        let num_grades = problem.grades.len();

        let teacher_ids: Vec<TeacherId> = problem.teachers.iter().map(|t| t.id).collect();
        let teacher_map: HashMap<TeacherId, usize> = teacher_ids
            .iter()
            .enumerate()
            .map(|(idx, &id)| (id, idx))
            .collect();

        let exam_ids: Vec<ExamId> = problem.exams.iter().map(|e| e.id).collect();
        let exam_map: HashMap<ExamId, usize> = exam_ids
            .iter()
            .enumerate()
            .map(|(idx, &id)| (id, idx))
            .collect();

        let grade_ids: Vec<GradeId> = problem.grades.iter().map(|g| g.id).collect();
        let grade_map: HashMap<GradeId, usize> = grade_ids
            .iter()
            .enumerate()
            .map(|(idx, &id)| (id, idx))
            .collect();

        let campus_ids: Vec<CampusId> = problem.campuses.iter().map(|c| c.id).collect();
        let campus_map: HashMap<CampusId, usize> = campus_ids
            .iter()
            .enumerate()
            .map(|(idx, &id)| (id, idx))
            .collect();

        let campuses: Vec<usize> = problem
            .teachers
            .iter()
            .map(|t| campus_map.get(&t.campus_id).copied().unwrap_or(0))
            .collect();

        let quotas_res = calculate_quotas(problem);
        let mut quotas = vec![0.0; num_teachers];
        let mut bounds = vec![(0, 0); num_teachers];
        for q in quotas_res {
            if let Some(&t_idx) = teacher_map.get(&q.teacher_id) {
                quotas[t_idx] = q.quota;
                bounds[t_idx] = (q.lo, q.hi);
            }
        }

        let mut qualified_grades = vec![vec![false; num_grades]; num_teachers];
        for tg in &problem.teacher_grades {
            if tg.school_year_id == problem.school_year.id {
                if let (Some(&t_idx), Some(&g_idx)) =
                    (teacher_map.get(&tg.teacher_id), grade_map.get(&tg.grade_id))
                {
                    qualified_grades[t_idx][g_idx] = true;
                }
            }
        }

        let mut unavailabilities = vec![vec![false; num_exams]; num_teachers];
        for u in &problem.unavailabilities {
            if let (Some(&t_idx), Some(&e_idx)) =
                (teacher_map.get(&u.teacher_id), exam_map.get(&u.exam_id))
            {
                unavailabilities[t_idx][e_idx] = true;
            }
        }

        let mut forbids_setter = vec![vec![HashSet::new(); num_grades]; num_exams];
        let mut forbids_reviewer = vec![vec![HashSet::new(); num_grades]; num_exams];
        for lock in &problem.locks {
            if lock.kind == LockKind::Forbid {
                if let (Some(&e_idx), Some(&g_idx), Some(&t_idx)) = (
                    exam_map.get(&lock.exam_id),
                    grade_map.get(&lock.grade_id),
                    teacher_map.get(&lock.teacher_id),
                ) {
                    match lock.role {
                        None => {
                            forbids_setter[e_idx][g_idx].insert(t_idx);
                            forbids_reviewer[e_idx][g_idx].insert(t_idx);
                        }
                        Some(Role::Setter) => {
                            forbids_setter[e_idx][g_idx].insert(t_idx);
                        }
                        Some(Role::Reviewer) => {
                            forbids_reviewer[e_idx][g_idx].insert(t_idx);
                        }
                    }
                }
            }
        }

        // Reviewer eligibility: active, load_weight > 0, quota >= 1.0, and has at least one panel where eligible
        let mut reviewer_eligible = vec![false; num_teachers];
        for (t_idx, t) in problem.teachers.iter().enumerate() {
            if !t.active || t.load_weight <= 0.0 || quotas[t_idx] < 1.0 {
                continue;
            }
            let eligible_any = (0..num_exams).any(|e_idx| {
                if unavailabilities[t_idx][e_idx] {
                    return false;
                }
                (0..num_grades).any(|g_idx| {
                    qualified_grades[t_idx][g_idx]
                        && !forbids_reviewer[e_idx][g_idx].contains(&t_idx)
                })
            });
            reviewer_eligible[t_idx] = eligible_any;
        }

        // Sorted exams by sort_order
        let mut sorted_exams: Vec<(usize, i32)> = problem
            .exams
            .iter()
            .enumerate()
            .map(|(idx, e)| (idx, e.sort_order))
            .collect();
        sorted_exams.sort_by_key(|&(_, order)| order);
        let sorted_exam_indices: Vec<usize> =
            sorted_exams.into_iter().map(|(idx, _)| idx).collect();

        // Rule settings
        let rule_keys = [
            RuleKey::S1,
            RuleKey::S2,
            RuleKey::S3,
            RuleKey::S4,
            RuleKey::S5,
            RuleKey::S6,
            RuleKey::S7,
            RuleKey::S8,
        ];
        let default_weights = [10.0, 3.0, 4.0, 6.0, 6.0, 2.0, 1.0, 8.0];
        let mut rule_enabled = [true; 8];
        let mut rule_weights = [0.0; 8];

        for (i, &key) in rule_keys.iter().enumerate() {
            if let Some(s) = problem.rule_settings.iter().find(|s| s.key == key) {
                rule_enabled[i] = s.enabled;
                rule_weights[i] = s.weight;
            } else {
                rule_enabled[i] = true;
                rule_weights[i] = default_weights[i];
            }
        }

        // Populate panels and initial assignments
        let num_panels = num_exams * num_grades;
        let mut panels = Vec::with_capacity(num_panels);

        for e_idx in 0..num_exams {
            for g_idx in 0..num_grades {
                panels.push(DensePanel {
                    exam_idx: e_idx,
                    grade_idx: g_idx,
                    setter1: usize::MAX,
                    setter2: usize::MAX,
                    reviewer: usize::MAX,
                    pinned_s1: false,
                    pinned_s2: false,
                    pinned_rev: false,
                });
            }
        }

        // Populate PIN status from problem.locks
        for lock in &problem.locks {
            if lock.kind == LockKind::Pin {
                if let (Some(&e_idx), Some(&g_idx), Some(&t_idx)) = (
                    exam_map.get(&lock.exam_id),
                    grade_map.get(&lock.grade_id),
                    teacher_map.get(&lock.teacher_id),
                ) {
                    let p_idx = e_idx * num_grades + g_idx;
                    // If role specified, record pin
                    if lock.role == Some(Role::Reviewer) {
                        panels[p_idx].pinned_rev = true;
                        panels[p_idx].reviewer = t_idx;
                    } else if lock.role == Some(Role::Setter) {
                        if !panels[p_idx].pinned_s1 {
                            panels[p_idx].pinned_s1 = true;
                            panels[p_idx].setter1 = t_idx;
                        } else {
                            panels[p_idx].pinned_s2 = true;
                            panels[p_idx].setter2 = t_idx;
                        }
                    }
                }
            }
        }

        // Insert assignments into panels
        for a in assignments {
            if let (Some(&e_idx), Some(&g_idx), Some(&t_idx)) = (
                exam_map.get(&a.exam_id),
                grade_map.get(&a.grade_id),
                teacher_map.get(&a.teacher_id),
            ) {
                let p_idx = e_idx * num_grades + g_idx;
                match a.role {
                    Role::Reviewer => {
                        panels[p_idx].reviewer = t_idx;
                    }
                    Role::Setter => {
                        if panels[p_idx].setter1 == usize::MAX || panels[p_idx].setter1 == t_idx {
                            panels[p_idx].setter1 = t_idx;
                        } else {
                            panels[p_idx].setter2 = t_idx;
                        }
                    }
                }
            }
        }

        // Normalize panel setters (s1 < s2)
        for panel in &mut panels {
            if panel.setter1 > panel.setter2 {
                std::mem::swap(&mut panel.setter1, &mut panel.setter2);
                std::mem::swap(&mut panel.pinned_s1, &mut panel.pinned_s2);
            }
        }

        // Initialize counters
        let mut state = Self {
            teacher_ids,
            teacher_map,
            exam_ids,
            grade_ids,
            campuses,
            quotas,
            bounds,
            reviewer_eligible,
            qualified_grades,
            unavailabilities,
            forbids_setter,
            forbids_reviewer,
            rule_enabled,
            rule_weights,
            panels,
            teacher_count: vec![0; num_teachers],
            teacher_setters: vec![0; num_teachers],
            teacher_reviewers: vec![0; num_teachers],
            teacher_exam_role: vec![vec![None; num_exams]; num_teachers],
            teacher_grade_counts: vec![vec![0; num_grades]; num_teachers],
            teacher_distinct_grades: vec![0; num_teachers],
            setter_pairs: vec![vec![0; num_teachers]; num_teachers],
            review_relations: vec![vec![0; num_teachers]; num_teachers],
            sorted_exam_indices,
            current_units: [0.0; 8],
            current_penalty: 0.0,
        };

        state.rebuild_counters_and_full_eval();
        state
    }

    /// Fully rebuilds dynamic counters from panels and calculates full score.
    pub fn rebuild_counters_and_full_eval(&mut self) {
        let num_teachers = self.teacher_ids.len();
        let num_exams = self.exam_ids.len();
        let num_grades = self.grade_ids.len();

        self.teacher_count = vec![0; num_teachers];
        self.teacher_setters = vec![0; num_teachers];
        self.teacher_reviewers = vec![0; num_teachers];
        self.teacher_exam_role = vec![vec![None; num_exams]; num_teachers];
        self.teacher_grade_counts = vec![vec![0; num_grades]; num_teachers];
        self.teacher_distinct_grades = vec![0; num_teachers];
        self.setter_pairs = vec![vec![0; num_teachers]; num_teachers];
        self.review_relations = vec![vec![0; num_teachers]; num_teachers];

        for panel in &self.panels {
            let e = panel.exam_idx;
            let g = panel.grade_idx;
            let s1 = panel.setter1;
            let s2 = panel.setter2;
            let rev = panel.reviewer;

            // S1 & S2 counters
            self.teacher_count[s1] += 1;
            self.teacher_setters[s1] += 1;
            self.teacher_exam_role[s1][e] = Some(Role::Setter);
            if self.teacher_grade_counts[s1][g] == 0 {
                self.teacher_distinct_grades[s1] += 1;
            }
            self.teacher_grade_counts[s1][g] += 1;

            self.teacher_count[s2] += 1;
            self.teacher_setters[s2] += 1;
            self.teacher_exam_role[s2][e] = Some(Role::Setter);
            if self.teacher_grade_counts[s2][g] == 0 {
                self.teacher_distinct_grades[s2] += 1;
            }
            self.teacher_grade_counts[s2][g] += 1;

            self.teacher_count[rev] += 1;
            self.teacher_reviewers[rev] += 1;
            self.teacher_exam_role[rev][e] = Some(Role::Reviewer);
            if self.teacher_grade_counts[rev][g] == 0 {
                self.teacher_distinct_grades[rev] += 1;
            }
            self.teacher_grade_counts[rev][g] += 1;

            // S4 setter pair
            let (u, v) = if s1 < s2 { (s1, s2) } else { (s2, s1) };
            self.setter_pairs[u][v] += 1;

            // S5 review relation
            self.review_relations[rev][s1] += 1;
            self.review_relations[rev][s2] += 1;
        }

        // Full score computation
        self.current_units = [0.0; 8];

        // S1: Reviewer count
        for t in 0..num_teachers {
            if self.reviewer_eligible[t] {
                let r = self.teacher_reviewers[t];
                if r == 0 {
                    self.current_units[0] += 1.0;
                } else if r > 2 {
                    self.current_units[0] += (r - 2) as f64;
                }
            }
        }

        // S2: Role balance
        for t in 0..num_teachers {
            let count = self.teacher_count[t];
            if count >= 2 {
                let r = self.teacher_reviewers[t] as f64;
                let ideal = count as f64 / 3.0;
                self.current_units[1] += (r - ideal).abs();
            }
        }

        // S3: Independent reviewer
        for panel in &self.panels {
            let r_camp = self.campuses[panel.reviewer];
            if self.campuses[panel.setter1] == r_camp {
                self.current_units[2] += 1.0;
            }
            if self.campuses[panel.setter2] == r_camp {
                self.current_units[2] += 1.0;
            }
        }

        // S4: Repeated setter pair
        for u in 0..num_teachers {
            for v in (u + 1)..num_teachers {
                let c = self.setter_pairs[u][v];
                if c > 1 {
                    self.current_units[3] += (c - 1) as f64;
                }
            }
        }

        // S5: Repeated review relation
        for r in 0..num_teachers {
            for s in 0..num_teachers {
                let c = self.review_relations[r][s];
                if c > 1 {
                    self.current_units[4] += (c - 1) as f64;
                }
            }
        }

        // S6: Consecutive setting
        if self.sorted_exam_indices.len() >= 2 {
            for t in 0..num_teachers {
                for i in 0..(self.sorted_exam_indices.len() - 1) {
                    let e1 = self.sorted_exam_indices[i];
                    let e2 = self.sorted_exam_indices[i + 1];
                    if self.teacher_exam_role[t][e1] == Some(Role::Setter)
                        && self.teacher_exam_role[t][e2] == Some(Role::Setter)
                    {
                        self.current_units[5] += 1.0;
                    }
                }
            }
        }

        // S7: Grade rotation
        for t in 0..num_teachers {
            let qualified_count = self.qualified_grades[t].iter().filter(|&&q| q).count();
            if qualified_count >= 2 {
                let count = self.teacher_count[t];
                let target = count.min(qualified_count);
                let assigned = self.teacher_distinct_grades[t];
                if assigned < target {
                    self.current_units[6] += (target - assigned) as f64;
                }
            }
        }

        // S8: Load balance
        for t in 0..num_teachers {
            let diff = self.teacher_count[t] as f64 - self.quotas[t];
            self.current_units[7] += diff * diff;
        }

        self.recompute_penalty();
    }

    #[inline]
    fn recompute_penalty(&mut self) {
        let mut total = 0.0;
        for i in 0..8 {
            if self.rule_enabled[i] {
                total += self.rule_weights[i] * self.current_units[i];
            }
        }
        self.current_penalty = total;
    }

    /// Converts current dense state to complete Vec<Assignment>.
    #[must_use]
    pub fn to_assignments(&self, plan_id: crate::domain::PlanId) -> Vec<Assignment> {
        let mut list = Vec::with_capacity(self.panels.len() * 3);
        for panel in &self.panels {
            let eid = self.exam_ids[panel.exam_idx];
            let gid = self.grade_ids[panel.grade_idx];

            let mut a1 = Assignment::new(eid, gid, self.teacher_ids[panel.setter1], Role::Setter);
            a1.plan_id = plan_id;
            list.push(a1);

            let mut a2 = Assignment::new(eid, gid, self.teacher_ids[panel.setter2], Role::Setter);
            a2.plan_id = plan_id;
            list.push(a2);

            let mut a3 =
                Assignment::new(eid, gid, self.teacher_ids[panel.reviewer], Role::Reviewer);
            a3.plan_id = plan_id;
            list.push(a3);
        }
        list
    }

    /// Computes full ScoreReport via core evaluate module to cross-check.
    #[must_use]
    pub fn full_evaluate(&self, problem: &Problem) -> ScoreReport {
        let assigns = self.to_assignments(crate::domain::PlanId(0));
        evaluate(problem, &assigns)
    }
}
