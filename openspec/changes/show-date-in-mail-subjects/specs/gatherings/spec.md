## MODIFIED Requirements

### Requirement: Cancel gatherings
An organizer SHALL be able to cancel a gathering that has not ended. A cancelled gathering SHALL stay visible to organizers as cancelled. Adunata SHALL inform every invited member by mail. The subject of that mail SHALL name the title and the span of the gathering, separated by a comma. Its invitation links SHALL show that it is cancelled and accept no responses.

#### Scenario: Cancellation mail
- **WHEN** an organizer cancels a gathering
- **THEN** Adunata sends every invited member a mail in the member's language that names the cancelled gathering, and its subject reads `<prefix>: <title>, <span>` with the span in the member's language, for example `Abgesagt: Probenwochenende, 12.–14. März 2027` in German or `Cancelled: Probenwochenende, March 12–14, 2027` in English

#### Scenario: Cancelled gathering takes no invitations
- **WHEN** an organizer tries to invite members to a cancelled gathering
- **THEN** Adunata refuses and explains that the gathering is cancelled

#### Scenario: Member opens a cancelled gathering
- **WHEN** a member opens the invitation link of a cancelled gathering
- **THEN** Adunata shows that the gathering is cancelled and offers no form
