## MODIFIED Requirements

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

## ADDED Requirements

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
