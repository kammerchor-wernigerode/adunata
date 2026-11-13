# Spec Delta

## Purpose

Lets organizers describe the gatherings an ensemble meets for, such as a rehearsal weekend, a rehearsal camp during the week or a concert. Members can then be invited to them.

## ADDED Requirements

### Requirement: Gathering data
A gathering SHALL have a title, a location, a first day and a last day. The description SHALL be optional. The last day SHALL NOT be before the first day.

#### Scenario: Create a gathering
- **WHEN** an organizer creates a gathering with title, location, first day and last day
- **THEN** Adunata stores the gathering and lists it among upcoming gatherings

#### Scenario: Last day before first day
- **WHEN** an organizer enters a last day that lies before the first day
- **THEN** Adunata rejects the gathering and marks the last day as invalid

### Requirement: Edit gatherings
An organizer SHALL be able to change every field of a gathering that has not ended. Existing responses SHALL keep their values when the span changes.

#### Scenario: Shorten a gathering
- **WHEN** an organizer moves the first day of a gathering one day later
- **THEN** Adunata keeps existing responses and marks each response whose period lies partly outside the new span

### Requirement: Cancel gatherings
An organizer SHALL be able to cancel a gathering that has not ended. A cancelled gathering SHALL stay visible to organizers as cancelled. Adunata SHALL inform every invited member by mail. Its invitation links SHALL show that it is cancelled and accept no responses.

#### Scenario: Cancellation mail
- **WHEN** an organizer cancels a gathering
- **THEN** Adunata sends every invited member a mail in the member's language that names the cancelled gathering

#### Scenario: Member opens a cancelled gathering
- **WHEN** a member opens the invitation link of a cancelled gathering
- **THEN** Adunata shows that the gathering is cancelled and offers no form

### Requirement: Gathering list
Adunata SHALL show organizers upcoming gatherings first, ordered by first day, and past gatherings separately.

#### Scenario: Gathering has ended
- **WHEN** the last day of a gathering has passed
- **THEN** Adunata lists the gathering among past gatherings
