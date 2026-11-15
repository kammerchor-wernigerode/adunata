CREATE TABLE members (
    id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    first_name TEXT NOT NULL CHECK (char_length(first_name) BETWEEN 1 AND 100),
    last_name TEXT NOT NULL CHECK (char_length(last_name) BETWEEN 1 AND 100),
    mail TEXT NOT NULL CHECK (char_length(mail) BETWEEN 3 AND 254),
    language TEXT NOT NULL CHECK (language IN ('de', 'en')),
    active BOOLEAN NOT NULL DEFAULT true,
    organizer BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE UNIQUE INDEX members_mail_ignoring_case ON members (lower(mail));
