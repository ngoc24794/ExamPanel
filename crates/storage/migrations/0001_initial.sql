-- 0001_initial.sql
-- Initial schema for ExamPanel SQLite database

CREATE TABLE campuses (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    code TEXT UNIQUE NOT NULL,
    name TEXT NOT NULL,
    color TEXT NOT NULL
);

CREATE TABLE grades (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    code INTEGER UNIQUE NOT NULL,
    name TEXT NOT NULL,
    sort_order INTEGER NOT NULL
);

CREATE TABLE teachers (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    full_name TEXT NOT NULL,
    campus_id INTEGER NOT NULL REFERENCES campuses(id) ON DELETE RESTRICT,
    load_weight REAL NOT NULL DEFAULT 1.0 CHECK (load_weight >= 0.0 AND load_weight <= 1.0),
    active INTEGER NOT NULL DEFAULT 1 CHECK (active IN (0, 1)),
    note TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE school_years (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT UNIQUE NOT NULL,
    is_current INTEGER NOT NULL DEFAULT 0 CHECK (is_current IN (0, 1))
);

-- Partial unique index: only one school year can be current
CREATE UNIQUE INDEX idx_school_years_one_current
ON school_years(is_current)
WHERE is_current = 1;

CREATE TABLE exams (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    school_year_id INTEGER NOT NULL REFERENCES school_years(id) ON DELETE CASCADE,
    code TEXT NOT NULL,
    name TEXT NOT NULL,
    sort_order INTEGER NOT NULL,
    UNIQUE(school_year_id, code)
);

CREATE TABLE teacher_grades (
    teacher_id INTEGER NOT NULL REFERENCES teachers(id) ON DELETE CASCADE,
    school_year_id INTEGER NOT NULL REFERENCES school_years(id) ON DELETE CASCADE,
    grade_id INTEGER NOT NULL REFERENCES grades(id) ON DELETE RESTRICT,
    PRIMARY KEY(teacher_id, school_year_id, grade_id)
);

CREATE TABLE unavailability (
    teacher_id INTEGER NOT NULL REFERENCES teachers(id) ON DELETE CASCADE,
    exam_id INTEGER NOT NULL REFERENCES exams(id) ON DELETE CASCADE,
    reason TEXT,
    PRIMARY KEY(teacher_id, exam_id)
);

CREATE TABLE locks (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    exam_id INTEGER NOT NULL REFERENCES exams(id) ON DELETE CASCADE,
    grade_id INTEGER NOT NULL REFERENCES grades(id) ON DELETE CASCADE,
    teacher_id INTEGER NOT NULL REFERENCES teachers(id) ON DELETE CASCADE,
    role TEXT NULL CHECK (role IS NULL OR role IN ('setter', 'reviewer')),
    kind TEXT NOT NULL CHECK (kind IN ('pin', 'forbid'))
);

-- Unique lock index: prevent duplicate locks for the same panel, teacher, kind, and role
CREATE UNIQUE INDEX idx_locks_unique
ON locks(exam_id, grade_id, teacher_id, kind, COALESCE(role, 'any'));

CREATE TABLE rule_settings (
    school_year_id INTEGER NOT NULL REFERENCES school_years(id) ON DELETE CASCADE,
    rule_key TEXT NOT NULL,
    enabled INTEGER NOT NULL CHECK (enabled IN (0, 1)),
    weight REAL NOT NULL CHECK (weight >= 0.0),
    params_json TEXT NOT NULL DEFAULT '{}',
    PRIMARY KEY(school_year_id, rule_key)
);

CREATE TABLE plans (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    school_year_id INTEGER NOT NULL REFERENCES school_years(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    seed INTEGER NOT NULL,
    score REAL,
    is_final INTEGER NOT NULL DEFAULT 0 CHECK (is_final IN (0, 1)),
    params_json TEXT
);

-- Partial unique index: at most one final plan per school year
CREATE UNIQUE INDEX idx_plans_one_final_per_year
ON plans(school_year_id, is_final)
WHERE is_final = 1;

CREATE TABLE assignments (
    plan_id INTEGER NOT NULL REFERENCES plans(id) ON DELETE CASCADE,
    exam_id INTEGER NOT NULL REFERENCES exams(id) ON DELETE RESTRICT,
    grade_id INTEGER NOT NULL REFERENCES grades(id) ON DELETE RESTRICT,
    teacher_id INTEGER NOT NULL REFERENCES teachers(id) ON DELETE RESTRICT,
    role TEXT NOT NULL CHECK (role IN ('setter', 'reviewer')),
    PRIMARY KEY(plan_id, exam_id, grade_id, teacher_id)
);

CREATE TABLE settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);

-- Indexes for frequent query patterns
CREATE INDEX idx_teacher_grades_school_year ON teacher_grades(school_year_id);
CREATE INDEX idx_teacher_grades_teacher ON teacher_grades(teacher_id);
CREATE INDEX idx_assignments_plan ON assignments(plan_id);
CREATE INDEX idx_exams_school_year ON exams(school_year_id);
CREATE INDEX idx_teachers_campus ON teachers(campus_id);
CREATE INDEX idx_locks_exam_grade ON locks(exam_id, grade_id);
