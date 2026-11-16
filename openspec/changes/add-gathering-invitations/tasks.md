# Tasks

## 1. Configuration, time and request protection

- [x] 1.1 Read the required setting `ADUNATA_TIME_ZONE` and the optional settings `ADUNATA_TRUSTED_PROXIES`, `ADUNATA_RETENTION_COMMENT_DAYS`, `ADUNATA_RETENTION_RESPONSE_DAYS`, `ADUNATA_PRIVACY_URL`, `ADUNATA_IMPRINT_URL` and the three `ADUNATA_FIRST_ORGANIZER_*` settings with their defaults from design.md. Verify with tests for the scenarios "Unknown time zone" and "Several settings missing" with the time zone among them, and with tests that a first organizer mail address without both names and a retention value other than a whole number or `never` are rejected
- [x] 1.2 Add `jiff` with the feature `tzdb-bundle-always` and one function in `configuration` that turns an instant into the date and wall-clock time of the instance. State in the commit why `jiff` is needed. Verify with unit tests around both daylight saving changes in `Europe/Berlin` and with a test that the release binary resolves the zone in the OCI image
- [x] 1.3 Add one layer that sets `Referrer-Policy`, `Content-Security-Policy`, `X-Content-Type-Options` on every response and `Cache-Control: no-store` on every API response. Verify with a server slice test that reads the headers of the client's entry page and of an API response, and with the end-to-end test that the client loads without a violation of the policy
- [x] 1.4 Add the middleware on `/api` that rejects a request with a method other than `GET` or `HEAD` whose `Origin` differs from the public URL, and let the Vite proxy set `Origin` to the public URL. Verify with tests for the scenario "Request from another origin" and with a request through the Vite server in development
- [x] 1.5 Add `getrandom` 0.4 and `sha2` 0.11 with one function for a new secret and one for its hash, state in the commit that both are already in the lock file, and verify with a unit test that a secret has 64 hexadecimal characters and that its hash is stable
- [x] 1.6 Add the new settings to `docs/operating/configuration.md`, set `ADUNATA_TIME_ZONE` and `ADUNATA_TRUSTED_PROXIES` in every example of `docs/operating/installation.md`, describe time and the security headers in arc42 section 8, write record 0012 (wall-clock times in the instance time zone) as Accepted and list it in arc42 section 9. Verify that every setting the server reads appears in the reference and that every link resolves

## 2. Mail

- [x] 2.1 Add the nullable columns `member_id` and `invitation_id` to the outbox, store the bare address in `recipient` and add the display name at delivery. Verify with a test that a mail with a display name arrives in Mailpit with that name
- [x] 2.2 Clear recipient, subject and both bodies in the statement that marks a mail as sent or failed. Verify with tests for the scenarios "Accepted mail" and "Failed mail" of the mail-delivery delta
- [x] 2.3 Move the escaping of entered values from `spike` to `mail` and apply it to every HTML part. Verify with a test for the scenario "Title with markup"
- [x] 2.4 Write record 0013 (mail loses its content once its delivery ends), add a line to record 0006 that points to it, list it in arc42 section 9 and add the rules for content and escaping to mail delivery in arc42 section 8. Verify that every link resolves

## 3. Client foundation

- [x] 3.1 Replace Geist with Literata and Source Sans 3 from fontsource, set the slate primary colors and values for `--input` and `--ring` that reach at least 3:1 in both schemes, draw the focus ring at full opacity and move the variant `dark` and the dark tokens to `prefers-color-scheme: dark`. Ship the copyright notice and the text of the SIL Open Font License of both fonts with the built client, so the binary, the OCI image and the release archive carry them. Verify that the license texts are served next to the fonts, and verify with an end-to-end check that computes the contrast of a control border and the focus outline against their background in light and dark mode
- [x] 3.2 Add the page frame with header, main heading, page title per route, focus on the main heading after a route change and a footer that links the privacy notice and the imprint when they are set. Expose both addresses to the client through the API. Verify with client slice tests for the scenarios "Both addresses set" and "No address set" and for the focus after a route change
- [x] 3.3 Add a status badge with an `aria-hidden` icon from `lucide-react` and its text for each of the four statuses. Verify with a client slice test that the accessible name of each badge is the status text alone
- [x] 3.4 Describe the user interface concept from design.md in arc42 section 8, including the patterns for `aria-invalid`, `aria-describedby` and `role="status"`, and verify that every link resolves

## 4. Members and the organizer role

- [x] 4.1 Add the module `members` with its table, the unique mail address ignoring case, the language, the active flag and the organizer role. Verify with tests for the scenarios "Add a member", "Duplicate address", "Change the mail address" and "Deactivate a member"
- [x] 4.2 Add the one function that revokes the role, deactivates or deletes a member and locks the active organizers with `FOR UPDATE`. Verify with tests for the scenarios "Revoke the last organizer" and "Deactivate or delete the last organizer" and with a test that two concurrent revocations leave one active organizer
- [x] 4.3 Delete a member with cascades to everything that references the member. Verify with tests for the scenarios "Delete a member" and "Member deleted while a mail waits" once the referencing tables exist, and extend the test in each later group that adds such a table
- [x] 4.4 Add `members` to arc42 section 5 and the terms member and organizer role to the glossary where they changed. Verify that every link resolves

## 5. Sign-in and sessions

- [x] 5.1 Add the module `access` with sign-in links, their mail in both languages and the confirmation page with a button. Redeem a link in one `UPDATE`. Verify with tests for the scenarios "Successful sign-in", "Link opened by a mail scanner", "Link used twice", "Expired link" and "Unknown address"
- [x] 5.2 Add sessions with the cookie name and attributes from design.md, including the port suffix on `localhost`, and the extractor for organizer routes. Verify with tests for the scenarios "Sign out", "Session expires", "Role revoked" and "Request without a session", and with a test that two instances on `localhost` with different ports keep separate sessions
- [x] 5.3 Add the limits per mail address and per network address with trusted proxies. Verify with tests for the scenarios "Repeated requests", "Many requests from one network address", "Request through a trusted proxy" and "Header from an untrusted client"
- [x] 5.4 Create the first organizer at start-up and add the subcommand `grant-organizer`. Verify with tests for the scenarios "Set up a new instance", "Instance without any organizer", "Instance already has an organizer" and "Every organizer mailbox lost"
- [x] 5.5 Add the server slice test that walks every operation of the OpenAPI document and expects status 401 without a session outside the allowlist from design.md. Verify it with a deliberately unprotected route
- [x] 5.6 Add the client pages for sign-in, confirmation and sign-out and the member list with search, filter, editing, deactivation, deletion and the organizer role. Verify with client slice tests with axe for each page and for the scenario "Search"
- [x] 5.7 Write record 0010 (sign-in by mail link) as Accepted, list it in arc42 sections 4 and 9, add `access` to arc42 section 5, add the sign-in scenario to the new arc42 section 6 and list it in `docs/arc42/README.md`, describe authentication and authorization in arc42 section 8, add the scenarios "Unknown address at sign-in", "Request from another origin" and "Organizer route without a session" to arc42 section 10, show the trusted proxy in arc42 section 7, add organizer session, sign-in link and trusted proxy to the glossary and describe `grant-organizer` in the operating documentation. Verify that every link resolves

## 6. Gatherings

- [x] 6.1 Add the module `gatherings` with title, location, optional description, first and last day and its state. Verify with tests for the scenarios "Create a gathering", "Last day before first day" and "Gathering has ended"
- [x] 6.2 Let organizers edit a gathering that has not ended. Verify with a test for the scenario "Shorten a gathering" once responses exist
- [x] 6.3 Add the client pages for the gathering list and for creating and editing a gathering. Verify with client slice tests with axe and with a test that upcoming gatherings come first ordered by first day
- [x] 6.4 Add `gatherings` to arc42 section 5 and verify that every link resolves

## 7. Invitations and personal links

- [x] 7.1 Add the module `invitations` with invitations, the hash of each secret and the invitation mail in both languages with the links to privacy notice and imprint. Verify with tests for the scenarios "Invite everyone", "Invite a subset", "Member added later", "Ended gathering" and "Mail content"
- [x] 7.2 Resolve an invitation from the header `Authorization: Bearer` through an extractor for `responses`. Verify with tests for the scenarios "Open a valid link", "Guessed link", "Open after the gathering" and "Request with a personal link"
- [x] 7.3 Add resend and revoke. Verify with tests for the scenarios "Resend", "Old link after resend" and "Revoke without resend"
- [x] 7.4 Cancel a gathering together with the mail to every invited member in one transaction. Verify with tests for the scenarios "Cancellation mail", "Cancelled gathering takes no invitations" and "Member opens a cancelled gathering"
- [x] 7.5 Add the client actions to invite, resend, revoke and cancel and the page that reads the secret from the fragment. Verify with client slice tests with axe and with a test that the page sends the secret only in the header
- [x] 7.6 Write record 0011 (personal links in the URL fragment) as Accepted, list it in arc42 sections 4 and 9, add `invitations` to arc42 section 5, add the invitation scenario to arc42 section 6, describe personal links in arc42 section 8, replace the reason under logging, add the scenarios "Guessed link" and "Personal link absent from logs" to arc42 section 10 and update the risk on date ranges in arc42 section 11. Verify that every link resolves

## 8. Responses and the overview

- [x] 8.1 Add the module `responses` with status, period, beds, comment and who changed it last. Verify with tests for the scenarios "Accept", "Decline", "Period outside the gathering", "Leave before arrival", "Time between the steps", "Offer beds", "Need without a count", "Comment with a decline" and a test that rejects 21 beds
- [x] 8.2 Let members change a response until the gathering begins and organizers at any time. Verify with tests for the scenarios "Change from tentative to accepted", "Open the link after the start", "Answer on a member's behalf" and "Member opens the link afterwards"
- [x] 8.3 Show members the attendees and the counts. Verify with tests for the scenarios "Member looks at attendees" and "Comment of another member"
- [ ] 8.4 Add the overview with totals, beds, filter, the mark for periods outside a changed span and failed invitation mail with resend. Verify with tests for the scenarios "Plan beds", "Filter by status", "No period stated", "Shorten a gathering" and a test that a failed mail appears at its invitation
- [ ] 8.5 Add the response form with radio rows for status and beds, a shadcn `Calendar` and a `Select` for arrival and departure, the notices on visibility and the confirmation through `role="status"`. Add `react-day-picker` and `date-fns` and state in the commit that the calendar component needs them. Verify with tests for the scenarios "Times in the time zone of the instance", "Save a response" and "Member reads the notice", and with a client slice test that operates the calendar by keyboard and checks the labels a screen reader announces
- [ ] 8.6 Add the narrow layout of the overview and verify with a client slice test that every column of a member is present below 40rem
- [ ] 8.7 Add the hourly task that clears comments, deletes expired invitations with their responses through `invitations` and ended sessions and sign-in links through `access`. Verify with tests for the scenarios "Default retention", "Keep everything", "Responses expire" and "Comments expire" and with a test that ended sessions and used links are gone
- [ ] 8.8 Add `responses` to arc42 section 5, add the answering scenario to arc42 section 6, describe retention of personal data in arc42 section 8, add the scenario "Expiring comments" to arc42 section 10 and add period of presence, tentative and retention period to the glossary. Verify that every link resolves

## 9. Removing the spike and integration

- [ ] 9.1 Remove the module `spike`, its client form and its tests, drop its table with a migration and move the mail outage scenario in arc42 section 10 to the invitation. Verify that the server and client build without warnings and that no file names `spike` any more
- [ ] 9.2 Write the end-to-end test that signs in through Mailpit, creates a gathering, invites a member, reads the link from Mailpit and answers through it. Run axe in light and dark mode on the response form and the overview and assert that neither scrolls horizontally at 390 px with 200 % text. Name it as the end-to-end example in arc42 section 8 and add the scenario "Member form at 200 % on a phone" to arc42 section 10
- [ ] 9.3 Point the memory check in `flake.nix` at the response form with a link it gets in a set-up step, measure under load and record the result in arc42 section 10. Verify that it stays below 150 MB
- [ ] 9.4 Update arc42 sections 1 and 3 for the new capabilities and partners, run `openspec validate add-gathering-invitations --strict`, the NixOS VM test and a check of every relative link in `docs/`, and verify that all pass

## Workflow follow-up

- Run `/review-change` on the implementation plan again if a group changes the design.
- Archive the change on `main` once every task is done and the pull requests are merged.
- Propose the change `instance-branding`.
