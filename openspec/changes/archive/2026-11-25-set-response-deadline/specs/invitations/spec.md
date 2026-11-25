## ADDED Requirements

### Requirement: Reminders end at the deadline
Adunata SHALL refuse to remind pending members once the deadline for responses of the gathering has passed, with status 422 and the error code `deadlinePassed` on the field `gathering`. It SHALL then change no link and queue no mail. A cancelled or ended gathering SHALL keep its error code, and a gathering that has begun SHALL keep the code `begun`. Inviting members and resending an invitation SHALL stay possible after the deadline until the gathering has ended.

#### Scenario: Remind after the deadline
- **WHEN** today is 7 March 2027, a gathering has the first day 14 March 2027 and a deadline of 7 days and an organizer reminds its pending members
- **THEN** Adunata answers with status 422 and `{"errors":[{"field":"gathering","code":"deadlinePassed"}]}`, changes no link and queues no mail

#### Scenario: Remind on the last day to answer
- **WHEN** today is 6 March 2027 for the same gathering and an organizer reminds its pending members
- **THEN** Adunata queues a reminder for each of them

#### Scenario: Invite and resend after the deadline
- **WHEN** the deadline of a gathering has passed, it has not begun and an organizer invites a member or resends an invitation
- **THEN** Adunata accepts and queues the mail

### Requirement: Last day to answer in mails
When the gathering has a deadline, the invitation mail and the reminder mail SHALL name the last day to answer as a single date in the member's language. An invitation mail sent after the deadline has passed SHALL instead say that the deadline passed on that day and that the member can still decline through the link. Without a deadline both mails SHALL stay as they are.

#### Scenario: Invitation with a deadline
- **WHEN** Adunata invites a member whose language is German to a gathering with the first day 14 March 2027 and a deadline of 7 days before 7 March 2027
- **THEN** the mail names `6. März 2027` as the last day to answer

#### Scenario: Reminder with a deadline
- **WHEN** Adunata reminds a member whose language is English for the same gathering
- **THEN** the mail names `March 6, 2027` as the last day to answer

#### Scenario: Invitation after the deadline
- **WHEN** Adunata invites a member to a gathering whose deadline has passed
- **THEN** the mail says that the deadline passed on the last day to answer and that the member can still decline through the link

#### Scenario: No deadline
- **WHEN** Adunata invites a member to a gathering without a deadline
- **THEN** the mail names no last day to answer
