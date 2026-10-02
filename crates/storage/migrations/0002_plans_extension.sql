-- 0002_plans_extension.sql
-- Add rank, score_report_json, run_params_json, and source to plans table

ALTER TABLE plans ADD COLUMN rank INTEGER;
ALTER TABLE plans ADD COLUMN score_report_json TEXT;
ALTER TABLE plans ADD COLUMN run_params_json TEXT;
ALTER TABLE plans ADD COLUMN source TEXT NOT NULL DEFAULT 'optimizer' CHECK (source IN ('optimizer', 'manual', 'duplicate'));
