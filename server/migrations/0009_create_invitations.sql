CREATE TABLE invitations (
    id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    gathering_id BIGINT NOT NULL REFERENCES gatherings (id) ON DELETE CASCADE,
    member_id BIGINT NOT NULL REFERENCES members (id) ON DELETE CASCADE,
    secret_hash BYTEA UNIQUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (gathering_id, member_id)
);

DELETE FROM mail_outbox WHERE invitation_id IS NOT NULL;

ALTER TABLE mail_outbox
    ADD CONSTRAINT mail_outbox_invitation FOREIGN KEY (invitation_id) REFERENCES invitations (id) ON DELETE CASCADE;
