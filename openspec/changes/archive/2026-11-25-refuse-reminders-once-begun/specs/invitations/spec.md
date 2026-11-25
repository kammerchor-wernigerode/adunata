## MODIFIED Requirements

### Requirement: Remind pending members
An organizer SHALL be able to remind every member of a gathering who has not answered. The recipients SHALL be every invitation of the gathering that has no response and whose link is active, including invitations whose mail failed for good and invitations of deactivated members. A tentative answer SHALL count as an answer. For each recipient Adunata SHALL store a new secret in place of the old one and queue a reminder mail, all in one transaction, so the old link stops working. Adunata SHALL report how many reminders it queued. Adunata SHALL refuse the action for a gathering that is cancelled or has ended. Adunata SHALL also refuse the action once the first day of the gathering has begun in the time zone of the instance, the same moment from which the link shows the response without a form, with status 422 and the error code `begun` on the field `gathering`. Adunata SHALL record no time of a reminder and SHALL allow a second reminder.

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

#### Scenario: Gathering has begun
- **WHEN** the first day of a gathering that is not cancelled is today in the time zone of the instance and an organizer reminds its pending members
- **THEN** Adunata answers with status 422 and `{"errors":[{"field":"gathering","code":"begun"}]}`, changes no link and queues no mail

#### Scenario: Day before the first day
- **WHEN** the first day of a gathering is tomorrow in the time zone of the instance and an organizer reminds its pending members
- **THEN** Adunata queues a reminder for each of them

#### Scenario: Invite and resend after the first day
- **WHEN** a gathering has begun and has not ended and an organizer invites a member or resends an invitation
- **THEN** Adunata accepts and queues the mail

#### Scenario: Second reminder
- **WHEN** an organizer reminds the pending members twice
- **THEN** Adunata queues a reminder each time
