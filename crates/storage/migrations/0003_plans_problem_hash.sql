-- 0003_plans_problem_hash.sql
-- Add problem_hash to plans table for staleness detection

ALTER TABLE plans ADD COLUMN problem_hash TEXT;
