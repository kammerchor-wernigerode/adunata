CREATE TABLE spike_responses (
    id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    name TEXT NOT NULL UNIQUE CHECK (char_length(name) BETWEEN 1 AND 100),
    attendance TEXT NOT NULL CHECK (attendance IN ('accepted', 'tentative', 'declined')),
    arrival DATE,
    beds SMALLINT NOT NULL CHECK (beds BETWEEN 0 AND 20),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
