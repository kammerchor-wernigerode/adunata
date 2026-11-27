## ADDED Requirements

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

## MODIFIED Requirements

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

### Requirement: Confirmation
Adunata SHALL confirm a stored response on the page right away. It SHALL send no mail for a response. The confirmation and the read-only view of a member's own response SHALL show the car with its free seats, or that the member arrives without a car.

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
