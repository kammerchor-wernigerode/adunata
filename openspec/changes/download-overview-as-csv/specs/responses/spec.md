## ADDED Requirements

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
The file SHALL have the columns last name, first name, status, arrival, departure, beds needed, beds offered and comment, in this order. Each cell SHALL hold one plain value. Arrival and departure SHALL be wall-clock times of the instance in the form `YYYY-MM-DD HH:MM` in every language, and empty when the member stated none. Beds needed and beds offered SHALL be whole numbers, and empty when the response neither needs nor offers beds.

#### Scenario: Member needs beds
- **WHEN** Anna Becker accepted, arrives on 14 March 2027 at 18:00, leaves on 16 March 2027 at 15:00, needs 2 beds and commented "vegan"
- **THEN** her row in English reads `Becker,Anna,Accepted,2027-03-14 18:00,2027-03-16 15:00,2,,vegan`

#### Scenario: Member offers beds
- **WHEN** a member accepted and offers 3 beds
- **THEN** the beds needed cell of the row is empty and the beds offered cell holds `3`

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
- **THEN** the heading row reads `Nachname,Vorname,Status,Ankunft,Abreise,Betten gesucht,Betten angeboten,Kommentar` and an accepted invitation has the status `Zugesagt`

#### Scenario: English interface
- **WHEN** an organizer with an English interface downloads the file
- **THEN** the heading row reads `Last name,First name,Status,Arrival,Departure,Beds needed,Beds offered,Comment`

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
