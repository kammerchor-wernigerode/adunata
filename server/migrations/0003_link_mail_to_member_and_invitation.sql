ALTER TABLE mail_outbox
    ADD COLUMN member_id BIGINT,
    ADD COLUMN invitation_id BIGINT;
