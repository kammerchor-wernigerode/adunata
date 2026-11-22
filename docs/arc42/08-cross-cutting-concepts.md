# Cross-cutting concepts

## Configuration

An instance takes its configuration from environment variables and from an optional configuration file in TOML, which the operator names with `--config` or `ADNT_CONFIG_FILE`. The file groups the settings in fixed tables by concern, and each variable derives from table and key. A setting in the environment wins over the same setting in the file. Each secret comes either as a plain value or from a file that a setting names. The file form fits secret handling in NixOS, Docker and systemd alike. One source must not set both forms of a secret.

A development build also reads the files `.env.local` and `.env` in its working directory, so `cargo run` and `cargo test` in `server/` need no setting in the shell. Each file is a source below the configuration file, and `.env.local` wins over `.env`. A release build never reads either file. The repository tracks `server/.env` with the settings for local development.

The module `configuration` reads all settings at start-up and checks them together. It collects every problem before Adunata touches the database or opens a port, so an operator fixes all mistakes in one round. Each message names the setting and never a secret value. A message about the configuration file names the file and the key but never a value or the content of a line. Other modules receive typed settings, such as the public base URL, and never read the environment or the file themselves. The exceptions are `SSL_CERT_FILE` and `SSL_CERT_DIR`, which the TLS library of mail delivery and of the database connection reads to find the trust store, as every TLS tool on the host does. The module `configuration` does not check them. It checks the parameters of the database URL before sqlx parses it, because sqlx logs an unknown parameter with its value. It rejects every variable whose name starts with `PGSSL`, because sqlx would read some of them besides the URL.

The settings and their defaults are listed in the [configuration reference](../operating/configuration.md).

## Localization

Every text that people read comes from a Fluent catalog per language in [`server/locales/`](../../server/locales/). The server formats mail with these catalogs and serves the same files to the client at `/api/catalogs/{language}`, so each text exists once for both. Fluent applies the plural rules of each language.

English is the fallback. When a catalog lacks a text, the English text appears instead of a key. The client loads the English catalog next to the chosen one for this purpose.

The client starts in the language the browser prefers among German and English. A signed-in organizer changes it in the account menu, everyone else in a select in the header. The choice stays on the device. The document language follows the choice, so screen readers pronounce text correctly.

Mail goes out in the language stored for the member who receives it. The default language of the instance applies to the first organizer and to members that `grant-organizer` creates. The client formats dates with the browser's own date formatting. The server spells out date ranges in mail through the catalog, because Fluent in Rust has no date formatting.


## Authentication and authorization

Members never sign in. They reach their invitation through a personal link. Organizers sign in with a single-use link by mail, see [ADR 0010](../decisions/0010-sign-in-by-mail-link.md) and the [runtime view](06-runtime-view.md#sign-in).

The module `access` owns sign-in, sessions and the first organizer. Other modules protect a route by taking the extractor `Organizer` as an argument. It loads the session from the cookie with its member and answers with status 401 unless the member is active and holds the organizer role. Revoking the role or deactivating a member therefore ends every session at the next request. The client sends a visitor without a session to the sign-in page.

`adunata serve --local` starts local mode for debug runs, see [ADR 0023](../decisions/0023-local-mode-on-localhost.md). It refuses to start unless the public URL is on `localhost` and the listen address is a loopback address. If the instance has no active organizer, it creates the first organizer from the bootstrap settings or the default member `organizer@example.org`. Before it accepts requests it logs a sign-in link for the oldest active organizer, without mail. While the mode is on, every requested link also goes to the log and the limit of links per address does not apply.

An integration test in [`server/src/api.rs`](../../server/src/api.rs) calls every operation of the OpenAPI document without a session and expects status 401. Only health, catalogs, the legal links, sign-in and sign-out and the routes behind an invitation link are exempt. A new route that forgets the extractor fails this test.

Sign-in requests are limited to three links per hour per address and 20 requests per hour per network address. Behind a trusted proxy the network address is the rightmost entry of `X-Forwarded-For` that is no trusted proxy.

## Retention of personal data

Personal data expires by default. Once per hour a task of the module `responses` applies two periods that count from the last day of a gathering. After `ADNT_RETENTION_COMMENT_DAYS`, 30 by default, it clears the comments of the gathering's responses. After `ADNT_RETENTION_RESPONSE_DAYS`, 365 by default, it deletes the gathering's invitations, and their responses go along through the foreign key. The gathering itself stays, so organizers see a past gathering without invitations. The value `never` keeps the data. The same task deletes ended sessions and sign-in links that are used or expired.

Each module deletes its own rows through a public function, so `responses` asks `invitations` and `access` to do so. Deleting a member deletes the member's invitations, responses, sessions, sign-in links and waiting mail through foreign keys. Mail loses its content once its delivery ends, see mail delivery below.

## Time

The days of a gathering are dates. Times of arrival and departure are wall-clock times without an offset, such as `2027-03-14T18:00`. The database, the API, the client and mail carry them unchanged, so every view shows the same time. [ADR 0012](../decisions/0012-wall-clock-times-in-the-instance-time-zone.md) records why.

The required setting `ADNT_LOCALIZATION_TIME_ZONE` names the time zone of the instance. One function in the module `configuration` turns an instant into the date and wall-clock time of the instance. Every comparison with the present goes through it, such as whether a gathering has begun. The calendars of the client may use the date of the browser to disable days, but only the server decides whether a day lies in the past. Instants such as the end of a session stay `timestamptz` in the database and pass through the same function before they reach the API or a mail. The binary embeds the time zone database, so hosts and images need no zone files.

## Personal links

Invitation links and sign-in links follow one rule, see [ADR 0011](../decisions/0011-personal-links-in-the-url-fragment.md). Each carries a random secret of 256 bits in the fragment, such as `/invitation#3f9a…`. The module `secret` creates it and its SHA-256 hash, and only the hash reaches the database. The client reads the fragment and sends the secret only in the header `Authorization: Bearer <secret>`.

The module `invitations` resolves the secret of an invitation through the extractor `PersonalLink`, which the routes of `responses` take as an argument. An unknown or revoked secret fails with status 404 and reveals no gathering or member. Resending stores the hash of a new secret in place of the old one.

## Security headers

One layer around the whole router sets these headers on every response, whether it comes from the API or serves the client:

|Header|Value|Effect|
|-|-|-|
|`Referrer-Policy`|`no-referrer`|Sites that a page links to never learn its address.|
|`Content-Security-Policy`|`default-src 'self'; frame-ancestors 'none'`|Pages load scripts, styles, fonts and data from Adunata only and no other site can frame them.|
|`X-Content-Type-Options`|`nosniff`|Browsers trust the declared content type.|

Every API response also carries `Cache-Control: no-store`, so no cache keeps personal data. The client must therefore ship every script and style as a file and set styles only through the DOM, never as an inline element.

A middleware on `/api` rejects every request with a method other than `GET` or `HEAD` whose `Origin` header differs from the origin of `ADNT_HTTP_PUBLIC_URL`. A page on another origin can therefore not change data through the browser of a signed-in organizer. A request without `Origin` does not come from a browser page and passes. In development the Vite server forwards `/api` and sets `Origin` to the public URL.

## User interface

The client builds on the components and libraries of [ADR 0008](../decisions/0008-client-libraries.md).

### Tokens and type

Components use only the semantic tokens of shadcn/ui, such as `--primary`, `--ring`, `--input` and `--border`, and never a fixed color. The brand color enters through `--primary` and `--primary-foreground` alone, so branding per instance can set it later. `--accent` stays the neutral hover surface. Until branding per instance exists, the primary color is a neutral slate, `#2F4A5A` in light and `#9CC3D5` in dark mode.

The boundary of a control and the focus indicator reach at least 3:1 against their background, as WCAG 2.2 asks. The tokens `--input` and `--ring` carry values that meet it in both color schemes, and the focus ring is drawn at full opacity. The client follows the color scheme of the operating system through `prefers-color-scheme`.

Headings use Literata and text uses Source Sans 3. The binary serves both fonts with their licenses. Sizes are in `rem`, so the interface scales with the browser's text size.

### Controls

A single-choice question is a list of full-width rows of a Radix `RadioGroup` with a visible circle. A chosen row shows a filled circle, a stronger border and a tint, never color alone.

A status is a pill that shows an icon from `lucide-react` next to its text. The icon is `aria-hidden`, so a screen reader reads only the text. An acceptance is tinted with the primary color and a tentative acceptance with the secondary surface. Declining is not an error and appears in muted colors. A pending invitation shows only an outline.

### Pages and forms

The frame around every page carries a header, the main area and a footer. The header is a band in the primary color with the product name in Literata. Organizer pages put their navigation into the band and underline the current page. They end the band with an account menu that names the organizer and holds the own member entry, the language and, last and set apart, sign-out. Main headings are set in Literata at about 1.6 rem with a tight line height, so a long title still fits a phone. The footer links the privacy notice and the imprint when the operator set them.

Every route sets its own page title through the main heading. After a route change, focus moves to the main heading. A page action such as creating or editing sits at the top right on the line of the main heading and wraps below it on a narrow screen, while form actions end the form. Forms follow these patterns:

- A field with an error carries `aria-invalid` and links its message through `aria-describedby`.
- After a rejected save, focus moves to the first invalid field.
- A successful save is announced through an element with `role="status"`.

Every page runs axe in its client integration test. Axe in jsdom computes neither colors nor layout, so end-to-end tests check contrast of control boundaries and focus indicators and the layout at narrow widths.

## Persistence and migrations

PostgreSQL holds all data, see [ADR 0004](../decisions/0004-postgresql-as-database.md). Queries are plain SQL that the server sends at runtime. Builds therefore need no database.

The connection to PostgreSQL takes its TLS settings from the database URL, see [ADR 0020](../decisions/0020-tls-to-the-database-configured-in-its-url.md). Only `verify-ca` and `verify-full` check the certificate of the database, against the trust store of the host as for the [mail server](#mail-delivery). The [configuration reference](../operating/configuration.md#settings) describes the modes.

The binary embeds its migrations. On start, the module `database` compares the newest migration applied to the database with the newest one the binary knows. If the database is ahead, a newer release has already migrated it. Adunata then refuses to start without changing anything and exits with a status that the [operating guide](../operating/installation.md#commands) lists. Otherwise it applies the pending migrations before it accepts requests. Migrations only move forward, and a released migration never changes. Going back to an older release means restoring a backup.

Waiting for a database connection is capped at five seconds, so a lost database shows up quickly in the health status.

## Logging

Adunata writes one JSON object per line to standard output and leaves collecting and storing logs to the platform, such as the journal or a container runtime.

Logs carry no personal data and no secrets. In particular they never contain:

- secrets or values of the configuration, which therefore never go to a log as a whole
- invitation links or other links that grant access
- recipients, subjects or bodies of mail
- the text of a mail server's reply, which often repeats the recipient

An event instead names what happened and the identifiers an operator needs to look further, such as the id of an outbox entry. Requests are not logged. Personal links never reach a request line anyway, because their secret sits in the fragment of the URL, see personal links below.

Local mode on `localhost` is the one exception. It logs sign-in links on purpose, see [authentication and authorization](#authentication-and-authorization).

## Mail delivery

Every mail goes through the outbox, see [ADR 0006](../decisions/0006-outbox-for-mail.md). A module that causes a mail adds it to the outbox within its own transaction and never talks to the mail server itself. Each mail has a plain text part and an HTML part with the same content and the same links, so it reads completely in any mail client.

The connection to the mail server is encrypted with STARTTLS unless the operator chooses implicit TLS or, explicitly, no encryption. Adunata checks the server certificate against the certificate authorities in the trust store of the host, see [ADR 0016](../decisions/0016-trust-store-of-the-host-for-mail-server-certificates.md). The [configuration reference](../operating/configuration.md#certificate-authorities) names the store each way of running Adunata uses.

A failed delivery is logged with the outbox entry and the kind of failure, as described under logging. Mail that failed for good stays in the outbox with its failure time.

A mail loses its recipient, subject and both bodies in the statement that marks it as sent or as failed for good, see [ADR 0013](../decisions/0013-mail-loses-its-content-once-its-delivery-ends.md). Personal links therefore stay in the database only while their mail waits. The outbox stores the bare address and references the member and, for an invitation mail, the invitation. Delivery adds the member's name to the address.

Every value that organizers or members entered, such as titles, locations and names, passes through the function `escape_html` of the module `mail` before it goes into an HTML part.

## Health

The endpoint `/api/health` reports whether the instance can serve requests and, if not, which dependency is unavailable. It carries no personal data and no configuration values, so monitors may call it without authentication.

The command `adunata healthcheck` asks the same endpoint from inside a container, because the image has no shell and no HTTP client, see [ADR 0022](../decisions/0022-healthcheck-as-a-command-of-the-binary.md). The [operating guide](../operating/installation.md#check-readiness) describes the command and how an operator wires it.

## Shutdown

On SIGTERM, Adunata stops accepting connections, lets requests in progress finish and exits with status zero. Mail that the background task has not yet sent stays in the outbox and goes out after the next start.

## Testing

Tests run at several levels. Each level checks what the level below cannot see. Locally and in CI, every check except the end-to-end tests and the scan for advisories runs as a check of the flake, so both use the same entry point.

The flake groups its checks into targets that build on each other like the phases of a build lifecycle. Each target contains the one before it, so `nix build .#checks.<system>.integration` runs lint, build, unit and integration tests together.

The workflow `CI` runs its jobs for `x86_64-linux` and `aarch64-linux`, each on a native runner. The job `build` builds the target `integration`, the release binary and the image of its architecture in one call and in one Nix store. It uploads the binary in a release archive and the image as artifacts of the run. On `x86_64` the same call runs the check `nixosVmTest`, so the VM test checks the bytes that the job uploads. The arm64 runner has no KVM, so the NixOS module and the example unit stay untested on `aarch64`. The job `e2e` runs the end-to-end tests on both architectures against the uploaded archive and then starts the uploaded image against PostgreSQL until `adunata healthcheck` succeeds inside the container. On `aarch64` this start is the only test of the image. A draft pull request builds only `x86_64` without image and VM test and skips `e2e`. The job `changes` runs first. If a pull request or a push changes only files that no check but `editorconfig` reads, it checks only `editorconfig` and every other job skips. The job lists these paths: the OpenSpec artifacts, the software factory, the files of coding agents and the documentation apart from the README, whose version pins a check reads. The job `Verdict` needs every other job and fails if one of them failed or was cancelled. It is the only check that the ruleset of `main` requires, so the matrix can change without touching the ruleset. A draft run reports it as `Verdict (draft)`, so its result cannot stand in for the full run of the same commit. The workflow `client-dependencies` writes the hash of the client dependencies to `client/pnpm-deps.hash` on pull requests of Dependabot, so the flake can fetch them. It pushes with the token of a GitHub App, so the push starts CI on the pull request.

The workflow `advisories` checks `server/Cargo.lock` with `cargo deny check advisories` and `client/pnpm-lock.yaml` with `pnpm audit` against the advisory databases. It needs network access, so it runs outside the flake with the tools of its nixpkgs. It runs daily and on pull requests that change a lock file or its configuration. It fails on an advisory of a crate and on an advisory of a client package of severity high or critical that `client/audit-exceptions.json` does not except until a date.

|Target|Contains|Builds on|
|-|-|-|
|`lint`|Formatting of Rust, Nix and every file per `.editorconfig`, Clippy, oxlint, the licenses of crates and installed client packages against the approved list in `server/deny.toml` and the exceptions for client packages in `client/license-exceptions.json`, the licenses of the tools from nixpkgs and of the packages in the OCI image against that list and a few licenses allowed only for them, crates only from crates.io and with a version requirement, zizmor over the GitHub workflows and the Dependabot configuration, the tests of the version comparison of the release workflow, whether `server/Cargo.toml`, `server/Cargo.lock` and `client/package.json` name the same version without build metadata, whether `server/about.toml` accepts the same licenses for the notices, whether the notices of the Rust standard library come from the version that builds the server and whether the versions of Rust, pnpm, `editorconfig-checker`, `cargo-deny` and `cargo-about` pinned for work without Nix match nixpkgs|nothing|
|`build`|The server with the embedded client and its license notices and, on Linux, whether the packages from nixpkgs in the OCI image match the list of packages whose notices the image carries|`lint`|
|`test`|Unit tests of server and client|`build`|
|`integration`|Integration tests of server and client, startup tests of the binary and the contract|`test`|
|`nixos`|The NixOS VM test, Linux only|`integration`|

A unit test has no input or output. It uses no database, no mail server, no socket, no file and no router. Every other test is an integration test. On the server it lives in a module named `integration`, on the client in a file named `*.integration.test.ts` or `*.integration.test.tsx`. The target `test` skips both and starts neither PostgreSQL nor Mailpit, so a test that needs them outside such a module fails there. Nothing enforces the rule for sockets, files and the router.

Each check inside a target is also a check of its own, such as `clippy` or `serverIntegrationTests`, so a failing one runs again alone.

On macOS with a Linux builder, such as the `linux-builder` of nix-darwin, `nix build .#vmChecks.nixos` runs the target `nixos`. The builder builds its Linux artifacts for the same architecture. QEMU starts the virtual machines on the Mac with HVF. `nix flake check` leaves it out, so it needs no Linux builder.

|Level|What it checks|Example|
|-|-|-|
|Unit|Logic without any input or output, such as formatting a message.|`plural_forms_in_german` in [`server/src/localization.rs`](../../server/src/localization.rs)|
|Server integration|An API handler called in process with real routing against a real PostgreSQL. Each test gets its own empty database with all migrations applied and drops it afterwards. Mail goes to a real Mailpit.|`duplicate_address` in [`server/src/members.rs`](../../server/src/members.rs)|
|Client integration|A component against the API mocks that Orval generates, driven like a user would. Accessibility checks with axe run on rendered components.|[`client/src/routes/_organizer/members/members.integration.test.tsx`](../../client/src/routes/_organizer/members/members.integration.test.tsx)|
|Contract|The committed OpenAPI document matches the server. The generated client code matches the document.|`the_client_holds_the_current_document` in [`server/src/api.rs`](../../server/src/api.rs) and `pnpm run check:generated` in `client/`|
|End to end|A few complete flows in a browser against the real binary, PostgreSQL and Mailpit, with accessibility checks by axe in light and dark mode, contrast of controls and reflow at 390 px with 200 % text.|[`client/e2e/invitation.e2e.ts`](../../client/e2e/invitation.e2e.ts)|
|NixOS VM|Each supported installation, meaning the NixOS module, the OCI image and the release binary with its example unit, starts in a virtual machine, migrates the database, reports healthy and serves the client. Runs on Linux with KVM.|[`nix/test.nix`](../../nix/test.nix)|

The end-to-end tests run outside the flake with `pnpm run e2e` in `client/`, on a laptop and in the CI job `e2e` on Ubuntu without Nix. A global setup of Playwright starts PostgreSQL and Mailpit from [`compose.yaml`](../../compose.yaml) as a fresh Compose project of the checkout on free ports. It then starts the server with only the settings of the tests, from a release build of the checkout or, in CI, from the release archive of the job `build`. After the tests it removes the project. This has consequences. The tests need Docker with Compose. The browsers come from Playwright and need nix-ld on NixOS. Node comes from the runner, and the versions of PostgreSQL and Mailpit come from `compose.yaml` instead of nixpkgs. A green `nix flake check` does not cover the end-to-end tests, and a release relies on their run on `main`. The tests take colors from computed styles and measure reflow only after the fonts of the client have loaded, so they give the same result on Linux and macOS.
