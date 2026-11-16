# Design

## Context

See [proposal.md](proposal.md) for the motivation and the [specs](specs/) for the requirements.

The foundation is in place. One binary serves a JSON API and the embedded React client, see [ADR 0003](../../../docs/decisions/0003-single-page-client-with-api.md). Every mail goes through the outbox, see [ADR 0006](../../../docs/decisions/0006-outbox-for-mail.md). The server splits into modules whose boundaries the compiler checks, see [ADR 0007](../../../docs/decisions/0007-modular-monolith.md). The module `spike` and its form exercise the stack from client to mail. Feature modules replace it.

These constraints shape the approach:

- At most about 10 people use one instance at the same time. One instance runs as one process.
- Operators often have no operations experience. Running Adunata in Docker or as a systemd service needs nothing beyond settings. A setting without a safe default is required, as [instance-configuration](../../specs/instance-configuration/spec.md) states.
- Logs carry no personal data and no links that grant access, as arc42 section 8 states under logging.
- The outbox stores the full content of every mail. An invitation mail contains a personal link.
- Operators run Adunata behind a reverse proxy. Its access log records the path and query of every request, which Adunata cannot control. In the documented container setup the proxy reaches Adunata through a Docker network, not through loopback.
- Other ensembles install Adunata with their own logo and colors. Branding per instance follows in its own change. This change must not block it.

## Goals / Non-Goals

**Goals:**

- No secret that grants access appears in a request line, a log, a referrer or the database in plain form.
- Organizer functions are closed to any request without a valid organizer session. A test proves that no organizer route is reachable without one.
- Times read the same in the form, the overview and every mail, whatever time zone the browser or the server uses.
- The member form works with keyboard, screen reader and 200 % text on a 390 px phone.
- Personal data expires by default.
- The client styles every component through semantic tokens only, so a later change can set the brand color per instance.

**Non-Goals:**

- Branding per instance, meaning a configurable logo and colors. The change `instance-branding` follows.
- A rate limit that survives a restart or spans several processes.
- Rendering pages without JavaScript.
- Statistics about past gatherings. Expired responses are deleted, not kept in another form.
- Reminders, deadlines and every other item that the proposal puts out of scope.

## Decisions

Decisions 1 to 4 are drafts for records in `docs/decisions/`. None of them supersedes an earlier record. Decision 4 settles the retention of mail content, which [ADR 0006](../../../docs/decisions/0006-outbox-for-mail.md) leaves open. That record gains a line that points to the new one. All four records are Accepted when the change lands. Each record states its consequences in its own section, as the template requires.

### 1. Sign-in by mail link

Record: `0010-sign-in-by-mail-link.md`

An organizer enters a mail address. If it belongs to an active organizer and the limits allow it, Adunata creates a sign-in link and puts a mail with it into the outbox. The response is the same in every case. The link opens a page with a button. Only the button's request redeems the link and starts a session. A program that checks mail for malicious content opens the page but never presses the button, so it neither uses up the link nor signs in. Redeeming is one statement, `UPDATE … SET used_at = now() WHERE used_at IS NULL AND expires_at > now() RETURNING …`, so two clicks cannot both succeed. The link works in any browser, because organizers often request it on a laptop and open the mail on a phone. A forwarded link therefore signs in whoever opens it within the hour.

A session is a random 256-bit identifier in a cookie. Outside `localhost` the cookie is named `__Host-adunata_session` and carries `Secure`, `HttpOnly`, `SameSite=Strict` and `Path=/`. The prefix makes browsers refuse the cookie from any other host, so a sibling subdomain cannot plant one. On `localhost` the cookie drops the prefix and `Secure`, because browsers reject both on plain HTTP. Browsers scope cookies by host and ignore the port. Worktrees that run side by side on `localhost` with their own ports, see [ADR 0009](../../../docs/decisions/0009-optional-worktree-tooling.md), would otherwise overwrite each other's session. On `localhost` the cookie name therefore ends with the port of `ADUNATA_PUBLIC_URL`, such as `adunata_session_8081`. The database stores only a SHA-256 hash of the identifier, with the member and the end of the session 14 days after sign-in. Each request that needs an organizer loads the session together with the member. The request fails unless the member is active and holds the organizer role. Revoking the role or deactivating the member therefore ends every session at the next request without extra bookkeeping. Signing out deletes the session.

One middleware on `/api` rejects every request with a method other than `GET` or `HEAD` whose `Origin` header differs from the origin of `ADUNATA_PUBLIC_URL`. `SameSite` alone would not suffice, because a page on a sibling subdomain counts as the same site. A form without a body could also reach a handler that parses none. The middleware covers both without a token. In development the Vite server runs on its own port and forwards `/api` to Adunata together with the browser's `Origin`. The proxy in `client/vite.config.ts` therefore sets `Origin` to `ADUNATA_PUBLIC_URL` on every forwarded request.

The module `access` owns sign-in and sessions. It offers other modules one extractor that yields the signed-in organizer or rejects the request with status 401. A server slice test walks every operation in the OpenAPI document and asserts status 401 without a session. Only an allowlist is exempt, meaning health, catalogs, the sign-in routes and the routes behind an invitation link. A new route that forgets the extractor fails this test.

Limits for sign-in requests:

- Per mail address, Adunata sends at most 3 links per hour. It counts the links stored in the database, so the limit survives a restart.
- Per network address, Adunata accepts at most 20 requests per hour. It counts them in memory, which is enough for one process.

The setting `ADUNATA_TRUSTED_PROXIES` lists the networks of reverse proxies in CIDR notation. The default is `127.0.0.1/32,::1/128`. If a connection comes from such a network, Adunata takes the network address from the rightmost entry of `X-Forwarded-For` that is not itself a trusted proxy. Entries further left are whatever the client sent and count for nothing. Otherwise Adunata uses the address of the connection. The Compose example sets the setting to the network of its proxy. Without it, every request would share the proxy's address and one anonymous client could use up the limit for all organizers.

The operator names the first organizer with three settings, `ADUNATA_FIRST_ORGANIZER_MAIL`, `ADUNATA_FIRST_ORGANIZER_FIRST_NAME` and `ADUNATA_FIRST_ORGANIZER_LAST_NAME`. They are optional, but the mail address needs both names. At start-up Adunata creates that member with the organizer role and the default language of the instance if no organizer exists. Without the settings and without an organizer, Adunata starts and logs a warning, because nobody can sign in. Settings are enough to set up an instance in Docker, in Compose or Swarm and as a systemd service.

For an emergency the binary offers the subcommand `adunata grant-organizer <mail> <first name> <last name>`. It reads the same configuration, connects to the database and makes the member with that address an active organizer. It creates the member if none exists. An operator who has lost every organizer mailbox runs it once, for example with `docker compose exec`. It is not part of the normal setup. The binary parses its command line with the crate `clap` and its derive macros. It offers `--help`, `--version` and help per subcommand. The server runs with the subcommand `serve`, which the image, the systemd unit and the NixOS module call. Without a subcommand the binary prints its help. It exits with status 0 on success, 1 on a failure at runtime such as an invalid configuration or an unreachable database, and 2 on wrong arguments.

Alternatives considered:

- Passwords. Adunata would have to store hashes, offer a reset by mail and defend against guessing. The reset mail is a sign-in by mail link anyway.
- Sign-in through an external identity provider. The proposal puts it out of scope. Most ensembles run none.
- A link that starts the session when it is opened. Mail scanners would use it up or sign in themselves.
- A link bound to the requesting browser. It stops forwarded links but fails whenever the organizer opens the mail on another device.
- Sessions as signed tokens without server state. Revoking a role would not end a session before the token expires.
- A token against cross-site request forgery. The check of `Origin` gives the same protection without state in the client.
- Recovery only by documented SQL or only by the subcommand. SQL needs database skills. The subcommand alone would force every operator onto a console for the first setup.

### 2. Personal links with the secret in the fragment

Record: `0011-personal-links-in-the-url-fragment.md`

Invitation links and sign-in links follow one rule. Each carries a random secret of 256 bits, written as 64 hexadecimal characters. Adunata stores only the SHA-256 hash of the secret. The secret sits in the fragment of the URL, such as `https://adunata.example.org/invitation#3f9a…`. Browsers never send the fragment to the server. The secret therefore stays out of the reverse proxy's access log and out of Adunata's request handling. The client reads the fragment and sends the secret to the API in the header `Authorization: Bearer <secret>`. Adunata never logs headers.

The server generates secrets with the operating system's random generator through the crate `getrandom` and hashes them with the crate `sha2`. Only the hash reaches the database, so the secret never appears in PostgreSQL's statement log. Both crates are in the lock file already, `sha2` through `sqlx` and `rust-embed`, `getrandom` through `ring` and `rand`. The server pins the versions already there, `sha2` 0.11 and `getrandom` 0.4, so the build compiles no new crate.

Every response from Adunata carries these headers, set in one layer:

- `Referrer-Policy: no-referrer` hides the path of a page from every site it links to. Browsers never put the fragment into a referrer anyway.
- `Content-Security-Policy: default-src 'self'; frame-ancestors 'none'` keeps foreign scripts away from the page that holds the secret and stops other sites from framing it.
- `X-Content-Type-Options: nosniff`.
- `Cache-Control: no-store` on every API response, so no cache keeps personal data.

The module `invitations` resolves the secret of an invitation link. It offers the module `responses` an extractor that yields the invitation, its gathering and its member, or rejects the request. An unknown secret and a revoked one both lead to the same page, which names neither the gathering nor the member. Resending an invitation stores the hash of a new secret in place of the old one, so the old link stops working in the same transaction.

Alternatives considered:

- The secret in the path, such as `/invitation/3f9a…`. The reverse proxy would log it with every request. Adunata cannot prevent that.
- The secret in a query parameter. Proxies log the query as well.
- A signed token that carries the invitation's id. Revoking it would need a list of revoked tokens, while a stored hash is revoked by replacing it.
- Hashing in PostgreSQL with its function `sha256`. It saves the crate `sha2`, but the secret would travel to the database and into its statement log.

### 3. Wall-clock times in the time zone of the instance

Record: `0012-wall-clock-times-in-the-instance-time-zone.md`

A gathering's first and last day are dates. A period of presence is two wall-clock times, such as 14 March 2027 18:00. The database stores them as `date` and `timestamp without time zone`. The API exchanges them as ISO 8601 strings without an offset, such as `2027-03-14T18:00`. Client, server and mail show these values as they are and never convert them. The form and the overview therefore show the same times in any browser.

The time zone of the instance matters in two places. Adunata compares with the present, for example whether a gathering has begun or ended. Adunata also shows instants, such as the time of the last change to a response. The module `configuration` reads the required setting `ADUNATA_TIME_ZONE` as an IANA name such as `Europe/Berlin` and rejects unknown names at start-up. No default is safe, because any default shifts for most ensembles when links close and gatherings begin.

One function in `configuration` turns an instant into the date and wall-clock time of the instance. Every comparison with the present goes through it. Instants stay `timestamptz` in the database, such as the end of a session or the time of the last change to a response. Before an instant goes into the API or a mail, the server turns it into wall-clock time with the same function. The client therefore never converts a time.

The crate `jiff` resolves the time zone. It is needed because the crate `time`, which Adunata uses already, has no time zone database. With its feature `tzdb-bundle-always`, `jiff` embeds the database in the binary. The OCI image and the static binary then need no time zone files on the host. Without the feature, `jiff` reads `/usr/share/zoneinfo` on Linux, which a minimal image lacks. `jiff` stays inside the conversion function. Every other type in the server remains a type of `time`, which `sqlx` and `utoipa` already use.

Alternatives considered:

- Instants in UTC, converted for display. Every view and every mail would need the conversion, and a time in the gap of a daylight saving change would have no instant.
- The browser's time zone. A member abroad would see times that differ from the organizer's overview, which the spec rules out.
- Comparing with the present in SQL with `AT TIME ZONE`. PostgreSQL knows the time zones, but Adunata could then check the setting only after connecting to the database, which breaks the rule that configuration fails before any connection.
- A default of `UTC` or `Europe/Berlin`. Either one is wrong for most installations and fails silently.

### 4. Mail loses its content once its delivery ends

Record: `0013-mail-loses-its-content-once-its-delivery-ends.md`

Once the mail server accepts a mail, the module `mail` clears its recipient, subject and both bodies in the same statement that marks it as sent. A mail that fails for good loses its content the same way. The outbox keeps the row with its times and number of attempts for diagnosis. Personal links therefore stay in the database only while their mail waits for delivery.

The outbox gains a nullable column `member_id` that references the member with `ON DELETE CASCADE`. Every mail to a member carries it. Deleting a member therefore deletes the member's mail in the same statement. A match on the address would miss mail sent before the member changed the address. The column `recipient` holds the bare address. The display name moves into the mail's headers at delivery.

The outbox also keeps a nullable `invitation_id`. The overview shows a failed invitation mail at the invitation and offers to resend it. Resending creates a new link and a new mail.

Every value that organizers or members enter, such as titles, locations, descriptions and names, is escaped before it goes into the HTML part of a mail. The function for it moves from the module `spike` to the module `mail`, which every mail passes through.

Alternatives considered:

- Keeping the content of failed mail for diagnosis. The personal link and the member's data would stay in the database and in backups without limit. An operator gains little from reading mail bodies in SQL.
- Keeping content for a fixed number of days. Every invitation link would stay readable in backups for that period.
- Encrypting the outbox. The key would sit next to the database on the same host.

### 5. Modules and data

The module `spike` with its table, form and tests goes away. A migration drops its table.

|Module|Owns|Uses|
|-|-|-|
|`access`|Sign-in links, sessions, the first organizer, the subcommand for recovery, rate limits, the check of `Origin`|`members`, `mail`, `localization`|
|`members`|The member list, the organizer role and the rule that one active organizer remains|nothing|
|`gatherings`|Gatherings, their span and their state|nothing|
|`invitations`|Invitations, personal links, invitation and cancellation mail, resend and revoke, cancelling a gathering|`members`, `gatherings`, `mail`, `localization`|
|`responses`|Responses, the member's response page, the organizer overview, the retention task|`invitations`, `gatherings`|

Cancelling a gathering changes its state and mails every invited member in one transaction. The module `invitations` owns this action, because it knows the recipients. `gatherings` then depends on nothing and no cycle arises.

The organizer role is a column on `members`, because a member holds at most one role and loses it on deletion. Revoking the role, deactivating a member and deleting a member can each remove an active organizer. All three go through one function in `members`. It locks the rows of the active organizers with `FOR UPDATE` and refuses the change if no active organizer would remain. Two organizers who revoke each other at the same moment therefore cannot leave the instance without one.

A response is a row keyed by its invitation. An invitation without a row is pending. The response records whether a member or an organizer changed it last, together with the time.

Foreign keys delete in cascade from member to invitations, responses, sessions, sign-in links and outbox entries, and from gathering to invitations. Whether a stated period lies partly or wholly outside a changed span is computed when the overview is read. The database stores no flag for it.

A background task runs once per hour. It deletes ended sessions and sign-in links that are used or expired. It applies the retention settings `ADUNATA_RETENTION_COMMENT_DAYS` and `ADUNATA_RETENTION_RESPONSE_DAYS`, whose defaults are 30 and 365 days after the last day of a gathering. The value `never` keeps the data. The comment period clears comments. The response period deletes the invitations of the gathering together with their responses, because an invitation without its response would appear as pending. The gathering itself stays. The task belongs to `responses` and asks `invitations` and `access` to delete their expired rows through their public functions.

### 6. User interface

The client builds on the components and libraries of [ADR 0008](../../../docs/decisions/0008-client-libraries.md). A round of prototypes with automated checks and blind grading settled these rules. They go into arc42 section 8 as a new concept.

Tokens and type:

- Components use only the semantic tokens of shadcn/ui, such as `--primary`, `--ring`, `--input` and `--border`, and never a fixed color. The brand color enters through `--primary` and `--primary-foreground` alone. shadcn's `--accent` stays the neutral hover surface.
- Until branding per instance exists, the client ships a neutral slate as primary color, `#2F4A5A` in light mode and `#9CC3D5` in dark mode. Each reaches a contrast above 8:1 against its background. No instance carries the colors of another ensemble.
- The shadcn defaults for `--input` and `--ring` reach only about 1.3:1 and 2.6:1 against white. WCAG 2.2 asks 3:1 for the boundary of a control and for the focus indicator. Both tokens get values that reach at least 3:1 in both color schemes. The focus ring is drawn at full opacity.
- Headings use Literata and text uses Source Sans 3, both from fontsource packages and served by the binary. They replace Geist. Source Sans 3 has plain zeros, so times like 18:00 read as numbers, not as code.
- Sizes are in `rem`, so the whole interface scales with the browser's text size.
- The client follows the color scheme of the operating system. In `index.css` the variant `dark` and the dark token values both move from the class `.dark` to the media query `prefers-color-scheme: dark`.

Controls and pages:

- Single-choice questions such as status and beds are full-width rows of Radix `RadioGroup` with a visible circle. The choice shows through the filled circle, a stronger border and a tint, never through color alone.
- Arrival and departure each follow the date picker with time of shadcn/ui, meaning a `Calendar` in a `Popover` for the day and a native time input for the time. The calendar offers only the days of the gathering. The time input steps in 30 minutes, and the server rejects any other time. Radix `Select` is not used, because it injects a style element when it opens, which the `Content-Security-Policy` blocks. The non-modal `Popover` injects none. The calendar brings `react-day-picker` and `date-fns`, which the shadcn component needs. Its keyboard use and its labels for screen readers get their own client slice test, because a calendar grid is the riskiest control of the form.
- A status shows an icon from `lucide-react` next to its text, which is a check for accepted, a question mark for tentative, a cross for declined and a circle for pending. The icons are `aria-hidden`, so a screen reader reads only the text. Declining is not an error, so it appears in a neutral color.
- The header is a band in the primary color with the product name in `--primary-foreground`. The organizer pages put their navigation into it as a `nav` element. The current page is underlined.

Pages:

- The response page opens with the gathering's title in Literata, its location and description, and a strip with one tile per day of the gathering that shows weekday, date and month. The form follows in this order: status, arrival and departure, beds with their count, comment and a save button across the full width. Arrival and departure each form a `fieldset` and stand side by side, and below each other when space runs out. The confirmation appears under the button. The attendees close the page as chips with their names, where a tentative attendee has a dashed outline and a hidden "tentative" for screen readers, and a last chip such as "and 10 more".
- The overview opens with the gathering's title, its span and location and the action to edit it. A strip of totals follows, which shows accepted, tentative, declined and pending with the icons of the status badges, then beds needed and beds offered. The tile for beds offered adds the balance, such as "3 more than needed". The filter by status is a `ToggleGroup` of buttons in one row.
- The table of the overview lists name, status badge, period, beds and comment, with alternating row backgrounds. Names use the text color, not the primary color, so the brand color does not repeat in every row. The period cell shows a presence bar in front of its text. The bar spans the days of the gathering with a gap between days and fills the stated period in `--muted-foreground`. It is `aria-hidden`, because the text says the same. On a narrow screen the overview turns into one block per member, so no column gets lost.

The prototype that settled these rules is kept outside the repository, because it carries the colors and the logo of one ensemble. It is a reference for layout, proportions and density. This design wins wherever the two differ.
- Every route sets its own page title. After a route change, focus moves to the main heading. After a rejected save, focus moves to the first invalid field. Fields mark errors with `aria-invalid` and link the message through `aria-describedby`. A saved response is announced through `role="status"`. The spike's form uses these patterns already, and the concept in arc42 names them so that new forms copy them.
- Every page has a footer with links to the privacy notice and the imprint, if the operator set `ADUNATA_PRIVACY_URL` and `ADUNATA_IMPRINT_URL`. Invitation mails carry the same links. The response page says that other invited members see the member's name and status. The comment field says that only organizers read the comment.

Each page gets an axe check in its client slice test. Axe in jsdom computes no colors and no layout, so it misses contrast and reflow. The end-to-end tests therefore run axe on the response form and the overview in light and in dark mode. They assert that neither page scrolls horizontally at a width of 390 px with the text enlarged to 200 %. They also check the computed border of a radio row and the focus outline against their background, because axe checks neither.

### 7. Tests that change

The end-to-end test signs in through Mailpit, creates a gathering, invites a member, reads the invitation link from Mailpit and answers through it. The memory check in [`flake.nix`](../../../flake.nix) posts to a fixed route today. It gets its link the same way in a set-up step and then loads the response page and stores responses under load.

### 8. Affected arc42 sections

|Section|Change|
|-|-|
|1 Introduction and goals|Lists the new capabilities in the requirements overview.|
|3 Context and scope|Shows organizers and members as real partners and drops the remark that only an example form exists.|
|4 Solution strategy|Summarizes sign-in by mail link and personal links under the goal data protection and security, with links to records 0010 and 0011.|
|5 Building block view|Replaces `spike` with `access`, `members`, `gatherings`, `invitations` and `responses` in the diagram and the table.|
|6 Runtime view|New file with three scenarios, which are sign-in, inviting members through the outbox and answering through a personal link.|
|7 Deployment view|Shows the trusted proxy in front of Adunata for each way of running it.|
|8 Cross-cutting concepts|Adds authentication and authorization, personal links, security headers, time, user interface and retention of personal data. Adds the new settings to configuration. Under logging, gives the fragment as the reason that personal links never reach a request. Under mail delivery, adds that mail loses its content once delivery ends and that HTML parts escape every entered value. Updates the slice and end-to-end examples that name `spike`.|
|9 Architecture decisions|Lists records 0010 to 0013.|
|10 Quality requirements|Adds scenarios for a guessed link, a personal link absent from logs, an unknown address at sign-in, a request from another origin, an organizer route without a session, expiring comments and the member form at 200 % text on a phone. Moves the mail outage scenario from `spike` to the invitation.|
|11 Risks and technical debt|Updates the risk on date ranges, because gatherings may span the end of a month.|
|12 Glossary|Adds organizer session, sign-in link, period of presence, tentative, retention period and trusted proxy.|

The operating documentation gains every new setting in [`docs/operating/configuration.md`](../../../docs/operating/configuration.md). It also describes the subcommand `grant-organizer` for an emergency, and the Compose example sets `ADUNATA_TRUSTED_PROXIES`.

## Risks / Trade-offs

- [A personal link lives in the outbox until its delivery ends.] → The window is minutes in normal operation and at most the retry period of about a day when the mail server is down.
- [The required time zone adds one setting to the first start.] → Adunata names the missing setting with all other problems in one round, and the example configuration sets it.
- [A forwarded sign-in link signs in whoever opens it first within one hour.] → The link works once and only for active organizers. Binding it to a browser would break the common switch from laptop to phone.
- [The calendar grid is harder to use with a keyboard and a screen reader than plain selects.] → It offers only the days of the gathering, has its own client slice test and runs through axe in the end-to-end test. A gathering has few days, so the grid stays small.
- [The fragment carries the secret, so the response page works only with JavaScript.] → The whole client needs JavaScript already, see [ADR 0003](../../../docs/decisions/0003-single-page-client-with-api.md).
- [Mail lists date ranges through Fluent, which covers only ranges within one month today. A gathering from 30 April to 2 May crosses a month.] → The catalogs gain messages for ranges across months and years, with tests.
- [Retention of responses deletes invitations as well. Organizers then see a past gathering without invitations.] → The spec states this behavior and the operating documentation describes it next to the setting.

## Migration Plan

The instance has no production data yet. Migrations create the new tables, add the member and invitation references to the outbox and drop the table of the spike. Existing installations gain the required setting `ADUNATA_TIME_ZONE` and refuse to start until it is set. The release notes say so. Rolling back means restoring the backup taken before the update, as for every migration.
