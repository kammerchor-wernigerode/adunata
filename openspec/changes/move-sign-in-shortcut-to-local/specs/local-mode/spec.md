## Purpose

Lets a person or an agent start Adunata for a manual debug run on their own machine and sign in at once, without setting up an organizer or opening a mailbox.

## ADDED Requirements

### Requirement: Local mode of serve
The subcommand `serve` SHALL accept the option `--local` to turn on local mode. Every build SHALL offer it, and every setting SHALL keep its meaning. Without the option Adunata SHALL behave as without local mode. Adunata SHALL log a warning at start that local mode is on.

#### Scenario: Start in local mode
- **WHEN** an operator runs `adunata serve --local` with a valid configuration whose public base URL is on `localhost`
- **THEN** Adunata serves the client and the API as without the option and logs a warning that local mode is on

#### Scenario: Start without the option
- **WHEN** an operator runs `adunata serve` without `--local`
- **THEN** Adunata creates no sign-in link at start-up and writes no sign-in link to its log

### Requirement: Local mode only on localhost
Adunata SHALL refuse to start in local mode unless the host of the public base URL is `localhost` and the listen address is a loopback address.

#### Scenario: Public URL on another host
- **WHEN** an operator runs `adunata serve --local` with the public base URL `https://adunata.example.org`
- **THEN** Adunata exits with a non-zero status before it opens a port or touches the database and names the public base URL as the reason

#### Scenario: Listen address on every interface
- **WHEN** an operator runs `adunata serve --local` with the public base URL `http://localhost:8080` and the listen address `0.0.0.0:8080`
- **THEN** Adunata exits with a non-zero status before it opens a port or touches the database and names the listen address as the reason

### Requirement: Default first organizer
In local mode Adunata SHALL use the member with the mail address `organizer@example.org`, the first name `Ada` and the last name `Lovelace` as the first organizer if the bootstrap settings name no first organizer.

#### Scenario: Fresh database in local mode
- **WHEN** an instance without an organizer starts in local mode without bootstrap settings
- **THEN** Adunata creates `organizer@example.org` as an active organizer named Ada Lovelace

#### Scenario: Bootstrap settings win
- **WHEN** an instance without an organizer starts in local mode with the bootstrap settings for `grace@example.org`
- **THEN** Adunata creates `grace@example.org` as the first organizer and not the default member

#### Scenario: Instance already has an organizer
- **WHEN** an instance with an active organizer starts in local mode
- **THEN** Adunata leaves members and roles unchanged

### Requirement: Sign-in link at start-up
In local mode Adunata SHALL create a sign-in link for the active organizer whose member entry is the oldest and write the link to its log before it accepts requests. The link SHALL work like a link sent by mail. Adunata SHALL send no mail for it.

#### Scenario: Sign in right after the start
- **WHEN** a person starts Adunata in local mode on a fresh database and opens the link from the log within one hour
- **THEN** the person can sign in as the first organizer

#### Scenario: Agent redeems the link
- **WHEN** an agent reads the link from the log and sends its secret to the API as a sign-in link
- **THEN** Adunata starts a session for the organizer of the link

#### Scenario: Database copied from another checkout
- **WHEN** Adunata starts in local mode on a database whose active organizers are Grace and Alan, whose member entry was created after Grace's
- **THEN** the link in the log signs in Grace

#### Scenario: Frequent restarts
- **WHEN** an operator starts Adunata in local mode more often within one hour than the limit of sign-in links per address allows
- **THEN** each start still logs a working link

### Requirement: Sign-in links in the log
In local mode Adunata SHALL write the link of every sign-in link it sends by mail to its log as well. It SHALL still send the mail.

#### Scenario: Request a link in local mode
- **WHEN** an active organizer requests a sign-in link while local mode is on
- **THEN** Adunata sends the mail and writes the same link to its log

#### Scenario: Request a link without local mode
- **WHEN** an active organizer requests a sign-in link while local mode is off
- **THEN** Adunata sends the mail and writes no link to its log

### Requirement: No limit per address in local mode
In local mode Adunata SHALL send a sign-in link for every request for an active organizer, however many links it created for that address within the period. The limit per network address SHALL still apply.

#### Scenario: Request after several restarts
- **WHEN** Adunata in local mode created more sign-in links for an organizer within one hour than the limit per address allows
- **AND** that organizer requests another link through the sign-in page
- **THEN** Adunata sends the mail and writes the link to its log
### Requirement: Option --dev without effect
The subcommand `serve` SHALL accept the option `--dev`. The option SHALL have no effect. Adunata SHALL start with `--dev` as without it, apply no check of local mode and log neither a warning about local mode nor a hint to `--local`.

#### Scenario: Start with --dev on another host
- **WHEN** an operator runs `adunata serve --dev` with a valid configuration whose public base URL is `https://adunata.example.org`
- **THEN** Adunata starts, creates no sign-in link at start-up and writes no sign-in link to its log

#### Scenario: Help of serve
- **WHEN** an operator runs `adunata serve --help`
- **THEN** the help lists `--local` with a description of local mode and lists `--dev` as an option without effect
