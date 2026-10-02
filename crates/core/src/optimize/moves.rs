//! Local search move operators and candidate generation.
//!
//! Defines M1..M4 moves that strictly preserve hard constraints H1–H7:
//! - M1 Replace: swap panel slot with qualified teacher.
//! - M2 IntraExamSwap: swap two teachers between panels in the same exam.
//! - M3 CrossExamSwap: swap two teachers between panels in different exams.
//! - M4 RoleSwap: swap setter and reviewer within the same panel.

use super::state::{DensePanel, IncrementalState, SlotRole};
use crate::domain::Role;
use rand_chacha::ChaCha8Rng;
use rand_core::RngCore;

/// A proposed local search move.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LocalMove {
    Replace {
        panel_idx: usize,
        slot: SlotRole,
        old_t: usize,
        new_t: usize,
    },
    IntraExamSwap {
        panel1_idx: usize,
        slot1: SlotRole,
        t1: usize,
        panel2_idx: usize,
        slot2: SlotRole,
        t2: usize,
    },
    CrossExamSwap {
        panel1_idx: usize,
        slot1: SlotRole,
        t1: usize,
        panel2_idx: usize,
        slot2: SlotRole,
        t2: usize,
    },
    RoleSwap {
        panel_idx: usize,
        setter_slot: SlotRole,
        setter_t: usize,
        reviewer_t: usize,
    },
}

impl IncrementalState {
    /// Evaluates S1 cost for a single teacher.
    #[inline]
    pub fn eval_teacher_s1(&self, t: usize) -> f64 {
        if !self.reviewer_capable[t] {
            return 0.0;
        }
        let r = self.teacher_reviewers[t];
        if r == 0 {
            1.0
        } else if r > self.s1_max_reviews {
            (r - self.s1_max_reviews) as f64
        } else {
            0.0
        }
    }

    /// Evaluates S2 cost for a single teacher (distance to [floor(c'*rho), ceil(c'*rho)]).
    #[inline]
    pub fn eval_teacher_s2(&self, t: usize) -> f64 {
        if !self.both_roles_capable[t] {
            return 0.0;
        }
        let c_prime = self.teacher_count[t].saturating_sub(self.forced_tasks[t]);
        if c_prime >= 2 {
            let r_prime = self.teacher_reviewers[t].saturating_sub(self.forced_reviewers[t]);
            let target = (c_prime as f64) * self.s2_rho;
            let lo = target.floor() as usize;
            let hi = target.ceil() as usize;
            if r_prime < lo {
                (lo - r_prime) as f64
            } else if r_prime > hi {
                (r_prime - hi) as f64
            } else {
                0.0
            }
        } else {
            0.0
        }
    }

    /// Evaluates S3 cost for a single panel.
    #[inline]
    pub fn eval_panel_s3(&self, panel: &DensePanel) -> f64 {
        if panel.reviewer == usize::MAX {
            return 0.0;
        }
        let r_camp = self.campuses[panel.reviewer];
        let mut u = 0.0;
        if panel.setter1 != usize::MAX && self.campuses[panel.setter1] == r_camp {
            u += 1.0;
        }
        if panel.setter2 != usize::MAX && self.campuses[panel.setter2] == r_camp {
            u += 1.0;
        }
        u
    }

    /// Evaluates S4 cost for a single setter pair.
    #[inline]
    pub fn eval_setter_pair_s4(&self, u: usize, v: usize) -> f64 {
        let (min_t, max_t) = if u < v { (u, v) } else { (v, u) };
        let c = self.setter_pairs[min_t][max_t];
        if c > 1 {
            (c - 1) as f64
        } else {
            0.0
        }
    }

    /// Evaluates S5 cost for a single review relation.
    #[inline]
    pub fn eval_review_rel_s5(&self, rev: usize, setter: usize) -> f64 {
        let c = self.review_relations[rev][setter];
        if c > 1 {
            (c - 1) as f64
        } else {
            0.0
        }
    }

    /// Evaluates S6 cost for a single teacher.
    #[inline]
    pub fn eval_teacher_s6(&self, t: usize) -> f64 {
        if self.sorted_exam_indices.len() < 2 {
            return 0.0;
        }
        let mut u = 0.0;
        for i in 0..(self.sorted_exam_indices.len() - 1) {
            let e1 = self.sorted_exam_indices[i];
            let e2 = self.sorted_exam_indices[i + 1];
            let s1 =
                self.teacher_exam_setters[t][e1].saturating_sub(self.forced_exam_setters[t][e1]);
            let s2 =
                self.teacher_exam_setters[t][e2].saturating_sub(self.forced_exam_setters[t][e2]);
            if s1 > 0 && s2 > 0 {
                u += 1.0;
            }
        }
        u
    }

    /// Evaluates S7 cost for a single teacher.
    #[inline]
    pub fn eval_teacher_s7(&self, t: usize) -> f64 {
        let qualified_count = self.qualified_grades[t].iter().filter(|&&q| q).count();
        if qualified_count >= 2 {
            let c_prime = self.teacher_count[t].saturating_sub(self.forced_tasks[t]);
            let target = c_prime.min(qualified_count);
            let assigned = self.teacher_distinct_grades[t];
            if assigned < target {
                (target - assigned) as f64
            } else {
                0.0
            }
        } else {
            0.0
        }
    }

    /// Evaluates S8 cost for a single teacher.
    #[inline]
    pub fn eval_teacher_s8(&self, t: usize) -> f64 {
        if self.has_quota_override[t] {
            return 0.0;
        }
        let f_t = self.forced_tasks[t];
        let c_prime = self.teacher_count[t].saturating_sub(f_t);
        let q_t = self.quotas[t];
        let q_prime = (q_t - f_t as f64).max(0.0);
        let diff = c_prime as f64 - q_prime;
        diff * diff
    }

    /// Evaluates S9 cost for a single teacher.
    #[inline]
    pub fn eval_teacher_s9(&self, t: usize) -> f64 {
        let mut u = 0.0;
        for e in 0..self.exam_ids.len() {
            let tasks = self.teacher_exam_tasks[t][e].saturating_sub(self.forced_exam_tasks[t][e]);
            if tasks > 1 {
                u += (tasks - 1) as f64;
            }
        }
        u
    }

    /// Evaluates S10 cost for a single teacher.
    #[inline]
    pub fn eval_teacher_s10(&self, t: usize) -> f64 {
        if !self.reviewer_capable[t] {
            return 0.0;
        }
        let mut u = 0.0;
        for s in 0..self.subject_ids.len() {
            if self.reviewer_subject_competent[t][s] && self.teacher_subject_reviews[t][s] == 0 {
                u += 1.0;
            }
        }
        u
    }

    /// Evaluates the total weighted penalty for the given set of teachers, panels, pairs, relations.
    fn eval_sub_penalty(
        &self,
        teachers: &[usize],
        panels: &[usize],
        pairs: &[(usize, usize)],
        relations: &[(usize, usize)],
    ) -> f64 {
        let mut total = 0.0;

        // Teacher terms: S1, S2, S6, S7, S8, S9, S10
        for &t in teachers {
            if self.rule_enabled[0] {
                total += self.rule_weights[0] * self.eval_teacher_s1(t);
            }
            if self.rule_enabled[1] {
                total += self.rule_weights[1] * self.eval_teacher_s2(t);
            }
            if self.rule_enabled[5] {
                total += self.rule_weights[5] * self.eval_teacher_s6(t);
            }
            if self.rule_enabled[6] {
                total += self.rule_weights[6] * self.eval_teacher_s7(t);
            }
            if self.rule_enabled[7] {
                total += self.rule_weights[7] * self.eval_teacher_s8(t);
            }
            if self.rule_enabled[8] {
                total += self.rule_weights[8] * self.eval_teacher_s9(t);
            }
            if self.rule_enabled[9] {
                total += self.rule_weights[9] * self.eval_teacher_s10(t);
            }
        }

        // Panel terms: S3
        if self.rule_enabled[2] {
            for &p_idx in panels {
                total += self.rule_weights[2] * self.eval_panel_s3(&self.panels[p_idx]);
            }
        }

        // Pair terms: S4
        if self.rule_enabled[3] {
            for &(u, v) in pairs {
                total += self.rule_weights[3] * self.eval_setter_pair_s4(u, v);
            }
        }

        // Relation terms: S5
        if self.rule_enabled[4] {
            for &(r, s) in relations {
                total += self.rule_weights[4] * self.eval_review_rel_s5(r, s);
            }
        }

        total
    }

    fn remove_panel_relations(&mut self, p_idx: usize) {
        let p = &self.panels[p_idx];
        let s1 = p.setter1;
        let s2 = p.setter2;
        let rev = p.reviewer;
        if s1 != usize::MAX && s2 != usize::MAX {
            let (u, v) = if s1 < s2 { (s1, s2) } else { (s2, s1) };
            self.setter_pairs[u][v] -= 1;
        }
        if rev != usize::MAX {
            if s1 != usize::MAX {
                self.review_relations[rev][s1] -= 1;
            }
            if s2 != usize::MAX {
                self.review_relations[rev][s2] -= 1;
            }
        }
    }

    fn add_panel_relations(&mut self, p_idx: usize) {
        let p = &self.panels[p_idx];
        let s1 = p.setter1;
        let s2 = p.setter2;
        let rev = p.reviewer;
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

    fn remove_teacher_counters(&mut self, t: usize, role: Role, e: usize, g: usize, s: usize) {
        self.teacher_count[t] -= 1;
        match role {
            Role::Setter => {
                self.teacher_setters[t] -= 1;
                self.teacher_exam_setters[t][e] -= 1;
            }
            Role::Reviewer => {
                self.teacher_reviewers[t] -= 1;
                self.teacher_subject_reviews[t][s] -= 1;
            }
        }
        self.teacher_exam_tasks[t][e] -= 1;
        self.teacher_grade_counts[t][g] -= 1;
        if self.teacher_grade_counts[t][g] == 0 {
            self.teacher_distinct_grades[t] -= 1;
        }
    }

    fn add_teacher_counters(&mut self, t: usize, role: Role, e: usize, g: usize, s: usize) {
        self.teacher_count[t] += 1;
        match role {
            Role::Setter => {
                self.teacher_setters[t] += 1;
                self.teacher_exam_setters[t][e] += 1;
            }
            Role::Reviewer => {
                self.teacher_reviewers[t] += 1;
                self.teacher_subject_reviews[t][s] += 1;
            }
        }
        self.teacher_exam_tasks[t][e] += 1;
        if self.teacher_grade_counts[t][g] == 0 {
            self.teacher_distinct_grades[t] += 1;
        }
        self.teacher_grade_counts[t][g] += 1;
    }

    /// Applies a slot change internal helper.
    pub fn apply_slot_change(&mut self, p_idx: usize, slot: SlotRole, old_t: usize, new_t: usize) {
        let e = self.panels[p_idx].exam_idx;
        let g = self.panels[p_idx].grade_idx;
        let s = self.panels[p_idx].subject_idx;
        let role = slot.to_role();

        self.remove_panel_relations(p_idx);
        self.remove_teacher_counters(old_t, role, e, g, s);

        self.panels[p_idx].set_slot(slot, new_t);

        self.add_teacher_counters(new_t, role, e, g, s);
        self.add_panel_relations(p_idx);
    }

    /// Evaluates the delta in total penalty for a move, applies it, and returns the delta.
    /// If the caller decides not to accept, they must call `revert_move(m)`.
    pub fn try_apply_move(&mut self, m: LocalMove) -> f64 {
        let aff = self.affected_entities(m);

        let old_penalty =
            self.eval_sub_penalty(&aff.teachers, &aff.panels, &aff.pairs, &aff.relations);

        self.apply_raw(m);

        let new_penalty =
            self.eval_sub_penalty(&aff.teachers, &aff.panels, &aff.pairs, &aff.relations);
        let delta = new_penalty - old_penalty;
        self.current_penalty += delta;
        delta
    }

    /// Reverts a previously applied move.
    pub fn revert_move(&mut self, m: LocalMove, delta: f64) {
        let inv = match m {
            LocalMove::Replace {
                panel_idx,
                slot,
                old_t,
                new_t,
            } => LocalMove::Replace {
                panel_idx,
                slot,
                old_t: new_t,
                new_t: old_t,
            },
            LocalMove::IntraExamSwap {
                panel1_idx,
                slot1,
                t1,
                panel2_idx,
                slot2,
                t2,
            } => LocalMove::IntraExamSwap {
                panel1_idx,
                slot1,
                t1: t2,
                panel2_idx,
                slot2,
                t2: t1,
            },
            LocalMove::CrossExamSwap {
                panel1_idx,
                slot1,
                t1,
                panel2_idx,
                slot2,
                t2,
            } => LocalMove::CrossExamSwap {
                panel1_idx,
                slot1,
                t1: t2,
                panel2_idx,
                slot2,
                t2: t1,
            },
            LocalMove::RoleSwap {
                panel_idx,
                setter_slot,
                setter_t,
                reviewer_t,
            } => LocalMove::RoleSwap {
                panel_idx,
                setter_slot,
                setter_t: reviewer_t,
                reviewer_t: setter_t,
            },
        };

        self.apply_raw(inv);
        self.current_penalty -= delta;
    }

    fn apply_raw(&mut self, m: LocalMove) {
        match m {
            LocalMove::Replace {
                panel_idx,
                slot,
                old_t,
                new_t,
            } => {
                self.apply_slot_change(panel_idx, slot, old_t, new_t);
            }
            LocalMove::IntraExamSwap {
                panel1_idx,
                slot1,
                t1,
                panel2_idx,
                slot2,
                t2,
            }
            | LocalMove::CrossExamSwap {
                panel1_idx,
                slot1,
                t1,
                panel2_idx,
                slot2,
                t2,
            } => {
                let e1 = self.panels[panel1_idx].exam_idx;
                let g1 = self.panels[panel1_idx].grade_idx;
                let s1 = self.panels[panel1_idx].subject_idx;
                let r1 = slot1.to_role();

                let e2 = self.panels[panel2_idx].exam_idx;
                let g2 = self.panels[panel2_idx].grade_idx;
                let s2 = self.panels[panel2_idx].subject_idx;
                let r2 = slot2.to_role();

                self.remove_panel_relations(panel1_idx);
                if panel1_idx != panel2_idx {
                    self.remove_panel_relations(panel2_idx);
                }
                self.remove_teacher_counters(t1, r1, e1, g1, s1);
                self.remove_teacher_counters(t2, r2, e2, g2, s2);

                self.panels[panel1_idx].set_slot(slot1, t2);
                self.panels[panel2_idx].set_slot(slot2, t1);

                self.add_teacher_counters(t2, r1, e1, g1, s1);
                self.add_teacher_counters(t1, r2, e2, g2, s2);
                self.add_panel_relations(panel1_idx);
                if panel1_idx != panel2_idx {
                    self.add_panel_relations(panel2_idx);
                }
            }
            LocalMove::RoleSwap {
                panel_idx,
                setter_slot,
                setter_t,
                reviewer_t,
            } => {
                let e = self.panels[panel_idx].exam_idx;
                let g = self.panels[panel_idx].grade_idx;
                let s = self.panels[panel_idx].subject_idx;

                self.remove_panel_relations(panel_idx);
                self.remove_teacher_counters(setter_t, Role::Setter, e, g, s);
                self.remove_teacher_counters(reviewer_t, Role::Reviewer, e, g, s);

                self.panels[panel_idx].set_slot(setter_slot, reviewer_t);
                self.panels[panel_idx].set_slot(SlotRole::Reviewer, setter_t);

                self.add_teacher_counters(reviewer_t, Role::Setter, e, g, s);
                self.add_teacher_counters(setter_t, Role::Reviewer, e, g, s);
                self.add_panel_relations(panel_idx);
            }
        }
    }
}

/// Set of entities affected by a local search move.
#[derive(Debug, Default)]
struct AffectedEntities {
    teachers: Vec<usize>,
    panels: Vec<usize>,
    pairs: Vec<(usize, usize)>,
    relations: Vec<(usize, usize)>,
}

impl IncrementalState {
    /// Gathers distinct affected teachers, panels, pairs, and relations for a move.
    fn affected_entities(&self, m: LocalMove) -> AffectedEntities {
        let mut teachers = Vec::new();
        let mut panels = Vec::new();
        let mut pairs = Vec::new();
        let mut relations = Vec::new();

        let add_panel_terms = |p: &DensePanel,
                               pairs: &mut Vec<(usize, usize)>,
                               relations: &mut Vec<(usize, usize)>| {
            let s1 = p.setter1;
            let s2 = p.setter2;
            let rev = p.reviewer;
            if s1 != usize::MAX && s2 != usize::MAX {
                let (u, v) = if s1 < s2 { (s1, s2) } else { (s2, s1) };
                if !pairs.contains(&(u, v)) {
                    pairs.push((u, v));
                }
            }
            if rev != usize::MAX {
                if s1 != usize::MAX && !relations.contains(&(rev, s1)) {
                    relations.push((rev, s1));
                }
                if s2 != usize::MAX && !relations.contains(&(rev, s2)) {
                    relations.push((rev, s2));
                }
            }
        };

        match m {
            LocalMove::Replace {
                panel_idx,
                slot,
                old_t,
                new_t,
            } => {
                teachers.push(old_t);
                teachers.push(new_t);
                panels.push(panel_idx);
                add_panel_terms(&self.panels[panel_idx], &mut pairs, &mut relations);

                let mut prospective = self.panels[panel_idx].clone();
                prospective.set_slot(slot, new_t);
                add_panel_terms(&prospective, &mut pairs, &mut relations);
            }
            LocalMove::IntraExamSwap {
                panel1_idx,
                slot1,
                t1: _,
                panel2_idx,
                slot2,
                t2: _,
            }
            | LocalMove::CrossExamSwap {
                panel1_idx,
                slot1,
                t1: _,
                panel2_idx,
                slot2,
                t2: _,
            } => {
                let t1 = self.panels[panel1_idx].get_slot(slot1);
                let t2 = self.panels[panel2_idx].get_slot(slot2);
                teachers.push(t1);
                teachers.push(t2);
                panels.push(panel1_idx);
                add_panel_terms(&self.panels[panel1_idx], &mut pairs, &mut relations);
                if !panels.contains(&panel2_idx) {
                    panels.push(panel2_idx);
                }
                add_panel_terms(&self.panels[panel2_idx], &mut pairs, &mut relations);

                let mut prospective1 = self.panels[panel1_idx].clone();
                let mut prospective2 = self.panels[panel2_idx].clone();
                prospective1.set_slot(slot1, t2);
                prospective2.set_slot(slot2, t1);
                add_panel_terms(&prospective1, &mut pairs, &mut relations);
                add_panel_terms(&prospective2, &mut pairs, &mut relations);
            }
            LocalMove::RoleSwap {
                panel_idx,
                setter_slot,
                setter_t,
                reviewer_t,
            } => {
                teachers.push(setter_t);
                teachers.push(reviewer_t);
                panels.push(panel_idx);
                add_panel_terms(&self.panels[panel_idx], &mut pairs, &mut relations);

                let mut prospective = self.panels[panel_idx].clone();
                prospective.set_slot(setter_slot, reviewer_t);
                prospective.set_slot(SlotRole::Reviewer, setter_t);
                add_panel_terms(&prospective, &mut pairs, &mut relations);
            }
        }

        teachers.sort_unstable();
        teachers.dedup();

        AffectedEntities {
            teachers,
            panels,
            pairs,
            relations,
        }
    }

    /// Checks if a panel with proposed teachers satisfies H3 (at least subject.min_campuses distinct campuses).
    #[inline]
    pub fn check_h3_panel(&self, p_idx: usize, s1: usize, s2: usize, rev: usize) -> bool {
        let min_c = self.panel_min_campuses[p_idx] as usize;
        if min_c <= 1 {
            return true;
        }
        let mut seen = 0u64;
        let mut count = 0;
        for &t in &[s1, s2, rev] {
            if t != usize::MAX {
                let bit = 1u64 << (self.campuses[t] % 64);
                if (seen & bit) == 0 {
                    seen |= bit;
                    count += 1;
                }
            }
        }
        count >= min_c
    }

    /// Generates a valid candidate move using random selection.
    pub fn sample_candidate_move(&self, rng: &mut ChaCha8Rng) -> Option<LocalMove> {
        let num_panels = self.panels.len();
        let num_teachers = self.teacher_ids.len();

        if num_panels == 0 || num_teachers == 0 {
            return None;
        }

        // Try up to 50 attempts to find a valid move
        for _ in 0..50 {
            let move_kind = rng.next_u32() % 100;
            if move_kind < 35 {
                // -------------------------------------------------------------
                // M1: Replace
                // -------------------------------------------------------------
                let p_idx = (rng.next_u32() as usize) % num_panels;
                let panel = &self.panels[p_idx];
                let slot = match rng.next_u32() % 3 {
                    0 => SlotRole::Setter1,
                    1 => SlotRole::Setter2,
                    _ => SlotRole::Reviewer,
                };
                if panel.is_pinned(slot) {
                    continue;
                }
                let old_t = panel.get_slot(slot);
                let new_t = (rng.next_u32() as usize) % num_teachers;
                if new_t == old_t {
                    continue;
                }

                // H1: distinct teachers per panel
                if new_t == panel.setter1 || new_t == panel.setter2 || new_t == panel.reviewer {
                    continue;
                }

                let role = slot.to_role();
                let role_idx = if role == Role::Setter { 0 } else { 1 };

                // H2, H5, H6 eligibility
                if !self.slot_eligible[new_t][p_idx][role_idx] {
                    continue;
                }

                let e = panel.exam_idx;

                // H4 task limits
                if self.teacher_exam_tasks[new_t][e] + 1 > self.max_tasks_per_exam[new_t][e] {
                    continue;
                }
                if role == Role::Setter
                    && self.teacher_exam_setters[new_t][e] + 1 > self.max_setters_per_exam[new_t][e]
                {
                    continue;
                }

                // H7: bounds
                if self.teacher_count[new_t] + 1 > self.bounds[new_t].1 {
                    continue;
                }
                if self.teacher_count[old_t] <= self.bounds[old_t].0 {
                    continue;
                }

                // H3: campus diversity in panel
                let (t1, t2, t3) = match slot {
                    SlotRole::Setter1 => (new_t, panel.setter2, panel.reviewer),
                    SlotRole::Setter2 => (panel.setter1, new_t, panel.reviewer),
                    SlotRole::Reviewer => (panel.setter1, panel.setter2, new_t),
                };
                if !self.check_h3_panel(p_idx, t1, t2, t3) {
                    continue;
                }

                return Some(LocalMove::Replace {
                    panel_idx: p_idx,
                    slot,
                    old_t,
                    new_t,
                });
            } else if move_kind < 60 {
                // -------------------------------------------------------------
                // M2: Intra-Exam Swap (same exam, different panels)
                // -------------------------------------------------------------
                let p1_idx = (rng.next_u32() as usize) % num_panels;
                let e = self.panels[p1_idx].exam_idx;

                // Pick another panel in same exam
                let same_exam_panels: Vec<usize> = (0..num_panels)
                    .filter(|&idx| idx != p1_idx && self.panels[idx].exam_idx == e)
                    .collect();
                if same_exam_panels.is_empty() {
                    continue;
                }
                let p2_idx = same_exam_panels[(rng.next_u32() as usize) % same_exam_panels.len()];

                let p1 = &self.panels[p1_idx];
                let p2 = &self.panels[p2_idx];

                let slot1 = match rng.next_u32() % 3 {
                    0 => SlotRole::Setter1,
                    1 => SlotRole::Setter2,
                    _ => SlotRole::Reviewer,
                };
                let slot2 = match rng.next_u32() % 3 {
                    0 => SlotRole::Setter1,
                    1 => SlotRole::Setter2,
                    _ => SlotRole::Reviewer,
                };
                if p1.is_pinned(slot1) || p2.is_pinned(slot2) {
                    continue;
                }
                let t1 = p1.get_slot(slot1);
                let t2 = p2.get_slot(slot2);
                if t1 == t2 || t1 == usize::MAX || t2 == usize::MAX {
                    continue;
                }

                // H1 distinct teachers check for p1 and p2
                let (p1_s1, p1_s2, p1_rev) = match slot1 {
                    SlotRole::Setter1 => (t2, p1.setter2, p1.reviewer),
                    SlotRole::Setter2 => (p1.setter1, t2, p1.reviewer),
                    SlotRole::Reviewer => (p1.setter1, p1.setter2, t2),
                };
                if (p1_s1 != usize::MAX && p1_s1 == p1_s2)
                    || (p1_s1 != usize::MAX && p1_s1 == p1_rev)
                    || (p1_s2 != usize::MAX && p1_s2 == p1_rev)
                {
                    continue;
                }

                let (p2_s1, p2_s2, p2_rev) = match slot2 {
                    SlotRole::Setter1 => (t1, p2.setter2, p2.reviewer),
                    SlotRole::Setter2 => (p2.setter1, t1, p2.reviewer),
                    SlotRole::Reviewer => (p2.setter1, p2.setter2, t1),
                };
                if (p2_s1 != usize::MAX && p2_s1 == p2_s2)
                    || (p2_s1 != usize::MAX && p2_s1 == p2_rev)
                    || (p2_s2 != usize::MAX && p2_s2 == p2_rev)
                {
                    continue;
                }

                let r1 = slot1.to_role();
                let r2 = slot2.to_role();
                let r1_idx = if r1 == Role::Setter { 0 } else { 1 };
                let r2_idx = if r2 == Role::Setter { 0 } else { 1 };

                // H2, H5, H6 eligibility
                if !self.slot_eligible[t1][p2_idx][r2_idx]
                    || !self.slot_eligible[t2][p1_idx][r1_idx]
                {
                    continue;
                }

                // H4 setter limit check if roles differ
                if r1 != r2 {
                    if r2 == Role::Setter
                        && self.teacher_exam_setters[t1][e] + 1 > self.max_setters_per_exam[t1][e]
                    {
                        continue;
                    }
                    if r1 == Role::Setter
                        && self.teacher_exam_setters[t2][e] + 1 > self.max_setters_per_exam[t2][e]
                    {
                        continue;
                    }
                }

                // H3 campus diversity
                if !self.check_h3_panel(p1_idx, p1_s1, p1_s2, p1_rev)
                    || !self.check_h3_panel(p2_idx, p2_s1, p2_s2, p2_rev)
                {
                    continue;
                }

                return Some(LocalMove::IntraExamSwap {
                    panel1_idx: p1_idx,
                    slot1,
                    t1,
                    panel2_idx: p2_idx,
                    slot2,
                    t2,
                });
            } else if move_kind < 85 {
                // -------------------------------------------------------------
                // M3: Cross-Exam Swap (different exams)
                // -------------------------------------------------------------
                let p1_idx = (rng.next_u32() as usize) % num_panels;
                let e1 = self.panels[p1_idx].exam_idx;
                let diff_exam_panels: Vec<usize> = (0..num_panels)
                    .filter(|&idx| self.panels[idx].exam_idx != e1)
                    .collect();
                if diff_exam_panels.is_empty() {
                    continue;
                }
                let p2_idx = diff_exam_panels[(rng.next_u32() as usize) % diff_exam_panels.len()];
                let e2 = self.panels[p2_idx].exam_idx;

                let p1 = &self.panels[p1_idx];
                let p2 = &self.panels[p2_idx];

                let slot1 = match rng.next_u32() % 3 {
                    0 => SlotRole::Setter1,
                    1 => SlotRole::Setter2,
                    _ => SlotRole::Reviewer,
                };
                let slot2 = match rng.next_u32() % 3 {
                    0 => SlotRole::Setter1,
                    1 => SlotRole::Setter2,
                    _ => SlotRole::Reviewer,
                };
                if p1.is_pinned(slot1) || p2.is_pinned(slot2) {
                    continue;
                }
                let t1 = p1.get_slot(slot1);
                let t2 = p2.get_slot(slot2);
                if t1 == t2 || t1 == usize::MAX || t2 == usize::MAX {
                    continue;
                }

                // H1 distinct teachers check for p1 and p2
                let (p1_s1, p1_s2, p1_rev) = match slot1 {
                    SlotRole::Setter1 => (t2, p1.setter2, p1.reviewer),
                    SlotRole::Setter2 => (p1.setter1, t2, p1.reviewer),
                    SlotRole::Reviewer => (p1.setter1, p1.setter2, t2),
                };
                if (p1_s1 != usize::MAX && p1_s1 == p1_s2)
                    || (p1_s1 != usize::MAX && p1_s1 == p1_rev)
                    || (p1_s2 != usize::MAX && p1_s2 == p1_rev)
                {
                    continue;
                }

                let (p2_s1, p2_s2, p2_rev) = match slot2 {
                    SlotRole::Setter1 => (t1, p2.setter2, p2.reviewer),
                    SlotRole::Setter2 => (p2.setter1, t1, p2.reviewer),
                    SlotRole::Reviewer => (p2.setter1, p2.setter2, t1),
                };
                if (p2_s1 != usize::MAX && p2_s1 == p2_s2)
                    || (p2_s1 != usize::MAX && p2_s1 == p2_rev)
                    || (p2_s2 != usize::MAX && p2_s2 == p2_rev)
                {
                    continue;
                }

                let r1 = slot1.to_role();
                let r2 = slot2.to_role();
                let r1_idx = if r1 == Role::Setter { 0 } else { 1 };
                let r2_idx = if r2 == Role::Setter { 0 } else { 1 };

                // H2, H5, H6 eligibility
                if !self.slot_eligible[t1][p2_idx][r2_idx]
                    || !self.slot_eligible[t2][p1_idx][r1_idx]
                {
                    continue;
                }

                // H4 task limits:
                // t1 gains task in e2
                if self.teacher_exam_tasks[t1][e2] + 1 > self.max_tasks_per_exam[t1][e2] {
                    continue;
                }
                if r2 == Role::Setter
                    && self.teacher_exam_setters[t1][e2] + 1 > self.max_setters_per_exam[t1][e2]
                {
                    continue;
                }
                // t2 gains task in e1
                if self.teacher_exam_tasks[t2][e1] + 1 > self.max_tasks_per_exam[t2][e1] {
                    continue;
                }
                if r1 == Role::Setter
                    && self.teacher_exam_setters[t2][e1] + 1 > self.max_setters_per_exam[t2][e1]
                {
                    continue;
                }

                // H3 campus diversity
                if !self.check_h3_panel(p1_idx, p1_s1, p1_s2, p1_rev)
                    || !self.check_h3_panel(p2_idx, p2_s1, p2_s2, p2_rev)
                {
                    continue;
                }

                return Some(LocalMove::CrossExamSwap {
                    panel1_idx: p1_idx,
                    slot1,
                    t1,
                    panel2_idx: p2_idx,
                    slot2,
                    t2,
                });
            } else {
                // -------------------------------------------------------------
                // M4: Role Swap within panel
                // -------------------------------------------------------------
                let p_idx = (rng.next_u32() as usize) % num_panels;
                let panel = &self.panels[p_idx];
                if panel.is_pinned(SlotRole::Reviewer) {
                    continue;
                }
                let setter_slot = if (rng.next_u32() & 1) == 0 {
                    SlotRole::Setter1
                } else {
                    SlotRole::Setter2
                };
                if panel.is_pinned(setter_slot) {
                    continue;
                }
                let s_t = panel.get_slot(setter_slot);
                let r_t = panel.reviewer;
                if s_t == r_t || s_t == usize::MAX || r_t == usize::MAX {
                    continue;
                }

                let e = panel.exam_idx;

                // Eligibility
                if !self.slot_eligible[s_t][p_idx][1] || !self.slot_eligible[r_t][p_idx][0] {
                    continue;
                }

                // H4 setter limit check for r_t
                if self.teacher_exam_setters[r_t][e] + 1 > self.max_setters_per_exam[r_t][e] {
                    continue;
                }

                return Some(LocalMove::RoleSwap {
                    panel_idx: p_idx,
                    setter_slot,
                    setter_t: s_t,
                    reviewer_t: r_t,
                });
            }
        }

        None
    }
}
