## ADDED Requirements

### Requirement: Calendar file on the response page
The response page SHALL offer the button "Add to calendar" ("In Kalender eintragen") to a member whose response is accepted or tentative, as long as the last day of the gathering has not passed in the time zone of the instance. It SHALL offer it also when the gathering is cancelled. It SHALL not offer it to a member who declined or has not answered, nor after the gathering has ended. The button SHALL save the file as `gathering.ics`. Next to the button one sentence SHALL state that the calendar does not follow later changes on its own and that a new download replaces the entry. The route `GET /api/invitation/calendar` SHALL expect the secret of the personal link in the header `Authorization` and SHALL answer 404 when the link is unknown or revoked or when the page offers no file.

#### Scenario: Accepted member adds the gathering
- **WHEN** a member who accepted opens the personal link of a gathering that has not ended and presses "Add to calendar"
- **THEN** the browser saves `gathering.ics`, which the route served with the media type `text/calendar; charset=utf-8`

#### Scenario: Tentative member of a cancelled gathering
- **WHEN** a member who accepted tentatively opens the personal link of a cancelled gathering whose last day has not passed
- **THEN** the page offers "Add to calendar" and the file holds `STATUS:CANCELLED`

#### Scenario: Declined or pending member
- **WHEN** a member who declined or has not answered opens the personal link
- **THEN** the page shows no button and `GET /api/invitation/calendar` with that secret answers 404

#### Scenario: Gathering has ended
- **WHEN** a member who accepted opens the personal link after the last day, cancelled or not
- **THEN** the page shows no button and the route answers 404

#### Scenario: No secret
- **WHEN** `GET /api/invitation/calendar` is called without a valid secret
- **THEN** it answers 404 and the response holds no data of a gathering

### Requirement: Content of the calendar file
The calendar file SHALL follow RFC 5545. It SHALL hold `VERSION:2.0`, `PRODID:-//Adunata//Adunata//EN`, `CALSCALE:GREGORIAN` and one `VEVENT` with, in this order, `UID`, `DTSTAMP`, `DTSTART;VALUE=DATE`, `DTEND;VALUE=DATE`, `SUMMARY`, `LOCATION`, `DESCRIPTION` only when the gathering has one, `STATUS`, `TRANSP:OPAQUE`, `CREATED`, `LAST-MODIFIED` and `SEQUENCE`. `DTSTART` SHALL be the first day and `DTEND` the day after the last day. `STATUS` SHALL be `CANCELLED` for a cancelled gathering and `CONFIRMED` otherwise. The file SHALL hold no `METHOD`, `ORGANIZER`, `ATTENDEE`, `URL`, `VALARM` or `VTIMEZONE`, and nothing of the member. Every line SHALL end with CRLF. Text SHALL escape backslash, comma and semicolon with a backslash and write a line break as `\n`. A line longer than 75 octets SHALL be folded without splitting a UTF-8 character.

#### Scenario: Gathering over three days
- **WHEN** a member downloads the file of "Rehearsal weekend" at "Music academy, Springfield" from 12 to 14 March 2027 with the description "Please bring your scores." and "We start with dinner." on two lines
- **THEN** the file holds `DTSTART;VALUE=DATE:20270312`, `DTEND;VALUE=DATE:20270315`, `SUMMARY:Rehearsal weekend`, `LOCATION:Music academy\, Springfield`, `DESCRIPTION:Please bring your scores.\nWe start with dinner.`, `STATUS:CONFIRMED` and `TRANSP:OPAQUE`, each line ends with CRLF and the file holds no `METHOD`, `URL` or `ATTENDEE`

#### Scenario: Gathering of one day
- **WHEN** a gathering lasts only 12 March 2027
- **THEN** the file holds `DTSTART;VALUE=DATE:20270312` and `DTEND;VALUE=DATE:20270313`

#### Scenario: Gathering without description
- **WHEN** a gathering has no description
- **THEN** the file holds no line `DESCRIPTION`

#### Scenario: Long title with umlauts
- **WHEN** a title of 200 characters with umlauts is written
- **THEN** no line of the file exceeds 75 octets before its CRLF, each continuation line starts with one space, every line is valid UTF-8 and unfolding gives the escaped title back

### Requirement: Identity and updates of the calendar entry
The `UID` SHALL be a UUID of version 5 per RFC 4122 in the namespace `81aa7074-4fe3-41aa-8af7-5d34430bdd9d` with the name `gathering:<id>:<created>`, where `<id>` is the id of the gathering and `<created>` is its creation time as microseconds since the Unix epoch in decimal digits. It SHALL be the same for every member and every download of a gathering. `SEQUENCE` SHALL count the stored changes and cancellations of the gathering, starting at 0. `DTSTAMP` and `LAST-MODIFIED` SHALL be the time of the last stored change, and `CREATED` the creation time, each in UTC as `YYYYMMDDTHHMMSSZ`. Every stored change of a gathering and its cancellation SHALL raise `SEQUENCE` by one and set the time of the last change.

#### Scenario: Stable UID
- **WHEN** the name `gathering:7:1799571600123456` is hashed in that namespace
- **THEN** the `UID` is `32d095c3-d573-5219-bd27-a2702c08aa6a`

#### Scenario: Entry after two changes
- **WHEN** an organizer changes a gathering twice and a member downloads the file
- **THEN** the file holds `SEQUENCE:2`, `DTSTAMP` and `LAST-MODIFIED` equal to the time of the second change and `CREATED` equal to the creation time, and the `UID` is the same as before the changes

#### Scenario: Cancellation raises the sequence
- **WHEN** an organizer cancels a gathering that was never changed
- **THEN** the file holds `SEQUENCE:1` and `STATUS:CANCELLED`

#### Scenario: Gathering created before the migration
- **WHEN** the migration runs on a database with an existing gathering
- **THEN** its `version` is 0 and its `changed_at` equals its `created_at`
