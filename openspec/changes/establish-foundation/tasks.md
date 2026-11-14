# Tasks

## 1. Repository basics

- [x] 1.1 Add `LICENSE` with the text of the AGPL-3.0 and verify that GitHub detects the license on the repository page
- [x] 1.2 Add the rules for plain Rust from decision 1 of design.md to `AGENTS.md` and verify that they name macros, generic traits, lifetimes and the reason each dependency needs
- [x] 1.3 Add a Nix flake with a development shell that provides the Rust toolchain, Node.js, PostgreSQL and Mailpit and verify that `nix develop` offers `cargo`, `node`, `psql` and `mailpit`
- [x] 1.4 Add a Compose file that runs PostgreSQL and Mailpit with the same versions and ports as the development shell and verify that `docker compose up` starts both and Mailpit's web interface answers
- [x] 1.5 Add a GitHub Actions workflow that runs formatting, lints and tests of the flake on every push to a pull request, cancels a run in progress for the same pull request and fails on any warning. Verify it on a draft pull request
- [x] 1.6 Write `docs/arc42/README.md` and the arc42 sections 1 (introduction and goals), 2 (architecture constraints), 11 (risks and technical debt) and 12 (glossary) with the skill `arc42` and verify that every relative link resolves
- [x] 1.7 Write records 0002 (Rust and React as stack) and 0003 (single-page client with API) with status Proposed following the template, link them from arc42 sections 4 and 9 and verify that both links resolve

## 2. Spike

- [x] 2.1 Create the server crate and the React client with Vite, serve the built client from the binary on one port and verify that a browser opening the root path shows the client
- [x] 2.2 Cache the Nix store between workflow runs with the cache of GitHub Actions and verify that a second run of an unchanged commit takes less than half the time of the first
- [x] 2.3 Let the binary write its OpenAPI document to a file on request, generate client code with Orval from that file and verify that Orval accepts the document and that the generated types and zod schemas handle an optional field, an enum, a date and an integer correctly
- [x] 2.4 Add one form with shadcn/ui, React Hook Form and the generated zod schema that stores a value through the API in PostgreSQL and verify that a server-side validation error appears at the field in the form
- [x] 2.5 Add the first migration, apply it on start and verify in CI that it runs against a fresh PostgreSQL
- [x] 2.6 Send one mail through an outbox to Mailpit, simulate an outage of the mail server and verify through Mailpit's API that the mail arrives after a retry
- [x] 2.7 Show one text with a plural form in German and English in the client and in the mail, add a localized date range to the mail and verify the correct plural form for the counts 0, 1 and 2 and the date format in both languages
- [x] 2.8 Try one translation catalog that the server delivers to the client and verify whether the client and the mail render from it, otherwise record why two catalogs are needed
- [x] 2.9 Add an automated accessibility check of the form with axe and verify that it reports no violation
- [ ] 2.10 Measure the peak resident memory of a Linux release build while 10 concurrent connections send the form and load the client for a fixed duration and verify that it stays below 150 MB
- [ ] 2.11 Check the spike code against the rules for plain Rust and the success criteria in design.md and present each result to the maintainer, who either accepts records 0002 and 0003 with the chosen libraries or has 0002 superseded by a record for TypeScript on server and client. Revise design.md before continuing if the spike failed

## 3. Server structure and client foundation

- [ ] 3.1 Split the server into modules for configuration, health, mail and localization, add a check that fails CI when a module uses another one past its public interface and verify the check with a deliberate violation
- [ ] 3.2 Set up TanStack Router, TanStack Query and the shadcn/ui base components in the client and verify with a client slice test that runs against the mocks Orval generates
- [ ] 3.3 Add a CI check that regenerates the OpenAPI document and the Orval output and fails when they differ from the committed files. Verify it with a deliberate change to the API
- [ ] 3.4 Write records 0007 (modular monolith) and 0008 (client libraries) as Accepted, update arc42 sections 4, 5 and 9 and verify that every link resolves
- [ ] 3.5 Describe the testing levels from design.md in arc42 section 8 and verify that each level names a test in the repository as its example

## 4. Instance configuration

- [ ] 4.1 Read every setting from environment variables and each secret from a file named by its variable. Verify with tests for the scenarios "Start with all required settings", "Read a secret from a file" and "Secret file unreadable"
- [ ] 4.2 Validate the whole configuration before opening a port or the database and report every invalid setting at once. Verify with tests for the scenarios "Several settings missing" and "Malformed value"
- [ ] 4.3 Require an HTTPS public base URL unless its host is localhost and build mail links from it. Verify with a test for the scenario "Link in a mail"
- [ ] 4.4 Write the configuration reference for operators in `docs/operating/configuration.md`, describe configuration in arc42 section 8 and verify that every setting the server reads appears in the reference

## 5. Service operation

- [ ] 5.1 Apply pending migrations on start and refuse to start against a newer schema. Verify with tests for the scenarios "Start against an empty database", "Start after an update" and "Start after a downgrade"
- [ ] 5.2 Add the health endpoint without personal data or configuration values and verify with tests for the scenarios "Healthy instance" and "Database unreachable"
- [ ] 5.3 Finish requests in progress on a termination signal and verify with a test for the scenario "Stop during a request"
- [ ] 5.4 Write structured logs to standard output without secrets, links or message bodies and verify with a test for the scenario "Log a failed mail delivery"
- [ ] 5.5 Describe persistence, migrations and logging in arc42 section 8, add the context with operators, browser users, PostgreSQL and the mail server to arc42 section 3 and verify that every link resolves

## 6. Mail delivery

- [ ] 6.1 Send mail through the configured SMTP server and require encryption unless the operator allows otherwise. Verify with tests for the scenarios "Send through the configured server" and "Server offers no encryption"
- [ ] 6.2 Store every mail in the outbox in the transaction of its action and verify with tests for the scenarios "Action fails" and "Action succeeds while the mail server is down"
- [ ] 6.3 Retry failed deliveries with increasing delays up to the configured number of attempts and resume after a restart. Verify with tests for the scenarios "Mail server comes back", "Permanent failure" and "Restart with pending mail"
- [ ] 6.4 Mark a mail as sent once the server accepts it and never resend it. Verify with a test for the scenario "Accepted mail"
- [ ] 6.5 Send every mail with a plain text part and an HTML part and verify through Mailpit's API that both parts hold every link
- [ ] 6.6 Write record 0006 (outbox for mail) as Accepted, describe mail delivery in arc42 section 8, add the quality scenario for a mail server outage to arc42 section 10 and verify that every link resolves

## 7. Localization

- [ ] 7.1 Load German and English catalogs with English as fallback and verify with a test for the scenario "Missing translation"
- [ ] 7.2 Render plural forms from the catalogs in client and mail and verify with tests for the scenario "Plural form"
- [ ] 7.3 Choose the client language from the browser, let a person switch it and remember the choice on the device. Verify with client tests for the scenarios "German browser", "Unsupported browser language" and "Switch language"
- [ ] 7.4 Write mail in the recipient's stored language or the default language of the instance and verify with tests for the scenarios "Recipient with a stored language" and "Recipient without a stored language"
- [ ] 7.5 Format dates, times and ranges per language in the time zone of the instance and verify with a test for the scenario "Date range in German"
- [ ] 7.6 Add an end-to-end test with Playwright that opens the client, switches the language and runs axe. Verify that it passes against the binary, PostgreSQL and Mailpit
- [ ] 7.7 Describe localization in arc42 section 8, add the quality scenario for plural forms to arc42 section 10 and verify that every link resolves

## 8. Packaging and deployment

- [ ] 8.1 Build the package, the OCI image and the release binary from the flake and verify that the image and the binary both start with a valid configuration and serve the client
- [ ] 8.2 Add the NixOS module with a hardened systemd service and secrets read from files. Verify with a NixOS VM test for the scenario "Install on NixOS" that the service starts with PostgreSQL and Mailpit, migrates the database and reports healthy
- [ ] 8.3 Add the example systemd unit for the release binary and verify with a test for the scenario "Install from release binary"
- [ ] 8.4 Extend the workflow so that pull requests ready for review and pushes to `main` also run the end-to-end tests and the NixOS VM test. Release tags build and publish the OCI image and the release binary. Verify each stage once
- [ ] 8.5 Add the rules for pull requests by coding agents from decision 8 of design.md to `AGENTS.md` and verify that they cover drafts, local checks before a push, one push per round of feedback, marking ready and asking after an approval
- [ ] 8.6 Write the installation guide for each supported method in `docs/operating/installation.md`, write record 0005 (Nix flake as build definition) as Accepted, write arc42 section 7 and verify that each documented installation runs as written

## 9. Integration

- [ ] 9.1 Measure the resident memory of the packaged binary with 10 simulated users and verify that it stays below 150 MB, then record the result in the quality scenario in arc42 section 10
- [ ] 9.2 Run `openspec validate establish-foundation --strict` and check every relative link in `docs/` and verify that both pass

## Workflow follow-up

- Archive the change once every task is done and the pull requests are merged.
- Write design.md and tasks.md for `add-gathering-invitations` after the spike has set the stack.
