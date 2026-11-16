DELETE FROM mail_outbox WHERE member_id IS NOT NULL AND member_id NOT IN (SELECT id FROM members);

ALTER TABLE mail_outbox
    ADD CONSTRAINT mail_outbox_member FOREIGN KEY (member_id) REFERENCES members (id) ON DELETE CASCADE;
