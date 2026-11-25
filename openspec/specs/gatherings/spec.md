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

### Requirement: Save and add another gathering
The form for a new gathering SHALL offer a second button "Save & add another", in German "Speichern & Weiter", after the button "Save". The form that changes a gathering SHALL NOT offer it. "Save" SHALL stay the button that the Enter key triggers and SHALL open the page of the stored gathering. The second button SHALL store the gathering and keep the form open with its title, location and description, empty first and last day and the focus on the title. The form SHALL then show a line, announced to screen readers, that names the title and the span of the stored gathering and links to its page. The line SHALL stay until the next save or refusal replaces it. Both buttons SHALL be disabled while a save runs. A refused save SHALL keep every value and mark the fields with either button.

#### Scenario: Second button only for a new gathering
- **WHEN** an organizer opens the form for a new gathering and then the form that changes a gathering
- **THEN** the first form shows "Save" followed by "Save & add another" and the second form shows "Save" without "Save & add another"

#### Scenario: Save and add another
- **WHEN** an organizer fills the form for a new gathering with title "Probenwochenende", location "Wernigerode", a description and the days 12 to 14 March 2027 and chooses "Save & add another"
- **THEN** Adunata stores the gathering, the form stays open with title, location and description unchanged, first and last day are empty, the focus is on the title and an element with `role="status"` reads the title and "Fri, March 12 – Sun, March 14, 2027" with a link to the page of the stored gathering

#### Scenario: Next save replaces the line
- **WHEN** an organizer stores a gathering with "Save & add another" and then another one with different days the same way
- **THEN** the line names only the second gathering

#### Scenario: Refusal replaces the line
- **WHEN** an organizer stores a gathering with "Save & add another" and the next save with "Save & add another" is refused because the first day lies in the past
- **THEN** the line disappears, the form keeps every value, marks the first day and moves the focus to it

#### Scenario: Enter saves and opens the page
- **WHEN** an organizer fills the form for a new gathering and presses Enter in the title
- **THEN** Adunata stores the gathering and opens its page

#### Scenario: No double save
- **WHEN** a save runs after an organizer chose "Save & add another"
- **THEN** both "Save" and "Save & add another" are disabled until the save finishes

#### Scenario: List shows the stored gathering
- **WHEN** an organizer stores a gathering with "Save & add another" and opens the gathering list
- **THEN** the list shows the stored gathering among the upcoming gatherings

### Requirement: Deadline for responses
A gathering SHALL have an optional deadline for responses, stored as a whole number of days before the first day of at least 1. Without a value the gathering SHALL have no deadline. The last day to answer SHALL be the first day minus the number of days minus one day, so it moves with the first day. The deadline SHALL pass when the day after the last day to answer begins in the time zone of the instance. The API SHALL take and return the deadline as the field `responseDeadlineDays`. A gathering SHALL also report `lastDayToAnswer` as a date, or null without a deadline, and `deadlinePassed`, computed on the server like `begun`. Adunata SHALL refuse a number below 1 with the error code `invalid` on `responseDeadlineDays`. Adunata SHALL refuse a last day to answer before today with the error code `deadlineInPast` on `responseDeadlineDays`, unless the gathering changes and its last day to answer equals the stored one. A number so large that the date cannot be computed SHALL count as a last day to answer before today. Removing the deadline SHALL always be valid.

The form of the gathering SHALL offer the deadline as an optional number with the unit days or weeks and, next to it, a calendar for the last day to answer. A number in weeks SHALL count 7 days per week. Choosing a day in the calendar SHALL set the number in days. Changing the number or the unit SHALL move the day in the calendar. The calendar SHALL enable only the days from today by the date of the browser up to two days before the first day, and SHALL stay disabled without a first day. An empty number SHALL mean no deadline. The form SHALL show a translated message at the deadline for each error code. The number, the unit and the calendar SHALL be shadcn components, reusing `Input`, `ToggleGroup` and `DatePicker`. "Save & add another" SHALL keep the number and the unit.

#### Scenario: Last day to answer
- **WHEN** an organizer creates a gathering with the first day 14 March 2027 and a deadline of 7 days
- **THEN** Adunata stores `responseDeadlineDays` 7 and reports `lastDayToAnswer` `2027-03-06`

#### Scenario: Smallest deadline
- **WHEN** an organizer creates a gathering with the first day 14 March 2027 and a deadline of 1 day
- **THEN** Adunata reports `lastDayToAnswer` `2027-03-12`

#### Scenario: No deadline
- **WHEN** an organizer creates a gathering without `responseDeadlineDays`
- **THEN** Adunata stores it and reports `responseDeadlineDays` null, `lastDayToAnswer` null and `deadlinePassed` false

#### Scenario: Existing gatherings
- **WHEN** the migration runs on a database with gatherings
- **THEN** every existing gathering has no deadline

#### Scenario: Zero days
- **WHEN** an organizer sends `responseDeadlineDays` 0
- **THEN** Adunata answers with status 422 and the error code `invalid` on `responseDeadlineDays`

#### Scenario: Last day to answer is today
- **WHEN** today is 6 March 2027 in the time zone of the instance and an organizer creates a gathering with the first day 14 March 2027 and a deadline of 7 days
- **THEN** Adunata stores it and reports `deadlinePassed` false

#### Scenario: Last day to answer in the past
- **WHEN** today is 7 March 2027 in the time zone of the instance and an organizer creates a gathering with the first day 14 March 2027 and a deadline of 7 days
- **THEN** Adunata answers with status 422 and the error code `deadlineInPast` on `responseDeadlineDays`

#### Scenario: Huge number of days
- **WHEN** an organizer sends `responseDeadlineDays` 2000000000
- **THEN** Adunata answers with status 422 and the error code `deadlineInPast` on `responseDeadlineDays`

#### Scenario: Deadline has passed
- **WHEN** today is 7 March 2027 in the time zone of the instance and a stored gathering has the first day 14 March 2027 and a deadline of 7 days
- **THEN** the gathering reports `deadlinePassed` true and `begun` false

#### Scenario: Other change after the deadline
- **WHEN** the deadline of a gathering has passed and an organizer changes its title and sends the deadline and the first day unchanged
- **THEN** Adunata stores the change

#### Scenario: Moved first day after the deadline
- **WHEN** today is 7 March 2027, a gathering has the first day 14 March 2027 and a deadline of 7 days and an organizer moves the first day to 21 March 2027 with the same deadline
- **THEN** Adunata stores the change, reports `lastDayToAnswer` `2027-03-13` and `deadlinePassed` false

#### Scenario: Moving the deadline into the past
- **WHEN** today is 7 March 2027, a gathering has the first day 21 March 2027 and a deadline of 7 days and an organizer changes the deadline to 14 days
- **THEN** Adunata answers with status 422 and the error code `deadlineInPast` on `responseDeadlineDays` and keeps the gathering as it was

#### Scenario: Removing a passed deadline
- **WHEN** the deadline of a gathering has passed and an organizer sends `responseDeadlineDays` null
- **THEN** Adunata stores the change and the gathering reports `deadlinePassed` false

#### Scenario: Weeks in the form
- **WHEN** an organizer has chosen 14 March 2027 as the first day, enters 2 and chooses the unit weeks
- **THEN** the calendar shows 27 February 2027 and the form sends `responseDeadlineDays` 14

#### Scenario: Day in the calendar
- **WHEN** an organizer has chosen 14 March 2027 as the first day and picks 6 March 2027 in the calendar of the deadline
- **THEN** the number shows 7 with the unit days and the form sends `responseDeadlineDays` 7

#### Scenario: Calendar of the deadline
- **WHEN** today is 1 March 2027 by the date of the browser, the first day is 14 March 2027 and an organizer opens the calendar of the deadline
- **THEN** 1 March and 12 March are enabled and 28 February and 13 March are disabled

#### Scenario: First day moves
- **WHEN** the first day is 14 March 2027 with a deadline of 7 days and the organizer changes the first day to 21 March 2027
- **THEN** the number stays 7 and the calendar shows 13 March 2027

#### Scenario: Empty number
- **WHEN** an organizer leaves the number of the deadline empty and saves
- **THEN** the form sends `responseDeadlineDays` null

### Requirement: Calendar file with the cancellation mail
The cancellation mail SHALL carry the calendar file of the cancelled gathering as the attachment `gathering.ics`, built the same way as the file of the response page and showing the gathering after the cancellation. Both the plain text part and the HTML part SHALL say in one sentence in the member's language that the attached file marks the entry in the calendar as cancelled. The mail SHALL hold no sentence about answering.

#### Scenario: Cancellation of an unchanged gathering
- **WHEN** an organizer cancels a gathering that was never changed and whose invitation mail carried `SEQUENCE:0`
- **THEN** each cancellation mail carries a calendar file with the same `UID`, `STATUS:CANCELLED` and `SEQUENCE:1`

#### Scenario: Sentence in English
- **WHEN** Adunata queues a cancellation mail for a member whose language is English
- **THEN** the plain text part and the HTML part each hold the English sentence that the attached file marks the entry in the calendar as cancelled
