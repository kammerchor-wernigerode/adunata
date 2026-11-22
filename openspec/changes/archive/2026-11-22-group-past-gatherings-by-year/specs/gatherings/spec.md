# Spec Delta

## MODIFIED Requirements

### Requirement: Gathering list
Adunata SHALL show organizers upcoming gatherings first, ordered by first day, and past gatherings separately. Adunata SHALL group past gatherings by the year of their first day, latest year first and latest gathering first within a year. Each year SHALL show its number of gatherings and SHALL open and close on its own. The client SHALL remember on the device which years the organizer opened or closed. A year without a remembered choice SHALL start open only if it is the latest year.

#### Scenario: Gathering has ended
- **WHEN** the last day of a gathering has passed
- **THEN** Adunata lists the gathering among past gatherings

#### Scenario: Years in order
- **WHEN** an organizer opens the gathering list with past gatherings whose first days lie in 2024, 2025 and 2026
- **THEN** Adunata shows the years 2026, 2025 and 2024 in this order as headings below "Past", each with the number of its gatherings

#### Scenario: Only the latest year starts open
- **WHEN** an organizer opens the gathering list on a device that remembers no choice, with past gatherings in 2024 and 2025 and none in the current year
- **THEN** Adunata shows the gatherings of 2025 with the latest first and keeps 2024 closed

#### Scenario: Several years open at once
- **WHEN** the year 2025 is open and the organizer opens the year 2024
- **THEN** Adunata shows the gatherings of both years

#### Scenario: Close a year
- **WHEN** the organizer closes an open year
- **THEN** Adunata hides its gatherings and still shows the year with its number of gatherings

#### Scenario: Remember open years
- **WHEN** an organizer closes 2025, opens 2024, leaves the gathering list and comes back on the same device
- **THEN** Adunata shows 2025 closed and 2024 open

#### Scenario: New latest year
- **WHEN** the device remembers choices for 2024 and 2025 and a gathering in 2026 has ended since
- **THEN** Adunata shows 2026 open and 2024 and 2025 as remembered

#### Scenario: Gathering over New Year
- **WHEN** a past gathering runs from 30 December 2025 to 2 January 2026
- **THEN** Adunata lists it in the year 2025

#### Scenario: Number of gatherings
- **WHEN** a year holds one past gathering and another year holds two
- **THEN** Adunata shows "1 gathering" and "2 gatherings" in English and "1 Treffen" and "2 Treffen" in German

#### Scenario: Cancelled gathering in a year
- **WHEN** an open year holds a cancelled past gathering
- **THEN** its row shows the badge for a cancelled gathering and its date line with the year

#### Scenario: No past gatherings
- **WHEN** an organizer opens the gathering list without past gatherings
- **THEN** Adunata shows no year below "Past" and shows the message it shows for an empty section
