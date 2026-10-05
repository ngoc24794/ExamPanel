//! Incremental state representation for local search soft-constraint evaluation.
//!
//! Maintains compact dense index arrays and fast O(1) counters for all S1–S8 terms.

use crate::domain::forced::{find_forced_placements, is_teacher_eligible};
use crate::domain::{
    calculate_quotas, Assignment, CampusId, ExamId, GradeId, LockKind, Problem, Role, RuleKey,
    SubjectId, TeacherId,
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
    pub subject_idx: usize,
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

/// Dense incremental state for evaluating soft rules S1–S10.
#[derive(Debug, Clone)]
pub struct IncrementalState {
    // Problem entity mapping
    pub teacher_ids: Vec<TeacherId>,
    pub teacher_map: HashMap<TeacherId, usize>,
    pub exam_ids: Vec<ExamId>,
    pub grade_ids: Vec<GradeId>,
    pub subject_ids: Vec<SubjectId>,
    pub subject_map: HashMap<SubjectId, usize>,
    pub campuses: Vec<usize>, // teacher_idx -> campus_idx
    pub quotas: Vec<f64>,
    pub bounds: Vec<(usize, usize)>, // (lo, hi)
    pub reviewer_eligible: Vec<bool>,
    pub qualified_grades: Vec<Vec<bool>>, // [t][g] -> bool
    pub unavailabilities: Vec<Vec<bool>>, // [t][e] -> bool
    pub forbids_setter: Vec<Vec<HashSet<usize>>>, // [e][g] -> set of teacher_idx
    pub forbids_reviewer: Vec<Vec<HashSet<usize>>>, // [e][g] -> set of teacher_idx
    pub panel_min_campuses: Vec<u8>,
    pub max_tasks_per_exam: Vec<Vec<usize>>,   // [t][e]
    pub max_setters_per_exam: Vec<Vec<usize>>, // [t][e]
    pub slot_eligible: Vec<Vec<[bool; 2]>>,    // [t][p_idx][role_idx: 0=Setter, 1=Reviewer]

    // Rule weights and enabled flags [0..9 for S1..S10]
    pub rule_enabled: [bool; 10],
    pub rule_weights: [f64; 10],

    // S1 dynamic cap
    pub s1_max_reviews: usize,
    pub reviewer_capable: Vec<bool>,

    // S2 role balance
    pub s2_rho: f64,
    pub both_roles_capable: Vec<bool>,

    // Forced task tracking for S2, S6, S8, S9
    pub forced_tasks: Vec<usize>,
    pub forced_setters: Vec<usize>,
    pub forced_reviewers: Vec<usize>,
    pub forced_exam_tasks: Vec<Vec<usize>>,   // [t][e]
    pub forced_exam_setters: Vec<Vec<usize>>, // [t][e]
    pub has_quota_override: Vec<bool>,

    // Panels state
    pub panels: Vec<DensePanel>,

    // Dynamic counters
    pub teacher_count: Vec<usize>,
    pub teacher_setters: Vec<usize>,
    pub teacher_reviewers: Vec<usize>,
    pub teacher_exam_tasks: Vec<Vec<usize>>,        // [t][e]
    pub teacher_exam_setters: Vec<Vec<usize>>,      // [t][e]
    pub teacher_exam_role: Vec<Vec<Option<Role>>>,  // [t][e]
    pub teacher_grade_counts: Vec<Vec<usize>>,      // [t][g]
    pub teacher_distinct_grades: Vec<usize>,        // [t]
    pub setter_pairs: Vec<Vec<usize>>,              // [u][v] with u < v
    pub review_relations: Vec<Vec<usize>>,          // [rev][setter]
    pub reviewer_subject_competent: Vec<Vec<bool>>, // [t][s]
    pub teacher_subject_reviews: Vec<Vec<usize>>,   // [t][s]

    // Available exams per teacher for S9 offset
    pub teacher_available_exams: Vec<usize>,

    // Sorted exam indices for S6 consecutive setting
    pub sorted_exam_indices: Vec<usize>,

    // Current cached score
    pub current_units: [f64; 10],
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
        let effective_subjects = problem.effective_subjects();
        let num_subjects = effective_subjects.len();
        let subject_ids: Vec<SubjectId> = effective_subjects.iter().map(|s| s.id).collect();
        let subject_map: HashMap<SubjectId, usize> = subject_ids
            .iter()
            .enumerate()
            .map(|(idx, &id)| (id, idx))
            .collect();

        // Rule settings S1..S10
        let rule_keys = [
            RuleKey::S1,
            RuleKey::S2,
            RuleKey::S3,
            RuleKey::S4,
            RuleKey::S5,
            RuleKey::S6,
            RuleKey::S7,
            RuleKey::S8,
            RuleKey::S9,
            RuleKey::S10,
        ];
        let default_weights = [10.0, 3.0, 4.0, 6.0, 6.0, 2.0, 1.0, 8.0, 5.0, 4.0];
        let mut rule_enabled = [true; 10];
        let mut rule_weights = [0.0; 10];

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
        let num_panels = num_exams * num_grades * num_subjects;
        let mut panels = Vec::with_capacity(num_panels);

        for e_idx in 0..num_exams {
            for g_idx in 0..num_grades {
                for (s_idx, sub) in effective_subjects.iter().enumerate() {
                    panels.push(DensePanel {
                        exam_idx: e_idx,
                        grade_idx: g_idx,
                        subject_idx: s_idx,
                        setter1: usize::MAX,
                        setter2: usize::MAX,
                        reviewer: usize::MAX,
                        pinned_s1: false,
                        pinned_s2: sub.setters < 2,
                        pinned_rev: sub.reviewers == 0,
                    });
                }
            }
        }

        // Populate PIN status from forced placements
        if let Ok(forced) = find_forced_placements(problem) {
            for p in forced {
                if let (Some(&e_idx), Some(&g_idx), Some(&s_idx), Some(&t_idx)) = (
                    exam_map.get(&p.panel.exam_id),
                    grade_map.get(&p.panel.grade_id),
                    subject_map.get(&p.panel.subject_id),
                    teacher_map.get(&p.teacher_id),
                ) {
                    if let Some(panel) = panels.iter_mut().find(|pan| {
                        pan.exam_idx == e_idx && pan.grade_idx == g_idx && pan.subject_idx == s_idx
                    }) {
                        match p.role {
                            Role::Reviewer => {
                                panel.reviewer = t_idx;
                                panel.pinned_rev = true;
                            }
                            Role::Setter => {
                                if p.position == 0 {
                                    panel.setter1 = t_idx;
                                    panel.pinned_s1 = true;
                                } else {
                                    panel.setter2 = t_idx;
                                    panel.pinned_s2 = true;
                                }
                            }
                        }
                    }
                }
            }
        }

        // Populate PIN status from problem.locks
        for lock in &problem.locks {
            if lock.kind == LockKind::Pin {
                if let (Some(&e_idx), Some(&g_idx), Some(&s_idx), Some(&t_idx)) = (
                    exam_map.get(&lock.exam_id),
                    grade_map.get(&lock.grade_id),
                    subject_map.get(&lock.subject_id),
                    teacher_map.get(&lock.teacher_id),
                ) {
                    if let Some(panel) = panels.iter_mut().find(|pan| {
                        pan.exam_idx == e_idx && pan.grade_idx == g_idx && pan.subject_idx == s_idx
                    }) {
                        if lock.role == Some(Role::Reviewer) {
                            panel.pinned_rev = true;
                            panel.reviewer = t_idx;
                        } else if lock.role == Some(Role::Setter) {
                            if !panel.pinned_s1 {
                                panel.pinned_s1 = true;
                                panel.setter1 = t_idx;
                            } else {
                                panel.pinned_s2 = true;
                                panel.setter2 = t_idx;
                            }
                        }
                    }
                }
            }
        }

        // Insert assignments into panels
        for a in assignments {
            if let (Some(&e_idx), Some(&g_idx), Some(&s_idx), Some(&t_idx)) = (
                exam_map.get(&a.exam_id),
                grade_map.get(&a.grade_id),
                subject_map.get(&a.subject_id),
                teacher_map.get(&a.teacher_id),
            ) {
                if let Some(panel) = panels.iter_mut().find(|pan| {
                    pan.exam_idx == e_idx && pan.grade_idx == g_idx && pan.subject_idx == s_idx
                }) {
                    match a.role {
                        Role::Reviewer => {
                            panel.reviewer = t_idx;
                        }
                        Role::Setter => {
                            if a.position == 0
                                || panel.setter1 == usize::MAX
                                || panel.setter1 == t_idx
                            {
                                panel.setter1 = t_idx;
                            } else {
                                panel.setter2 = t_idx;
                            }
                        }
                    }
                }
            }
        }

        // Normalize panel setters (s1 < s2)
        for panel in &mut panels {
            if panel.setter1 != usize::MAX
                && panel.setter2 != usize::MAX
                && panel.setter1 > panel.setter2
            {
                std::mem::swap(&mut panel.setter1, &mut panel.setter2);
                std::mem::swap(&mut panel.pinned_s1, &mut panel.pinned_s2);
            }
        }

        let forced = find_forced_placements(problem).unwrap_or_default();
        let non_forced_reviewer_seats: usize = problem
            .all_panels()
            .iter()
            .map(|p| {
                let sub = effective_subjects
                    .iter()
                    .find(|s| s.id == p.subject_id)
                    .unwrap();
                sub.reviewers as usize
            })
            .sum::<usize>()
            .saturating_sub(forced.iter().filter(|fp| fp.role == Role::Reviewer).count());

        let mut reviewer_capable = vec![false; num_teachers];
        for (t_idx, t) in problem.teachers.iter().enumerate() {
            if !t.active || t.load_weight <= 0.0 || quotas[t_idx] < 1.0 {
                continue;
            }
            let is_comp = problem
                .competencies
                .iter()
                .any(|c| c.teacher_id == t.id && c.role == Role::Reviewer);
            reviewer_capable[t_idx] = is_comp;
        }
        let s1_cfg = problem
            .rule_settings
            .iter()
            .find(|s| s.key == RuleKey::S1)
            .and_then(|s| {
                s.params
                    .get("max_reviews")
                    .and_then(serde_json::Value::as_u64)
            });
        let num_rev_capable = reviewer_capable.iter().filter(|&&c| c).count();
        let auto_max_reviews = if num_rev_capable > 0 {
            (non_forced_reviewer_seats as f64 / num_rev_capable as f64).ceil() as usize
        } else {
            2
        };
        let s1_max_reviews = s1_cfg.map_or(auto_max_reviews, |v| v as usize);

        let total_non_forced_seats = (problem
            .all_panels()
            .iter()
            .map(|p| {
                let sub = effective_subjects
                    .iter()
                    .find(|s| s.id == p.subject_id)
                    .unwrap();
                (sub.setters + sub.reviewers) as usize
            })
            .sum::<usize>())
        .saturating_sub(forced.len());

        let s2_rho = if total_non_forced_seats > 0 {
            non_forced_reviewer_seats as f64 / total_non_forced_seats as f64
        } else {
            1.0 / 3.0
        };

        let mut both_roles_capable = vec![false; num_teachers];
        for (t_idx, t) in problem.teachers.iter().enumerate() {
            let has_s = problem
                .competencies
                .iter()
                .any(|c| c.teacher_id == t.id && c.role == Role::Setter);
            let has_r = problem
                .competencies
                .iter()
                .any(|c| c.teacher_id == t.id && c.role == Role::Reviewer);
            both_roles_capable[t_idx] = has_s && has_r;
        }

        let mut forced_tasks = vec![0; num_teachers];
        let mut forced_setters = vec![0; num_teachers];
        let mut forced_reviewers = vec![0; num_teachers];
        let mut forced_exam_tasks = vec![vec![0; num_exams]; num_teachers];
        let mut forced_exam_setters = vec![vec![0; num_exams]; num_teachers];
        for fp in &forced {
            if let (Some(&t_idx), Some(&e_idx)) = (
                teacher_map.get(&fp.teacher_id),
                exam_map.get(&fp.panel.exam_id),
            ) {
                forced_tasks[t_idx] += 1;
                forced_exam_tasks[t_idx][e_idx] += 1;
                match fp.role {
                    Role::Setter => {
                        forced_setters[t_idx] += 1;
                        forced_exam_setters[t_idx][e_idx] += 1;
                    }
                    Role::Reviewer => {
                        forced_reviewers[t_idx] += 1;
                    }
                }
            }
        }
        let has_quota_override: Vec<bool> = problem
            .teachers
            .iter()
            .map(|t| t.quota_override.is_some())
            .collect();

        let mut reviewer_subject_competent = vec![vec![false; num_subjects]; num_teachers];
        for (t_idx, t) in problem.teachers.iter().enumerate() {
            for (s_idx, sub) in effective_subjects.iter().enumerate() {
                let is_comp = problem.competencies.iter().any(|c| {
                    c.teacher_id == t.id && c.subject_id == sub.id && c.role == Role::Reviewer
                });
                reviewer_subject_competent[t_idx][s_idx] = is_comp;
            }
        }

        let default_h4 = problem
            .rule_settings
            .iter()
            .find(|s| s.key == RuleKey::H4)
            .map(|s| {
                let tasks = s
                    .params
                    .get("max_tasks_per_exam")
                    .and_then(serde_json::Value::as_u64)
                    .unwrap_or(2) as usize;
                let setters = s
                    .params
                    .get("max_setter_per_exam")
                    .and_then(serde_json::Value::as_u64)
                    .unwrap_or(1) as usize;
                (tasks, setters)
            })
            .unwrap_or((2, 1));

        let mut max_tasks_per_exam = vec![vec![0; num_exams]; num_teachers];
        let mut max_setters_per_exam = vec![vec![0; num_exams]; num_teachers];
        for (t_idx, t) in problem.teachers.iter().enumerate() {
            let configured_tasks = t
                .max_tasks_per_exam_override
                .map_or(default_h4.0, |v| v as usize);
            for e_idx in 0..num_exams {
                let ft = forced_exam_tasks[t_idx][e_idx];
                let fs = forced_exam_setters[t_idx][e_idx];
                max_tasks_per_exam[t_idx][e_idx] = configured_tasks.max(ft);
                max_setters_per_exam[t_idx][e_idx] = default_h4.1.max(fs);
            }
        }

        let mut slot_eligible = vec![vec![[false, false]; panels.len()]; num_teachers];
        for (t_idx, t) in problem.teachers.iter().enumerate() {
            for (p_idx, p) in panels.iter().enumerate() {
                let e_id = exam_ids[p.exam_idx];
                let g_id = grade_ids[p.grade_idx];
                let s_id = subject_ids[p.subject_idx];
                slot_eligible[t_idx][p_idx][0] =
                    is_teacher_eligible(problem, t.id, e_id, g_id, s_id, Role::Setter);
                slot_eligible[t_idx][p_idx][1] =
                    is_teacher_eligible(problem, t.id, e_id, g_id, s_id, Role::Reviewer);
            }
        }

        let mut panel_min_campuses = Vec::with_capacity(panels.len());
        for p in &panels {
            let sub = &effective_subjects[p.subject_idx];
            panel_min_campuses.push(sub.min_campuses);
        }

        let mut teacher_available_exams = vec![0usize; num_teachers];
        for (t_idx, t) in problem.teachers.iter().enumerate() {
            teacher_available_exams[t_idx] = problem
                .exams
                .iter()
                .filter(|e| {
                    !problem
                        .unavailabilities
                        .iter()
                        .any(|u| u.teacher_id == t.id && u.exam_id == e.id)
                })
                .count();
        }

        // Initialize counters
        let mut state = Self {
            teacher_ids,
            teacher_map,
            exam_ids,
            grade_ids,
            subject_ids,
            subject_map,
            campuses,
            quotas,
            bounds,
            reviewer_eligible,
            qualified_grades,
            unavailabilities,
            forbids_setter,
            forbids_reviewer,
            panel_min_campuses,
            max_tasks_per_exam,
            max_setters_per_exam,
            slot_eligible,
            rule_enabled,
            rule_weights,
            s1_max_reviews,
            reviewer_capable,
            s2_rho,
            both_roles_capable,
            forced_tasks,
            forced_setters,
            forced_reviewers,
            forced_exam_tasks,
            forced_exam_setters,
            has_quota_override,
            panels,
            teacher_count: vec![0; num_teachers],
            teacher_setters: vec![0; num_teachers],
            teacher_reviewers: vec![0; num_teachers],
            teacher_exam_tasks: vec![vec![0; num_exams]; num_teachers],
            teacher_exam_setters: vec![vec![0; num_exams]; num_teachers],
            teacher_exam_role: vec![vec![None; num_exams]; num_teachers],
            teacher_grade_counts: vec![vec![0; num_grades]; num_teachers],
            teacher_distinct_grades: vec![0; num_teachers],
            setter_pairs: vec![vec![0; num_teachers]; num_teachers],
            review_relations: vec![vec![0; num_teachers]; num_teachers],
            reviewer_subject_competent,
            teacher_subject_reviews: vec![vec![0; num_subjects]; num_teachers],
            teacher_available_exams,
            sorted_exam_indices,
            current_units: [0.0; 10],
            current_penalty: 0.0,
        };

        state.rebuild_counters_and_full_eval();
        state
    }

    #[inline]
    #[must_use]
    pub fn find_panel_idx(&self, e_idx: usize, g_idx: usize, s_idx: usize) -> Option<usize> {
        self.panels
            .iter()
            .position(|p| p.exam_idx == e_idx && p.grade_idx == g_idx && p.subject_idx == s_idx)
    }

    /// Fully rebuilds dynamic counters from panels and calculates full score.
    pub fn rebuild_counters_and_full_eval(&mut self) {
        let num_teachers = self.teacher_ids.len();
        let num_exams = self.exam_ids.len();
        let num_grades = self.grade_ids.len();
        let num_subjects = self.subject_ids.len();

        self.teacher_count = vec![0; num_teachers];
        self.teacher_setters = vec![0; num_teachers];
        self.teacher_reviewers = vec![0; num_teachers];
        self.teacher_exam_tasks = vec![vec![0; num_exams]; num_teachers];
        self.teacher_exam_setters = vec![vec![0; num_exams]; num_teachers];
        self.teacher_exam_role = vec![vec![None; num_exams]; num_teachers];
        self.teacher_grade_counts = vec![vec![0; num_grades]; num_teachers];
        self.teacher_distinct_grades = vec![0; num_teachers];
        self.setter_pairs = vec![vec![0; num_teachers]; num_teachers];
        self.review_relations = vec![vec![0; num_teachers]; num_teachers];
        self.teacher_subject_reviews = vec![vec![0; num_subjects]; num_teachers];

        for panel in &self.panels {
            let e = panel.exam_idx;
            let g = panel.grade_idx;
            let s = panel.subject_idx;
            let s1 = panel.setter1;
            let s2 = panel.setter2;
            let rev = panel.reviewer;

            if s1 != usize::MAX {
                self.teacher_count[s1] += 1;
                self.teacher_setters[s1] += 1;
                self.teacher_exam_tasks[s1][e] += 1;
                self.teacher_exam_setters[s1][e] += 1;
                self.teacher_exam_role[s1][e] = Some(Role::Setter);
                if self.teacher_grade_counts[s1][g] == 0 {
                    self.teacher_distinct_grades[s1] += 1;
                }
                self.teacher_grade_counts[s1][g] += 1;
            }

            if s2 != usize::MAX {
                self.teacher_count[s2] += 1;
                self.teacher_setters[s2] += 1;
                self.teacher_exam_tasks[s2][e] += 1;
                self.teacher_exam_setters[s2][e] += 1;
                self.teacher_exam_role[s2][e] = Some(Role::Setter);
                if self.teacher_grade_counts[s2][g] == 0 {
                    self.teacher_distinct_grades[s2] += 1;
                }
                self.teacher_grade_counts[s2][g] += 1;
            }

            if rev != usize::MAX {
                self.teacher_count[rev] += 1;
                self.teacher_reviewers[rev] += 1;
                self.teacher_exam_tasks[rev][e] += 1;
                self.teacher_subject_reviews[rev][s] += 1;
                self.teacher_exam_role[rev][e] = Some(Role::Reviewer);
                if self.teacher_grade_counts[rev][g] == 0 {
                    self.teacher_distinct_grades[rev] += 1;
                }
                self.teacher_grade_counts[rev][g] += 1;
            }

            if s1 != usize::MAX && s2 != usize::MAX {
                let (u, v) = if s1 < s2 { (s1, s2) } else { (s2, s1) };
                self.setter_pairs[u][v] += 1;
            }

            if rev != usize::MAX {
                if s1 != usize::MAX {
                    self.review_relations[rev][s1] += 1;
                }
                if s2 != usize::MAX {
                    self.review_relations[rev][s2] += 1;
                }
            }
        }

        // Full score computation
        self.current_units = [0.0; 10];

        // S1: Reviewer count
        for t in 0..num_teachers {
            self.current_units[0] += self.eval_teacher_s1(t);
        }

        // S2: Role balance
        for t in 0..num_teachers {
            self.current_units[1] += self.eval_teacher_s2(t);
        }

        // S3: Independent reviewer
        for panel in &self.panels {
            self.current_units[2] += self.eval_panel_s3(panel);
        }

        // S4: Repeated setter pair
        for u in 0..num_teachers {
            for v in (u + 1)..num_teachers {
                self.current_units[3] += self.eval_setter_pair_s4(u, v);
            }
        }

        // S5: Repeated review relation
        for r in 0..num_teachers {
            for s in 0..num_teachers {
                self.current_units[4] += self.eval_review_rel_s5(r, s);
            }
        }

        // S6: Consecutive setting
        for t in 0..num_teachers {
            self.current_units[5] += self.eval_teacher_s6(t);
        }

        // S7: Grade rotation
        for t in 0..num_teachers {
            self.current_units[6] += self.eval_teacher_s7(t);
        }

        // S8: Load balance
        for t in 0..num_teachers {
            self.current_units[7] += self.eval_teacher_s8(t);
        }

        // S9: Exam crowding
        for t in 0..num_teachers {
            self.current_units[8] += self.eval_teacher_s9(t);
        }

        // S10: Review subject missing
        for t in 0..num_teachers {
            self.current_units[9] += self.eval_teacher_s10(t);
        }

        self.recompute_penalty();
    }

    #[inline]
    fn recompute_penalty(&mut self) {
        let mut total = 0.0;
        for i in 0..10 {
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
            let sid = self.subject_ids[panel.subject_idx];

            if panel.setter1 != usize::MAX {
                let mut a1 = Assignment::new(
                    eid,
                    gid,
                    sid,
                    self.teacher_ids[panel.setter1],
                    Role::Setter,
                    0,
                );
                a1.plan_id = plan_id;
                list.push(a1);
            }

            if panel.setter2 != usize::MAX {
                let mut a2 = Assignment::new(
                    eid,
                    gid,
                    sid,
                    self.teacher_ids[panel.setter2],
                    Role::Setter,
                    1,
                );
                a2.plan_id = plan_id;
                list.push(a2);
            }

            if panel.reviewer != usize::MAX {
                let mut a3 = Assignment::new(
                    eid,
                    gid,
                    sid,
                    self.teacher_ids[panel.reviewer],
                    Role::Reviewer,
                    0,
                );
                a3.plan_id = plan_id;
                list.push(a3);
            }
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
