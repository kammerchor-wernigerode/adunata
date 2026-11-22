# gatherings Specification

## Purpose
Lets organizers describe the gatherings an ensemble meets for, such as a rehearsal weekend, a rehearsal camp during the week or a concert. Members can then be invited to them.

## Requirements

### Requirement: Gathering data
A gathering SHALL have a title, a location, a first day and a last day. The description SHALL be optional. The last day SHALL NOT be before the first day. A first day equal to the last day SHALL be valid. A new first day SHALL NOT be before today in the time zone of the instance. When a gathering changes, this applies only if its first day differs from the stored one. The form SHALL offer only days that can be valid and SHALL move the last day along with the first day.

#### Scenario: Create a gathering
- **WHEN** an organizer creates a gathering with title, location, first day and last day
- **THEN** Adunata stores the gathering and lists it among upcoming gatherings

#### Scenario: Last day before first day
- **WHEN** an organizer enters a last day that lies before the first day
- **THEN** Adunata rejects the gathering and marks the last day as invalid

#### Scenario: One-day gathering
- **WHEN** an organizer creates a gathering whose first day is today and whose last day equals the first day
- **THEN** Adunata stores the gathering

#### Scenario: New gathering starts in the past
- **WHEN** an organizer creates a gathering whose first day lies before today
- **THEN** Adunata rejects the gathering with the error code `firstDayInPast` on the first day and the form shows a translated message at the first day

#### Scenario: Changed first day in the past
- **WHEN** an organizer changes the first day of a gathering that has not ended to a day before today that differs from the stored first day
- **THEN** Adunata rejects the change with the error code `firstDayInPast` on the first day and keeps the gathering as it was

#### Scenario: Running gathering keeps its first day
- **WHEN** an organizer changes the title of a running gathering whose stored first day lies before today and sends that first day unchanged
- **THEN** Adunata stores the change

#### Scenario: Calendar of the first day
- **WHEN** an organizer opens the calendar of the first day
- **THEN** every day before today by the date of the browser is disabled, except the first day already chosen, and today is enabled

#### Scenario: Calendar of the last day
- **WHEN** an organizer has chosen 12 March as the first day and opens the calendar of the last day
- **THEN** every day before 12 March is disabled and 12 March is enabled

#### Scenario: Last day moves with the first day
- **WHEN** the first day is 12 March, the last day is 14 March and the organizer changes the first day to 15 March
- **THEN** the form shows 17 March as the last day

#### Scenario: Nothing moves without both days
- **WHEN** the last day is set, the first day is empty and the organizer chooses a first day
- **THEN** the last day stays as it was

### Requirement: Edit gatherings
An organizer SHALL be able to change every field of a gathering that has not ended. Existing responses SHALL keep their values when the span changes.

#### Scenario: Shorten a gathering
- **WHEN** an organizer moves the first day of a gathering one day later
- **THEN** Adunata keeps existing responses and marks each response whose period lies partly or wholly outside the new span

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

### Requirement: Gathering list
Adunata SHALL show organizers upcoming gatherings first, ordered by first day, and past gatherings separately. Adunata SHALL group past gatherings by the year of their first day, latest year first and latest gathering first within a year. Each year SHALL show its number of gatherings and SHALL open and close on its own. The client SHALL remember on the device which years the organizer opened or closed. A year without a remembered choice SHALL start open only if it is the latest year.

#### Scenario: Gathering has ended
- **WHEN** the last day of a gathering has passed
- **THEN** Adunata lists the gathering among past gatherings

#### Scenario: Years in order
- **WHEN** an organizer opens the gathering list with past gatherings whose first days lie in 2024, 2025 and 2026
- **THEN** Adunata shows the years 2026, 2025 and 2024 in this order as headings below "Past", each with the number of its gatherings

#### Scenario: Only the latest year starts open
- **WHEN** an organizer opens the gathering list on a device that remembers no choice, with past gatherings in 2024 and 2025 and none in the current year
- **THEN** Adunata shows the gatherings of 2025 with the latest first and keeps 2024 closed

#### Scenario: Several years open at once
- **WHEN** the year 2025 is open and the organizer opens the year 2024
- **THEN** Adunata shows the gatherings of both years

#### Scenario: Close a year
- **WHEN** the organizer closes an open year
- **THEN** Adunata hides its gatherings and still shows the year with its number of gatherings

#### Scenario: Remember open years
- **WHEN** an organizer closes 2025, opens 2024, leaves the gathering list and comes back on the same device
- **THEN** Adunata shows 2025 closed and 2024 open

#### Scenario: New latest year
- **WHEN** the device remembers choices for 2024 and 2025 and a gathering in 2026 has ended since
- **THEN** Adunata shows 2026 open and 2024 and 2025 as remembered

#### Scenario: Gathering over New Year
- **WHEN** a past gathering runs from 30 December 2025 to 2 January 2026
- **THEN** Adunata lists it in the year 2025

#### Scenario: Number of gatherings
- **WHEN** a year holds one past gathering and another year holds two
- **THEN** Adunata shows "1 gathering" and "2 gatherings" in English and "1 Treffen" and "2 Treffen" in German

#### Scenario: Cancelled gathering in a year
- **WHEN** an open year holds a cancelled past gathering
- **THEN** its row shows the badge for a cancelled gathering and its date line with the year

#### Scenario: No past gatherings
- **WHEN** an organizer opens the gathering list without past gatherings
- **THEN** Adunata shows no year below "Past" and shows the message it shows for an empty section

### Requirement: Page actions of gatherings
The gathering list SHALL offer "Create gathering" and the page of a gathering SHALL offer "Edit gathering" as the page action in the page header. The page action SHALL sit at the top right on the line of the main heading, outside the heading. When heading and action do not fit on one line, the action SHALL wrap below the heading. The page of a gathering that is cancelled or has ended SHALL offer no page action.

#### Scenario: Create gathering in the header
- **WHEN** an organizer opens the gathering list
- **THEN** the link "Create gathering" is the first control after the main heading outside the heading. The heading still names the page and the browser tab

#### Scenario: Edit gathering in the header
- **WHEN** an organizer opens a gathering that is neither cancelled nor ended
- **THEN** the link "Edit gathering" is the first control after the main heading outside the heading. Span, location and description follow below the header

#### Scenario: No edit for an ended or cancelled gathering
- **WHEN** an organizer opens a gathering that has ended or is cancelled
- **THEN** the page shows no link "Edit gathering"
