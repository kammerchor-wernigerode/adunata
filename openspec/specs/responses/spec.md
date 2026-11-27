# responses Specification

## Purpose
Lets invited members say whether and when they attend a gathering and what they need. Gives organizers the overview they plan with.

## Requirements

### Requirement: Response status
An invitation SHALL have one of the statuses pending, accepted, tentative or declined. It SHALL start as pending. A member SHALL be able to choose accepted, tentative or declined. A declined response SHALL have no period, beds, car or free seats, also when it is stored after the deadline, and the database SHALL refuse a declined response that holds one of them.

#### Scenario: Accept
- **WHEN** a member opens the personal link and chooses accepted
- **THEN** Adunata stores the status accepted for that invitation

#### Scenario: Decline
- **WHEN** a member chooses declined
- **THEN** Adunata stores the status declined and clears the period, the beds, the car and the free seats of that response

#### Scenario: Decline after the deadline clears the car
- **WHEN** the deadline has passed and a member who accepted with a car and 3 free seats declines
- **THEN** the stored response has no car and no free seats

### Requirement: Period of presence
A member who accepts or accepts tentatively SHALL be able to state when they arrive and when they leave, in steps of 30 minutes within the span of the gathering. Without a stated period Adunata SHALL treat the member as present for the whole gathering. Adunata SHALL show these times in the time zone of the instance, which the operator configures.

#### Scenario: Times in the time zone of the instance
- **WHEN** the instance runs in the time zone Europe/Berlin and a member arrives on 14 March 2027 at 18:00 local time
- **THEN** the form and the overview show 18:00, whatever time zone the browser or the server uses

#### Scenario: Time between the steps
- **WHEN** a member sends an arrival at 18:10
- **THEN** Adunata rejects the response and marks the arrival as invalid

#### Scenario: No period stated
- **WHEN** a member accepts without stating a period
- **THEN** organizers see the member as present from the first to the last day

#### Scenario: Period outside the gathering
- **WHEN** a member enters an arrival before the first day of the gathering
- **THEN** Adunata rejects the response and marks the arrival as invalid

#### Scenario: Leave before arrival
- **WHEN** a member enters a departure that lies before the arrival
- **THEN** Adunata rejects the response and marks the departure as invalid

### Requirement: Beds
A member who accepts or accepts tentatively SHALL state whether they need beds, offer beds or need none. A member who needs or offers beds SHALL state how many, from 1 to 20.

#### Scenario: Offer beds
- **WHEN** a member states that they offer two beds
- **THEN** Adunata stores an offer of two beds for that response

#### Scenario: Need without a count
- **WHEN** a member states that they need beds but gives no count
- **THEN** Adunata rejects the response and asks for the count

### Requirement: Comment
A member SHALL be able to add a comment of up to 2000 characters to a response with any status.

#### Scenario: Comment with a decline
- **WHEN** a member declines and adds a comment
- **THEN** Adunata stores the comment with the response

### Requirement: Change a response
A member SHALL be able to change a response through the same link until the deadline for responses has passed or, without a deadline, until the first day of the gathering begins in the time zone of the instance. Adunata SHALL show the stored values when the member opens the link again. After the deadline has passed and before the first day begins, the member SHALL only be able to decline, as the requirement "Decline after the deadline" describes. After the gathering has begun the link SHALL show the response without a form.

#### Scenario: Change from tentative to accepted
- **WHEN** a member who answered tentative opens the link again and chooses accepted
- **THEN** the form shows the earlier values and Adunata stores the new status

#### Scenario: Open the link after the start
- **WHEN** a member opens the personal link on the first day of the gathering
- **THEN** Adunata shows the stored response and offers no form

#### Scenario: Change on the last day to answer
- **WHEN** today is 6 March 2027, the gathering has the first day 14 March 2027 and a deadline of 7 days and a member changes the response to accepted
- **THEN** Adunata stores the new status

### Requirement: Responses by organizers
An organizer SHALL be able to enter and change the response of any invitation with the same fields as the member, also after the gathering has begun. Adunata SHALL record whether the member or an organizer made the last change and when.

#### Scenario: Answer on a member's behalf
- **WHEN** a member accepts by phone and an organizer enters the acceptance
- **THEN** Adunata stores the acceptance and the overview shows that an organizer made the last change

#### Scenario: Member opens the link afterwards
- **WHEN** a member opens the personal link after an organizer changed the response before the gathering began
- **THEN** the form shows the values the organizer entered

### Requirement: Retention
Adunata SHALL delete comments 30 days and whole responses 365 days after the last day of a gathering. The operator SHALL be able to set each period in days or to keep the data. When the period for whole responses ends, Adunata SHALL delete the invitations of the gathering together with their responses and keep the gathering.

#### Scenario: Default retention
- **WHEN** the operator configured no retention period and a gathering ended 31 days ago
- **THEN** its responses exist without their comments

#### Scenario: Keep everything
- **WHEN** the operator set both periods to keep the data and a gathering ended two years ago
- **THEN** its responses and comments still exist

#### Scenario: Responses expire
- **WHEN** the operator configured a retention period of 90 days for responses and a gathering ended 91 days ago
- **THEN** the gathering still exists without invitations and responses

#### Scenario: Comments expire
- **WHEN** the operator configured a retention period of 30 days for comments and a gathering ended 31 days ago
- **THEN** the responses of that gathering exist without their comments

### Requirement: Confirmation
Adunata SHALL confirm a stored response on the page right away. It SHALL send no mail for a response. The confirmation and the read-only view of a member's own response SHALL show the car with its free seats, or that the member arrives without a car. They SHALL show nothing about a car for a response without a statement about a car, such as a decline.

#### Scenario: Save a response
- **WHEN** a member saves a response
- **THEN** the page states that the response is stored and shows its values

#### Scenario: Confirmation names the car
- **WHEN** a member with an English interface saves an acceptance with a car and 3 free seats
- **THEN** the confirmation names the car with 3 free seats

#### Scenario: Read-only view names the car
- **WHEN** a member who accepted with a car and 2 free seats opens the personal link after the gathering has begun
- **THEN** the page shows the status and the car with 2 free seats

### Requirement: Attendees visible to members
On the response page Adunata SHALL show a member the names of everyone invited to the same gathering who accepted or accepted tentatively, each with that status. It SHALL show only counts for declined and pending invitations. It SHALL never show another member's period, beds, car, free seats, comment or mail address.

#### Scenario: Member looks at attendees
- **WHEN** a member opens the personal link of a gathering with three accepted, one tentative and two declined invitations
- **THEN** the page lists four names with their status and states that two declined

#### Scenario: Comment of another member
- **WHEN** another invited member has added a comment
- **THEN** the page does not show that comment

#### Scenario: Car of another member
- **WHEN** another invited member accepted with a car and 4 free seats
- **THEN** neither the page nor the data the server sends for the personal link holds that car or its free seats

### Requirement: Notice on the response page
The response page SHALL state that other invited members see the member's name and status. It SHALL state at the comment field that only organizers read the comment. It SHALL state at the car question that only organizers see the car and the free seats.

#### Scenario: Member reads the notice
- **WHEN** a member opens the response form
- **THEN** the page states who sees the name, the status and the comment

#### Scenario: Notice at the car question
- **WHEN** a member with an English interface opens the response form
- **THEN** the car question carries the hint "Only organizers see your car and free seats."

### Requirement: Overview for organizers
Adunata SHALL show organizers every invitation of a gathering with member, status, period, beds, car, comment and who made the last change. The car column SHALL show the free seats for a member with a car, "no car" for a member without one and a dash for a response without a statement. It SHALL show the count per status, the total of beds needed and beds offered, the number of cars and the total of free seats. A response without a statement about a car SHALL add to neither total. It SHALL show an invitation whose mail failed for good and offer to resend it.

#### Scenario: Plan beds
- **WHEN** two members need one bed each and one member offers three beds
- **THEN** the overview shows two beds needed and three beds offered

#### Scenario: Plan rides
- **WHEN** one member arrives by car with 3 free seats, one by car with 0 free seats, one without a car and one accepted response has no statement about a car
- **THEN** the overview shows 2 cars and 3 free seats, the rows read "3 free seats", "0 free seats", "no car" and "–", and the German interface reads "kein Auto" for the member without a car

#### Scenario: Filter by status
- **WHEN** an organizer filters the overview by pending
- **THEN** the overview lists only the members who have not answered yet

### Requirement: Actions column of the overview
The overview table SHALL end with a column "Actions". For an invitation whose mail failed or whose link is revoked, this column SHALL hold the notice and, while the gathering is neither cancelled nor ended, the action "Resend". The name cell SHALL hold only the linked name. In the card layout below a container width of 40rem, the actions SHALL be one more labelled line of the card, which a row with an empty actions cell SHALL not show.

#### Scenario: Last column names the actions
- **WHEN** an organizer opens the overview of a gathering
- **THEN** the last column heading reads "Actions" in English and "Aktionen" in German

#### Scenario: Failed mail in the actions column
- **WHEN** the invitation mail of a member failed and the gathering is neither cancelled nor ended
- **THEN** the last cell of the member's row shows "mail failed" and the button "Resend" while the name cell shows only the linked name

#### Scenario: Revoked link in the actions column
- **WHEN** the link of a member is revoked and the gathering is neither cancelled nor ended
- **THEN** the last cell of the member's row shows "link revoked" and the button "Resend"

#### Scenario: Resend accepted
- **WHEN** an organizer resends an invitation from the actions column and the server accepts it
- **THEN** the button "Resend" disappears from that row

#### Scenario: Resend refused
- **WHEN** an organizer resends an invitation from the actions column and the server refuses it
- **THEN** the button "Resend" stays and the cell names the failure

#### Scenario: Notice without action on a closed gathering
- **WHEN** the gathering has ended or is cancelled and a member's mail failed
- **THEN** the actions cell of that row shows the notice with its label "Actions" and no button "Resend"

#### Scenario: Row without actions
- **WHEN** a member's mail did not fail and the link is active
- **THEN** the actions cell of that row is empty and carries no label, so the card layout shows no actions line for it

### Requirement: Remind action on the overview
While the first day of a gathering has not begun in the time zone of the instance, the gathering is not cancelled and at least one invitation is pending with an active link, the overview SHALL offer a button next to the status filter that names the number of these members, "Remind N members" in English and "N Mitglieder erinnern" in German. The overview SHALL take whether the gathering has begun from the server, not from the clock of the browser. A click SHALL open a dialog that names the number of members, says that each of them receives a new link and that the old link stops working, and offers to confirm or to go back. Adunata SHALL send nothing before the organizer confirms. After the server accepts, the dialog SHALL close, the overview SHALL reload and the page SHALL show the number of reminders queued. If the server refuses or fails, the dialog SHALL stay open and name the failure. The button and the dialog SHALL be shadcn components.

#### Scenario: Button names the count
- **WHEN** an organizer opens the overview of an open gathering with three pending invitations, one of them with a revoked link
- **THEN** the overview shows the button "Remind 2 members"

#### Scenario: Go back
- **WHEN** an organizer clicks the button and then goes back in the dialog
- **THEN** the dialog closes and Adunata queues no mail

#### Scenario: Confirm
- **WHEN** an organizer clicks the button and confirms in the dialog
- **THEN** the dialog closes, the overview reloads and the page shows that 2 reminders were sent

#### Scenario: Server refuses
- **WHEN** an organizer confirms and the server refuses or fails
- **THEN** the dialog stays open and names the failure

#### Scenario: Nobody to remind
- **WHEN** no invitation of the gathering is pending with an active link
- **THEN** the overview shows no remind button

#### Scenario: Closed gathering
- **WHEN** the gathering is cancelled or has ended
- **THEN** the overview shows no remind button

#### Scenario: Gathering has begun
- **WHEN** the server reports that the gathering has begun and one invitation is pending with an active link
- **THEN** the overview shows no remind button

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

### Requirement: Overview as a CSV file
Adunata SHALL let an organizer download the overview of a gathering as a CSV file. The file SHALL hold a heading row and one row per invitation of the gathering, in the order of the overview, whatever status filter the page shows. It SHALL hold no row of totals. Without an organizer session the download SHALL fail with 401, and for an unknown gathering with 404.

#### Scenario: Every invitation
- **WHEN** an organizer filters the overview of a gathering with three invitations by pending and downloads the file
- **THEN** the file holds the heading row and three rows, one per invitation, and no row of totals

#### Scenario: Gathering without invitations
- **WHEN** an organizer downloads the file of a gathering without invitations
- **THEN** the file holds only the heading row

#### Scenario: No session
- **WHEN** a request without an organizer session asks for the file of a gathering
- **THEN** Adunata answers 401 and sends no file

#### Scenario: Unknown gathering
- **WHEN** an organizer asks for the file of a gathering that does not exist
- **THEN** Adunata answers 404

### Requirement: Columns of the CSV file
The file SHALL have the columns last name, first name, status, arrival, departure, beds needed, beds offered, free seats and comment, in this order. Each cell SHALL hold one plain value. Arrival and departure SHALL be wall-clock times of the instance in the form `YYYY-MM-DD HH:MM` in every language, and empty when the member stated none. Beds needed and beds offered SHALL be whole numbers, and empty when the response neither needs nor offers beds. Free seats SHALL be a whole number for a member with a car, 0 included, and empty without a car or without a statement.

#### Scenario: Member needs beds
- **WHEN** Anna Becker accepted, arrives on 14 March 2027 at 18:00, leaves on 16 March 2027 at 15:00, needs 2 beds, has no car and commented "vegan"
- **THEN** her row in English reads `Becker,Anna,Accepted,2027-03-14 18:00,2027-03-16 15:00,2,,,vegan`

#### Scenario: Member offers beds
- **WHEN** a member accepted and offers 3 beds
- **THEN** the beds needed cell of the row is empty and the beds offered cell holds `3`

#### Scenario: Full car
- **WHEN** a member accepted with a car and 0 free seats
- **THEN** the free seats cell holds `0`

#### Scenario: Pending invitation
- **WHEN** a member has not answered
- **THEN** the row holds the names, the status Pending and empty cells otherwise

#### Scenario: Only an arrival stated
- **WHEN** a member stated an arrival on 14 March 2027 at 18:00 and no departure
- **THEN** the arrival cell holds `2027-03-14 18:00` and the departure cell is empty

### Requirement: Language of the CSV file
The column headings and the status values of the file SHALL be in the language that the request names, which the client sets to the language of its interface. For a missing or unsupported language the file SHALL be in English.

#### Scenario: German interface
- **WHEN** an organizer with a German interface downloads the file
- **THEN** the heading row reads `Nachname,Vorname,Status,Ankunft,Abreise,Betten gesucht,Betten angeboten,Freie Plätze,Kommentar` and an accepted invitation has the status `Zugesagt`

#### Scenario: English interface
- **WHEN** an organizer with an English interface downloads the file
- **THEN** the heading row reads `Last name,First name,Status,Arrival,Departure,Beds needed,Beds offered,Free seats,Comment`

#### Scenario: Unsupported language
- **WHEN** the request names the language `fr`
- **THEN** the headings and statuses are in English

### Requirement: Format of the CSV file
The file SHALL follow RFC 4180 with a comma as separator and CRLF between rows, encoded as UTF-8 that starts with a byte order mark. A cell that holds a comma, a double quote, a CR or an LF SHALL be enclosed in double quotes, with each double quote inside doubled.

#### Scenario: Byte order mark
- **WHEN** an organizer downloads the file
- **THEN** its first three bytes are `EF BB BF` and its rows end with CR LF

#### Scenario: Comment with special characters
- **WHEN** a comment reads `Late, "maybe"` followed by a line break and `Bring a tent`
- **THEN** its cell reads `"Late, ""maybe""` followed by an LF, `Bring a tent"` and a spreadsheet shows the comment unchanged

### Requirement: Guard against CSV injection
A cell whose value starts with `=`, `+`, `-`, `@`, a tab or a carriage return SHALL get an apostrophe in front before quoting.

#### Scenario: Formula in a comment
- **WHEN** a comment reads `=HYPERLINK("x")`
- **THEN** its cell reads `"'=HYPERLINK(""x"")"`

#### Scenario: Name starting with a minus
- **WHEN** a last name reads `-Smith`
- **THEN** its cell reads `'-Smith`

#### Scenario: Comment starting with a tab
- **WHEN** a comment starts with a tab followed by `note`
- **THEN** its cell starts with an apostrophe followed by the tab

### Requirement: Name of the CSV file
The file name SHALL be the title of the gathering, a space, its first day as `YYYY-MM-DD` and `.csv`. Each character of the title that is a control character or one of `/ \ : * ? " < > |` SHALL become `_`. The response SHALL name the file as attachment with an ASCII fallback, in which every other non-ASCII character also becomes `_`, and with the full name encoded as UTF-8.

#### Scenario: Title with umlaut, slash and quote
- **WHEN** an organizer downloads the file of the gathering `Probe "Süd"/Nord` with the first day 14 March 2027
- **THEN** the browser saves it as `Probe _Süd__Nord 2027-03-14.csv`, and the ASCII fallback in the header is `Probe _S_d__Nord 2027-03-14.csv`

### Requirement: Download link on the overview
The overview SHALL offer a link that downloads the file in the language of the interface, also for a cancelled or ended gathering. Next to it the page SHALL show, without hover or click, the notice "The file holds personal data. Adunata does not delete it, so delete it once you no longer need it." in English and "Die Datei enthält personenbezogene Daten. Adunata löscht sie nicht, also lösche sie, sobald du sie nicht mehr brauchst." in German. The link SHALL be a shadcn component.

#### Scenario: Link and notice
- **WHEN** an organizer with an English interface opens the overview of a gathering
- **THEN** the page shows the link "Download CSV" and the English notice next to it

#### Scenario: German interface
- **WHEN** an organizer with a German interface opens the overview
- **THEN** the page shows the link "CSV herunterladen", its address names the language `de`, and the German notice is next to it

#### Scenario: Closed gathering
- **WHEN** the gathering is cancelled or has ended
- **THEN** the overview still shows the link and the notice

### Requirement: Decline after the deadline
Once the deadline for responses has passed and the first day has not begun, the personal link SHALL report the state `deadlinePassed`. The page SHALL show the stored response, the attendees, a notice that the deadline has passed, and a form that offers only the status declined with the optional comment. A member without a response SHALL be able to decline too. Adunata SHALL store a decline from the member, with its comment. Adunata SHALL refuse every other status from the member with status 422 and the error code `deadlinePassed` on the field `gathering` and store nothing. The states SHALL take precedence in this order: cancelled, ended, begun, `deadlinePassed`, open. The response form of organizers SHALL stay as it is.

#### Scenario: Open the link after the deadline
- **WHEN** today is 7 March 2027 and a member who accepted opens the link of a gathering with the first day 14 March 2027 and a deadline of 7 days
- **THEN** the page shows the acceptance, the attendees, the notice and a form with only the status declined and the comment

#### Scenario: Late decline
- **WHEN** the deadline has passed, the gathering has not begun and a member declines with the comment "Krank"
- **THEN** Adunata stores the status declined with the comment and the change by the member

#### Scenario: Decline without an earlier answer
- **WHEN** the deadline has passed and a member who never answered declines
- **THEN** Adunata stores the status declined

#### Scenario: Accept after the deadline
- **WHEN** the deadline has passed and a request `PUT /api/invitation/response` through the personal link sends the status accepted or tentative
- **THEN** Adunata answers with status 422 and `{"errors":[{"field":"gathering","code":"deadlinePassed"}]}` and the stored response stays as it was

#### Scenario: Begun wins over the deadline
- **WHEN** the deadline has passed and the first day has begun and a member opens the link
- **THEN** the link reports the state `begun` and a decline is refused with the error code `begun`

#### Scenario: Cancelled wins over the deadline
- **WHEN** the deadline of a cancelled gathering has passed and a member opens the link
- **THEN** the link reports the state `cancelled`

#### Scenario: Organizer after the deadline
- **WHEN** the deadline has passed and an organizer records an acceptance on a member's behalf
- **THEN** Adunata stores the acceptance

### Requirement: Last day to answer on the response page
While the link is open and the gathering has a deadline, the response page SHALL name the last day to answer in the member's language. The page SHALL take the date from the field `lastDayToAnswer` of the gathering on the page, which is null without a deadline.

#### Scenario: Page names the last day
- **WHEN** a member opens the link of a gathering with the first day 14 March 2027 and a deadline of 7 days before 7 March 2027
- **THEN** the page names 6 March 2027 as the last day to answer

#### Scenario: No deadline
- **WHEN** a member opens the link of a gathering without a deadline
- **THEN** the page names no last day to answer

### Requirement: Deadline on the page of the gathering
The page of a gathering with a deadline SHALL show organizers the last day to answer next to its days. Once the deadline has passed, by the field `deadlinePassed` of the server, the line SHALL say that responses closed after that day.

#### Scenario: Deadline ahead
- **WHEN** an organizer opens the page of a gathering whose deadline has not passed
- **THEN** the page names the last day to answer

#### Scenario: Deadline passed
- **WHEN** an organizer opens the page of a gathering whose deadline has passed
- **THEN** the page says that responses closed after the last day to answer

### Requirement: Remind action after the deadline
The overview SHALL hide the remind button once the server reports `deadlinePassed` for the gathering, in addition to the cases of "Remind action on the overview".

#### Scenario: Deadline has passed
- **WHEN** the server reports that the deadline of the gathering has passed and one invitation is pending with an active link
- **THEN** the overview shows no remind button

### Requirement: Car and free seats
A member who accepts or accepts tentatively SHALL state whether they arrive by car. A member who arrives by car SHALL state how many seats are free, not counting the driver, as a whole number from 0 to 8. 0 SHALL still count as a car. Without a car the response SHALL hold no count. Adunata SHALL reject a missing answer, a missing count and a count outside 0 to 8 on the server and in the form and mark the field. The API SHALL carry the answer in the field `car`, a boolean or null, and the count in the field `freeSeats`, a number or null. A response stored before this change SHALL have no statement about a car, and its form SHALL open with the question unanswered.

#### Scenario: Car with free seats
- **WHEN** a member accepts and states a car with 3 free seats
- **THEN** Adunata stores `car` true and `freeSeats` 3 for that response

#### Scenario: Full car
- **WHEN** a member accepts tentatively and states a car with 0 free seats
- **THEN** Adunata stores `car` true and `freeSeats` 0

#### Scenario: No car
- **WHEN** a member accepts, states no car and sends `freeSeats` 2
- **THEN** Adunata stores `car` false and no count

#### Scenario: Coming without an answer
- **WHEN** a member sends `{"status":"accepted","beds":"none"}` without `car`
- **THEN** Adunata answers 422 with `{"errors":[{"field":"car","code":"required"}]}` and the form marks the car question

#### Scenario: Car without a count
- **WHEN** a member accepts with `car` true and no `freeSeats`
- **THEN** Adunata answers 422 with the error `required` on the field `freeSeats`

#### Scenario: Count out of range
- **WHEN** a member accepts with `car` true and `freeSeats` 9 or -1
- **THEN** Adunata answers 422 with the error `invalid` on the field `freeSeats` and stores nothing

#### Scenario: Response from before the change
- **WHEN** a member opens the link of an accepted response that has no statement about a car
- **THEN** neither choice of the car question is selected, and saving it as accepted without an answer fails with the error `required` on `car`

#### Scenario: Organizer states the car
- **WHEN** an organizer enters an acceptance with a car and 2 free seats for a member
- **THEN** Adunata stores the car and the 2 free seats for that response
