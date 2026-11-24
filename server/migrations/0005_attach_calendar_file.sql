ALTER TABLE mail_outbox
    ADD COLUMN calendar TEXT,
    ADD CONSTRAINT mail_outbox_calendar_while_pending CHECK (
        calendar IS NULL OR (sent_at IS NULL AND failed_at IS NULL)
    );
