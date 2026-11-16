ALTER TABLE mail_outbox
    ALTER COLUMN recipient DROP NOT NULL,
    ALTER COLUMN subject DROP NOT NULL,
    ALTER COLUMN text_body DROP NOT NULL,
    ALTER COLUMN html_body DROP NOT NULL;

UPDATE mail_outbox
SET recipient = NULL, subject = NULL, text_body = NULL, html_body = NULL
WHERE sent_at IS NOT NULL OR failed_at IS NOT NULL;

ALTER TABLE mail_outbox ADD CONSTRAINT mail_outbox_content_while_pending CHECK (
    (sent_at IS NULL AND failed_at IS NULL) = (recipient IS NOT NULL AND subject IS NOT NULL AND text_body IS NOT NULL AND html_body IS NOT NULL)
);
