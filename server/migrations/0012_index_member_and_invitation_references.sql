CREATE INDEX mail_outbox_member_id ON mail_outbox (member_id);
CREATE INDEX mail_outbox_invitation_id ON mail_outbox (invitation_id, id);
CREATE INDEX invitations_member_id ON invitations (member_id);
CREATE INDEX sessions_member_id ON sessions (member_id);
