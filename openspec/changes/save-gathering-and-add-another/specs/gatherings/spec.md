## ADDED Requirements

### Requirement: Save and add another gathering
The form for a new gathering SHALL offer a second button "Save & add another", in German "Speichern & Weiter", after the button "Save". The form that changes a gathering SHALL NOT offer it. "Save" SHALL stay the button that the Enter key triggers and SHALL open the page of the stored gathering. The second button SHALL store the gathering and keep the form open with its title, location and description, empty first and last day and the focus on the title. The form SHALL then show a line, announced to screen readers, that names the title and the span of the stored gathering and links to its page. The line SHALL stay until the next save or refusal replaces it. Both buttons SHALL be disabled while a save runs. A refused save SHALL keep every value and mark the fields with either button.

#### Scenario: Second button only for a new gathering
- **WHEN** an organizer opens the form for a new gathering and then the form that changes a gathering
- **THEN** the first form shows "Save" followed by "Save & add another" and the second form shows "Save" without "Save & add another"

#### Scenario: Save and add another
- **WHEN** an organizer fills the form for a new gathering with title "Probenwochenende", location "Wernigerode", a description and the days 12 to 14 March 2027 and chooses "Save & add another"
- **THEN** Adunata stores the gathering, the form stays open with title, location and description unchanged, first and last day are empty, the focus is on the title and an element with `role="status"` reads the title and "March 12–14, 2027" with a link to the page of the stored gathering

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
