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
An organizer SHALL be able to cancel a gathering that has not ended. A cancelled gathering SHALL stay visible to organizers as cancelled. Adunata SHALL inform every invited member by mail. Its invitation links SHALL show that it is cancelled and accept no responses.

#### Scenario: Cancellation mail
- **WHEN** an organizer cancels a gathering
- **THEN** Adunata sends every invited member a mail in the member's language that names the cancelled gathering

#### Scenario: Cancelled gathering takes no invitations
- **WHEN** an organizer tries to invite members to a cancelled gathering
- **THEN** Adunata refuses and explains that the gathering is cancelled

#### Scenario: Member opens a cancelled gathering
- **WHEN** a member opens the invitation link of a cancelled gathering
- **THEN** Adunata shows that the gathering is cancelled and offers no form

### Requirement: Gathering list
Adunata SHALL show organizers upcoming gatherings first, ordered by first day, and past gatherings separately.

#### Scenario: Gathering has ended
- **WHEN** the last day of a gathering has passed
- **THEN** Adunata lists the gathering among past gatherings

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
