## ADDED Requirements

### Requirement: Mail confinement
While mail confinement is on, Adunata SHALL deliver every mail to the redirect address instead of its recipient. This SHALL hold for every mail in the outbox at delivery, including mail enqueued before the start and mail without a member. A redirected mail SHALL name its original recipient in the header `X-Original-To`. Its subject and both body parts SHALL stay unchanged. While confinement is off, Adunata SHALL deliver to the recipient and add no header `X-Original-To`.

#### Scenario: Invitation on a test system
- **WHEN** `serve --dev` runs with `ADNT_MAIL_CONFINE=true` and `ADNT_MAIL_CONFINE_TO=test@example.org` and an organizer invites the member `ada@example.org`
- **THEN** the mail server receives the invitation for `test@example.org` only, with the header `X-Original-To: ada@example.org` and the subject and bodies of an unconfined invitation

#### Scenario: Mail waiting from before the start
- **WHEN** the outbox holds a mail for `ada@example.org` from an earlier run and `serve --dev` starts with confinement on
- **THEN** the mail server receives that mail for the redirect address only

#### Scenario: Confinement off
- **WHEN** `serve --dev` runs without `ADNT_MAIL_CONFINE`
- **THEN** each mail goes to its recipient and carries no header `X-Original-To`
