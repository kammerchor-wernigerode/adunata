ALTER TABLE gatherings ADD COLUMN response_deadline_days INTEGER CHECK (response_deadline_days >= 1);
