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

CREATE TABLE sign_in_links (
    id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    member_id BIGINT NOT NULL REFERENCES members (id) ON DELETE CASCADE,
    secret_hash BYTEA NOT NULL UNIQUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    expires_at TIMESTAMPTZ NOT NULL,
    used_at TIMESTAMPTZ
);

CREATE INDEX sign_in_links_recent ON sign_in_links (member_id, created_at);

CREATE TABLE sessions (
    id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    member_id BIGINT NOT NULL REFERENCES members (id) ON DELETE CASCADE,
    secret_hash BYTEA NOT NULL UNIQUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    expires_at TIMESTAMPTZ NOT NULL
);

CREATE INDEX sessions_member_id ON sessions (member_id);

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

CREATE TABLE invitations (
    id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    gathering_id BIGINT NOT NULL REFERENCES gatherings (id) ON DELETE CASCADE,
    member_id BIGINT NOT NULL REFERENCES members (id) ON DELETE CASCADE,
    secret_hash BYTEA UNIQUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (gathering_id, member_id)
);

CREATE INDEX invitations_member_id ON invitations (member_id);

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

ALTER TABLE mail_outbox
    ADD COLUMN member_id BIGINT,
    ADD COLUMN invitation_id BIGINT,
    ADD CONSTRAINT mail_outbox_member FOREIGN KEY (member_id) REFERENCES members (id) ON DELETE CASCADE,
    ADD CONSTRAINT mail_outbox_invitation FOREIGN KEY (invitation_id) REFERENCES invitations (id) ON DELETE CASCADE,
    ALTER COLUMN recipient DROP NOT NULL,
    ALTER COLUMN subject DROP NOT NULL,
    ALTER COLUMN text_body DROP NOT NULL,
    ALTER COLUMN html_body DROP NOT NULL,
    ADD CONSTRAINT mail_outbox_content_while_pending CHECK (
        (sent_at IS NULL AND failed_at IS NULL) = (recipient IS NOT NULL AND subject IS NOT NULL AND text_body IS NOT NULL AND html_body IS NOT NULL)
    );

CREATE INDEX mail_outbox_member_id ON mail_outbox (member_id);
CREATE INDEX mail_outbox_invitation_id ON mail_outbox (invitation_id, id);
