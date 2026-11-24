## ADDED Requirements

### Requirement: Calendar file with the cancellation mail
The cancellation mail SHALL carry the calendar file of the cancelled gathering as the attachment `gathering.ics`, built the same way as the file of the response page and showing the gathering after the cancellation. Both the plain text part and the HTML part SHALL say in one sentence in the member's language that the attached file marks the entry in the calendar as cancelled. The mail SHALL hold no sentence about answering.

#### Scenario: Cancellation of an unchanged gathering
- **WHEN** an organizer cancels a gathering that was never changed and whose invitation mail carried `SEQUENCE:0`
- **THEN** each cancellation mail carries a calendar file with the same `UID`, `STATUS:CANCELLED` and `SEQUENCE:1`

#### Scenario: Sentence in English
- **WHEN** Adunata queues a cancellation mail for a member whose language is English
- **THEN** the plain text part and the HTML part each hold the English sentence that the attached file marks the entry in the calendar as cancelled
