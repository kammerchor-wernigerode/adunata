## ADDED Requirements

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
