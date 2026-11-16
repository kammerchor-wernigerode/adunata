CREATE TABLE responses (
    invitation_id BIGINT PRIMARY KEY REFERENCES invitations (id) ON DELETE CASCADE,
    status TEXT NOT NULL CHECK (status IN ('accepted', 'tentative', 'declined')),
    arrival TIMESTAMP,
    departure TIMESTAMP,
    beds TEXT CHECK (beds IN ('need', 'offer', 'none')),
    bed_count SMALLINT CHECK (bed_count BETWEEN 1 AND 20),
    comment TEXT CHECK (char_length(comment) <= 2000),
    changed_by TEXT NOT NULL CHECK (changed_by IN ('member', 'organizer')),
    changed_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CHECK (status <> 'declined' OR (arrival IS NULL AND departure IS NULL AND beds IS NULL AND bed_count IS NULL)),
    CHECK ((beds IN ('need', 'offer')) = (bed_count IS NOT NULL))
);
