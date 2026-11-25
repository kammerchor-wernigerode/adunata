## ADDED Requirements

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
