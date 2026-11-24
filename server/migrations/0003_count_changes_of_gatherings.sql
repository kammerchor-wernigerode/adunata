ALTER TABLE gatherings
    ADD COLUMN version INTEGER NOT NULL DEFAULT 0,
    ADD COLUMN changed_at TIMESTAMPTZ;

UPDATE gatherings SET changed_at = created_at;

ALTER TABLE gatherings
    ALTER COLUMN changed_at SET NOT NULL,
    ALTER COLUMN changed_at SET DEFAULT now();
