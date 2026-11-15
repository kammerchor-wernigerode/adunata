# Cross-cutting concepts

## Configuration

An instance takes its configuration from environment variables only. There is no configuration file. Secrets come from files that a variable names, which fits secret handling in NixOS, Docker and systemd alike.

The module `configuration` reads all settings at start-up and checks them together. It collects every problem before Adunata touches the database or opens a port, so an operator fixes all mistakes in one round. Each message names the setting and never a secret value. Other modules receive typed settings, such as the public base URL, and never read the environment themselves.

The settings and their defaults are listed in the [configuration reference](../operating/configuration.md).

## Localization

Every text that people read comes from a Fluent catalog per language in [`server/locales/`](../../server/locales/). The server formats mail with these catalogs and serves the same files to the client at `/api/catalogs/{language}`, so each text exists once for both. Fluent applies the plural rules of each language.

English is the fallback. When a catalog lacks a text, the English text appears instead of a key. The client loads the English catalog next to the chosen one for this purpose.

The client starts in the language the browser prefers among German and English. A switch next to the title changes it. The choice stays on the device. The document language follows the choice, so screen readers pronounce text correctly.

Mail goes out in the language stored for the recipient. Without one it uses the default language of the instance. The client formats dates with the browser's own date formatting. The server spells out date ranges in mail through the catalog, because Fluent in Rust has no date formatting.


## Time

The days of a gathering are dates. Times of arrival and departure are wall-clock times without an offset, such as `2027-03-14T18:00`. The database, the API, the client and mail carry them unchanged, so every view shows the same time. [ADR 0012](../decisions/0012-wall-clock-times-in-the-instance-time-zone.md) records why.

The required setting `ADUNATA_TIME_ZONE` names the time zone of the instance. One function in the module `configuration` turns an instant into the date and wall-clock time of the instance. Every comparison with the present goes through it, such as whether a gathering has begun. Instants such as the end of a session stay `timestamptz` in the database and pass through the same function before they reach the API or a mail. The binary embeds the time zone database, so hosts and images need no zone files.

## Security headers

One layer around the whole router sets these headers on every response, whether it comes from the API or serves the client:

|Header|Value|Effect|
|-|-|-|
|`Referrer-Policy`|`no-referrer`|Sites that a page links to never learn its address.|
|`Content-Security-Policy`|`default-src 'self'; frame-ancestors 'none'`|Pages load scripts, styles, fonts and data from Adunata only and no other site can frame them.|
|`X-Content-Type-Options`|`nosniff`|Browsers trust the declared content type.|

Every API response also carries `Cache-Control: no-store`, so no cache keeps personal data. The client must therefore ship every script and style as a file and set styles only through the DOM, never as an inline element.

A middleware on `/api` rejects every request with a method other than `GET` or `HEAD` whose `Origin` header differs from the origin of `ADUNATA_PUBLIC_URL`. A page on another origin can therefore not change data through the browser of a signed-in organizer. A request without `Origin` does not come from a browser page and passes. In development the Vite server forwards `/api` and sets `Origin` to the public URL.

## User interface

The client builds on the components and libraries of [ADR 0008](../decisions/0008-client-libraries.md).

### Tokens and type

Components use only the semantic tokens of shadcn/ui, such as `--primary`, `--ring`, `--input` and `--border`, and never a fixed color. The brand color enters through `--primary` and `--primary-foreground` alone, so branding per instance can set it later. `--accent` stays the neutral hover surface. Until branding per instance exists, the primary color is a neutral slate, `#2F4A5A` in light and `#9CC3D5` in dark mode.

The boundary of a control and the focus indicator reach at least 3:1 against their background, as WCAG 2.2 asks. The tokens `--input` and `--ring` carry values that meet it in both color schemes, and the focus ring is drawn at full opacity. The client follows the color scheme of the operating system through `prefers-color-scheme`.

Headings use Literata and text uses Source Sans 3. The binary serves both fonts with their licenses. Sizes are in `rem`, so the interface scales with the browser's text size.

### Controls

A single-choice question is a list of full-width rows of a Radix `RadioGroup` with a visible circle. A chosen row shows a filled circle, a stronger border and a tint, never color alone.

A status shows an icon from `lucide-react` next to its text. The icon is `aria-hidden`, so a screen reader reads only the text. Declining is not an error and appears in a neutral color.

### Pages and forms

The frame around every page carries a header with the product name on the primary color, the main area and a footer. The footer links the privacy notice and the imprint when the operator set them.

Every route sets its own page title through the main heading. After a route change, focus moves to the main heading. Forms follow these patterns:

- A field with an error carries `aria-invalid` and links its message through `aria-describedby`.
- After a rejected save, focus moves to the first invalid field.
- A successful save is announced through an element with `role="status"`.

Every page runs axe in its client slice test. Axe in jsdom computes neither colors nor layout, so end-to-end tests check contrast of control boundaries and focus indicators and the layout at narrow widths.

## Persistence and migrations

PostgreSQL holds all data, see [ADR 0004](../decisions/0004-postgresql-as-database.md). Queries are plain SQL that the server sends at runtime. Builds therefore need no database.

The binary embeds its migrations. On start, the module `database` compares the newest migration applied to the database with the newest one the binary knows. If the database is ahead, a newer release has already migrated it. Adunata then refuses to start without changing anything. Otherwise it applies the pending migrations before it accepts requests. Migrations only move forward. Going back to an older release means restoring a backup.

Waiting for a database connection is capped at five seconds, so a lost database shows up quickly in the health status.

## Logging

Adunata writes one JSON object per line to standard output and leaves collecting and storing logs to the platform, such as the journal or a container runtime.

Logs carry no personal data and no secrets. In particular they never contain:

- secrets or values of the configuration, which therefore never go to a log as a whole
- invitation links or other links that grant access
- recipients, subjects or bodies of mail
- the text of a mail server's reply, which often repeats the recipient

An event instead names what happened and the identifiers an operator needs to look further, such as the id of an outbox entry. Requests are not logged, because their paths will carry personal links.

## Mail delivery

Every mail goes through the outbox, see [ADR 0006](../decisions/0006-outbox-for-mail.md). A module that causes a mail adds it to the outbox within its own transaction and never talks to the mail server itself. Each mail has a plain text part and an HTML part with the same content and the same links, so it reads completely in any mail client.

The connection to the mail server is encrypted with STARTTLS unless the operator chooses implicit TLS or, explicitly, no encryption. Adunata checks the server certificate against the Mozilla root certificates.

A failed delivery is logged with the outbox entry and the kind of failure, as described under logging. Mail that failed for good stays in the outbox with its failure time.

A mail loses its recipient, subject and both bodies in the statement that marks it as sent or as failed for good, see [ADR 0013](../decisions/0013-mail-loses-its-content-once-its-delivery-ends.md). Personal links therefore stay in the database only while their mail waits. The outbox stores the bare address and references the member and, for an invitation mail, the invitation. Delivery adds the member's name to the address.

Every value that organizers or members entered, such as titles, locations and names, passes through the function `escape_html` of the module `mail` before it goes into an HTML part.

## Health

The endpoint `/api/health` reports whether the instance can serve requests and, if not, which dependency is unavailable. It carries no personal data and no configuration values, so monitors may call it without authentication.

## Shutdown

On SIGTERM, Adunata stops accepting connections, lets requests in progress finish and exits with status zero. Mail that the background task has not yet sent stays in the outbox and goes out after the next start.

## Testing

Tests run at several levels. Each level checks what the level below cannot see. Every check runs through `nix flake check`, locally and in CI, so both use the same entry point.

|Level|What it checks|Example|
|-|-|-|
|Unit|Logic without input or output, such as formatting a message.|`plural_forms_in_german` in [`server/src/localization.rs`](../../server/src/localization.rs)|
|Server slice|An API handler called in process with real routing against a real PostgreSQL. Each test gets its own empty database with all migrations applied and drops it afterwards. Mail goes to a real Mailpit.|`rejects_a_taken_name_at_the_name_field` in [`server/src/spike.rs`](../../server/src/spike.rs)|
|Client slice|A component against the API mocks that Orval generates, driven like a user would. Accessibility checks with axe run on rendered components.|[`client/src/spike/SpikeForm.test.tsx`](../../client/src/spike/SpikeForm.test.tsx) and [`client/src/spike/SpikeForm.a11y.test.tsx`](../../client/src/spike/SpikeForm.a11y.test.tsx)|
|Contract|The committed OpenAPI document matches the server. The generated client code matches the document.|The check `contract` in [`flake.nix`](../../flake.nix)|
|End to end|A few complete flows in a browser against the real binary, PostgreSQL and Mailpit, with accessibility checks by axe. Runs on Linux.|[`client/e2e/language.e2e.ts`](../../client/e2e/language.e2e.ts)|
|NixOS VM|Each supported installation, meaning the NixOS module, the OCI image and the release binary with its example unit, starts in a virtual machine, migrates the database, reports healthy and serves the client. Runs on Linux with KVM.|[`nix/test.nix`](../../nix/test.nix)|

The check `memory` in [`flake.nix`](../../flake.nix) measures peak memory of the release build under load on Linux and fails above 150 MB.
