## ADDED Requirements

### Requirement: Remind pending members
An organizer SHALL be able to remind every member of a gathering who has not answered. The recipients SHALL be every invitation of the gathering that has no response and whose link is active, including invitations whose mail failed for good and invitations of deactivated members. A tentative answer SHALL count as an answer. For each recipient Adunata SHALL store a new secret in place of the old one and queue a reminder mail, all in one transaction, so the old link stops working. Adunata SHALL report how many reminders it queued. Adunata SHALL refuse the action for a gathering that is cancelled or has ended. Adunata SHALL record no time of a reminder and SHALL allow a second reminder.

#### Scenario: Remind pending members
- **WHEN** a gathering has one accepted, one tentative and two pending invitations with active links and an organizer reminds the pending members
- **THEN** Adunata queues a reminder for the two pending members only and reports a count of 2

#### Scenario: Old link after a reminder
- **WHEN** a member opens the link they held before a reminder
- **THEN** Adunata shows that the link is no longer valid, and the link in the reminder opens the response form

#### Scenario: Revoked link is skipped
- **WHEN** a pending invitation has a revoked link and an organizer reminds the pending members
- **THEN** Adunata queues no reminder for it and its link stays revoked

#### Scenario: Failed mail is included
- **WHEN** the invitation mail of a pending member failed for good and an organizer reminds the pending members
- **THEN** Adunata queues a reminder for that member

#### Scenario: Nobody to remind
- **WHEN** every invited member has answered and an organizer reminds the pending members
- **THEN** Adunata queues no mail and reports a count of 0

#### Scenario: Closed gathering
- **WHEN** an organizer reminds the pending members of a gathering that is cancelled or has ended
- **THEN** Adunata refuses with the same error as for a resend, changes no link and queues no mail

#### Scenario: Second reminder
- **WHEN** an organizer reminds the pending members twice
- **THEN** Adunata queues a reminder each time

### Requirement: Reminder mail
The reminder mail SHALL be written in the member's language. Its subject SHALL start with `Erinnerung:` in German and `Reminder:` in English and then name the title and the span of the gathering, separated by a comma, as the invitation does. Its body SHALL say that no answer has arrived yet and SHALL otherwise follow the invitation mail, with title, location, span, the new personal link and the links to the privacy notice and the imprint when the operator configured them.

#### Scenario: Subject of a German reminder
- **WHEN** Adunata reminds a member whose language is German for the gathering "Probenwochenende" from 12 to 14 March 2027
- **THEN** the subject of the mail is `Erinnerung: Probenwochenende, 12.–14. März 2027`

#### Scenario: Subject of an English reminder
- **WHEN** Adunata reminds a member whose language is English for the same gathering
- **THEN** the subject of the mail starts with `Reminder: Probenwochenende, `

#### Scenario: Reminder content
- **WHEN** Adunata reminds a member and the operator configured a privacy notice and an imprint
- **THEN** the mail says that no answer has arrived yet and contains the title, location, span, the new personal link and both legal links
