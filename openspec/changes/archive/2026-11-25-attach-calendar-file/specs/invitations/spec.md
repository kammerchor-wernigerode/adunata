## ADDED Requirements

### Requirement: Calendar file with invitation and reminder
The invitation mail, a resent invitation mail and the reminder mail SHALL carry the calendar file of the gathering as the attachment `gathering.ics`. The file SHALL be the one the response page offers, built the same way, and SHALL show the gathering as it is in the transaction that queues the mail. Every invited member SHALL receive it, whatever the member answered. Directly after the personal link, both the plain text part and the HTML part SHALL say in one sentence in the member's language that the attached file adds the gathering to a calendar and that saving it does not answer the invitation.

#### Scenario: Invitation carries the file
- **WHEN** an organizer invites a member to a gathering that was changed twice
- **THEN** the queued mail carries a calendar file that equals the file the response page would serve for that gathering at that moment, with `SEQUENCE:2`, `STATUS:CONFIRMED` and no `METHOD`, `ORGANIZER` or `ATTENDEE`

#### Scenario: Reminder to a pending member
- **WHEN** an organizer reminds a member who has not answered
- **THEN** the reminder mail carries the calendar file of the gathering, although the response page offers that member no file

#### Scenario: Sentence next to the link
- **WHEN** Adunata queues an invitation mail for a member whose language is German
- **THEN** the plain text part and the HTML part each hold, directly after the personal link, the German sentence that the attached file adds the gathering to a calendar and that saving it does not answer the invitation

#### Scenario: Resent invitation
- **WHEN** an organizer resends an invitation
- **THEN** the new mail carries the calendar file of the gathering as it is at that moment
