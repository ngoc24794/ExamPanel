-- 0004_phase9_data_model.sql
-- Split problem_hash into data_hash and rules_hash, backfilling data_hash from problem_hash.
ALTER TABLE plans ADD COLUMN data_hash TEXT;
ALTER TABLE plans ADD COLUMN rules_hash TEXT;
UPDATE plans SET data_hash = problem_hash;
ALTER TABLE plans DROP COLUMN problem_hash;

-- Optional teacher code with unique index where not null
ALTER TABLE teachers ADD COLUMN code TEXT;
CREATE UNIQUE INDEX idx_teachers_code ON teachers(code) WHERE code IS NOT NULL;
