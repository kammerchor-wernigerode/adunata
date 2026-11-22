## ADDED Requirements

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
