## MODIFIED Requirements

### Requirement: Invitation mail
The invitation mail SHALL name the gathering, its location and its span and SHALL contain the member's personal link. Its subject SHALL name the title and the span of the gathering, separated by a comma. It SHALL be written in the member's language. It SHALL link the privacy notice and the imprint when the operator configured them.

#### Scenario: Mail content
- **WHEN** Adunata sends an invitation to a member whose language is German
- **THEN** the mail is in German and contains the title, location, span and personal link of the gathering

#### Scenario: Subject names title and span
- **WHEN** Adunata sends an invitation for the gathering "Probenwochenende" from 12 to 14 March 2027 to a member whose language is German
- **THEN** the subject of the mail is `Einladung: Probenwochenende, 12.–14. März 2027`
