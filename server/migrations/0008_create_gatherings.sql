CREATE TABLE gatherings (
    id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    title TEXT NOT NULL CHECK (char_length(title) BETWEEN 1 AND 200),
    location TEXT NOT NULL CHECK (char_length(location) BETWEEN 1 AND 200),
    description TEXT CHECK (char_length(description) <= 5000),
    first_day DATE NOT NULL,
    last_day DATE NOT NULL,
    cancelled_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CHECK (last_day >= first_day)
);
