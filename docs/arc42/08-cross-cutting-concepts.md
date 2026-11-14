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

## Health

The endpoint `/api/health` reports whether the instance can serve requests and, if not, which dependency is unavailable. It carries no personal data and no configuration values, so monitors may call it without authentication.

## Shutdown

On SIGTERM, Adunata stops accepting connections, lets requests in progress finish and exits with status zero. Mail that the background task has not yet sent stays in the outbox and goes out after the next start.

## Testing

Tests run at several levels. Each level checks what the level below cannot see. Every check runs through `nix flake check`, locally and in CI, so both use the same entry point.

|Level|What it checks|Example|
|-|-|-|
|Unit|Logic without input or output, such as formatting a message.|`plural_forms_in_german` in [`server/src/localization.rs`](../../server/src/localization.rs)|
|Server slice|An API handler called in process with real routing against a real PostgreSQL. Each test gets its own empty database with all migrations applied. Mail goes to a real Mailpit.|`rejects_a_taken_name_at_the_name_field` in [`server/src/spike.rs`](../../server/src/spike.rs)|
|Client slice|A component against the API mocks that Orval generates, driven like a user would. Accessibility checks with axe run on rendered components.|[`client/src/spike/SpikeForm.test.tsx`](../../client/src/spike/SpikeForm.test.tsx) and [`client/src/spike/SpikeForm.a11y.test.tsx`](../../client/src/spike/SpikeForm.a11y.test.tsx)|
|Contract|The committed OpenAPI document matches the server. The generated client code matches the document.|The check `contract` in [`flake.nix`](../../flake.nix)|
|End to end|A few complete flows in a browser against the real binary, PostgreSQL and Mailpit, with accessibility checks by axe. Runs on Linux.|[`client/e2e/language.e2e.ts`](../../client/e2e/language.e2e.ts)|
|NixOS VM|The NixOS module starts, migrates the database and reports healthy.|Planned|

The check `memory` in [`flake.nix`](../../flake.nix) measures peak memory of the release build under load on Linux and fails above 150 MB.
