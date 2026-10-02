-- 0005_subjects_and_competencies.sql
-- Multi-subject panels (VL / CN), competencies, per-exam task limits, forced assignments.

-- 1. Create subjects table
CREATE TABLE subjects (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    school_year_id INTEGER NOT NULL REFERENCES school_years(id) ON DELETE CASCADE,
    code TEXT NOT NULL,
    name TEXT NOT NULL,
    color TEXT NOT NULL,
    sort_order INTEGER NOT NULL,
    setters INTEGER NOT NULL DEFAULT 2,
    reviewers INTEGER NOT NULL DEFAULT 1,
    min_campuses INTEGER NOT NULL DEFAULT 2,
    UNIQUE(school_year_id, code)
);

CREATE INDEX idx_subjects_school_year ON subjects(school_year_id);

-- 2. Create teacher_competencies table
CREATE TABLE teacher_competencies (
    teacher_id INTEGER NOT NULL REFERENCES teachers(id) ON DELETE CASCADE,
    subject_id INTEGER NOT NULL REFERENCES subjects(id) ON DELETE CASCADE,
    role TEXT NOT NULL CHECK (role IN ('setter', 'reviewer')),
    grade_scope TEXT NOT NULL DEFAULT 'taught' CHECK (grade_scope IN ('taught', 'any')),
    PRIMARY KEY (teacher_id, subject_id, role)
);

CREATE INDEX idx_teacher_competencies_subject ON teacher_competencies(subject_id);

-- 3. Extend teachers table
ALTER TABLE teachers ADD COLUMN display_name TEXT;
ALTER TABLE teachers ADD COLUMN quota_override INTEGER;
ALTER TABLE teachers ADD COLUMN max_tasks_per_exam_override INTEGER;

-- 4. Backfill default subject 'CHUNG' for every existing school year
INSERT INTO subjects (school_year_id, code, name, color, sort_order, setters, reviewers, min_campuses)
SELECT id, 'CHUNG', 'Chung', 'palette-1', 1, 2, 1, 2 FROM school_years;

-- 5. Backfill teacher competencies for every existing teacher and subject
INSERT INTO teacher_competencies (teacher_id, subject_id, role, grade_scope)
SELECT t.id, s.id, 'setter', 'taught'
FROM teachers t
JOIN subjects s ON s.code = 'CHUNG';

INSERT INTO teacher_competencies (teacher_id, subject_id, role, grade_scope)
SELECT t.id, s.id, 'reviewer', 'taught'
FROM teachers t
JOIN subjects s ON s.code = 'CHUNG';

-- 6. Rebuild locks table with subject_id
CREATE TABLE locks_new (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    exam_id INTEGER NOT NULL REFERENCES exams(id) ON DELETE CASCADE,
    grade_id INTEGER NOT NULL REFERENCES grades(id) ON DELETE CASCADE,
    subject_id INTEGER NOT NULL REFERENCES subjects(id) ON DELETE CASCADE,
    teacher_id INTEGER NOT NULL REFERENCES teachers(id) ON DELETE CASCADE,
    role TEXT NULL CHECK (role IS NULL OR role IN ('setter', 'reviewer')),
    kind TEXT NOT NULL CHECK (kind IN ('pin', 'forbid'))
);

INSERT INTO locks_new (id, exam_id, grade_id, subject_id, teacher_id, role, kind)
SELECT l.id, l.exam_id, l.grade_id, s.id, l.teacher_id, l.role, l.kind
FROM locks l
JOIN exams e ON l.exam_id = e.id
JOIN subjects s ON s.school_year_id = e.school_year_id AND s.code = 'CHUNG';

DROP TABLE locks;
ALTER TABLE locks_new RENAME TO locks;

CREATE UNIQUE INDEX idx_locks_unique
ON locks(exam_id, grade_id, subject_id, teacher_id, kind, COALESCE(role, 'any'));
CREATE INDEX idx_locks_exam_grade ON locks(exam_id, grade_id, subject_id);

-- 7. Rebuild assignments table with subject_id and position
CREATE TABLE assignments_new (
    plan_id INTEGER NOT NULL REFERENCES plans(id) ON DELETE CASCADE,
    exam_id INTEGER NOT NULL REFERENCES exams(id) ON DELETE RESTRICT,
    grade_id INTEGER NOT NULL REFERENCES grades(id) ON DELETE RESTRICT,
    subject_id INTEGER NOT NULL REFERENCES subjects(id) ON DELETE RESTRICT,
    teacher_id INTEGER NOT NULL REFERENCES teachers(id) ON DELETE RESTRICT,
    role TEXT NOT NULL CHECK (role IN ('setter', 'reviewer')),
    position INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY(plan_id, exam_id, grade_id, subject_id, teacher_id)
);

INSERT INTO assignments_new (plan_id, exam_id, grade_id, subject_id, teacher_id, role, position)
SELECT a.plan_id, a.exam_id, a.grade_id, s.id, a.teacher_id, a.role,
       CASE WHEN a.role = 'setter' THEN
         (SELECT COUNT(*) FROM assignments a2
          WHERE a2.plan_id = a.plan_id AND a2.exam_id = a.exam_id AND a2.grade_id = a.grade_id
            AND a2.role = 'setter' AND a2.teacher_id < a.teacher_id)
       ELSE 0 END AS position
FROM assignments a
JOIN plans p ON a.plan_id = p.id
JOIN subjects s ON s.school_year_id = p.school_year_id AND s.code = 'CHUNG';

DROP TABLE assignments;
ALTER TABLE assignments_new RENAME TO assignments;

CREATE INDEX idx_assignments_plan ON assignments(plan_id);
