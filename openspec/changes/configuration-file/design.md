# Design

## Context

See proposal.md for the motivation.

`Configuration::load` takes one lookup from a setting name to an optional value and does all parsing and validation through it. `from_environment` passes `std::env::var` and treats an empty value as unset. Each secret has only a `_FILE` setting, which `secret()` reads. A password from that file wins over a password in `ADUNATA_DATABASE_URL`. `main.rs` builds the configuration for `serve` and `grant-organizer` and logs each problem through `tracing::error!`. The command line has subcommands and no global option.

The NixOS module, the OCI image and the release archive set environment variables. Each setting is listed in `docs/operating/configuration.md`.

## Goals / Non-Goals

**Goals:**

- Every setting can come from the environment or from one TOML file, with the environment winning.
- Every secret can be a plain value or a file. The environment wins over the file. One source never sets both forms of a secret.
- A mistake in the file stops the start with a message that names the file and the key, never a value.

**Non-Goals:**

- A default path for the file. Adunata never reads a file the operator did not name.
- A `settings` option in the NixOS module that renders the file. The module keeps setting variables. An option can follow when someone needs it.
- Any migration or detection of old variable names or module options.
- Reloading the file at run time.
- Checking the permissions of the file.

## Decisions

### TOML as the format

Architecturally significant, because it adds an interface for operators and a dependency, and section 08 states that there is no configuration file.

TOML has explicit types, comments and no meaning in indentation. It is the format of Cargo and most Rust tools. The crate `toml` under MIT or Apache-2.0 parses it into a table without any schema of its own. NixOS writes TOML with `pkgs.formats.toml`, so a later module option costs little.

Alternatives considered:

- YAML. Its implicit typing turns values such as `no` or `08` into booleans or numbers, and the common Serde crate for it is no longer maintained.
- JSON. It has no comments, which an operator needs to explain a setting in place.
- dotenv as a file. systemd and Compose already read it as environment, so it adds nothing over today.

### Fixed tables by concern

Not architecturally significant on its own, because it shapes the file the decision above introduces.

Each table matches a concern of Adunata, mostly a cross-cutting concept of section 08. Each setting has exactly one key, in kebab-case as TOML commonly writes keys:

|Table|Key|Variable|Replaces|
|-|-|-|-|
|`http`|`public-url`|`ADNT_HTTP_PUBLIC_URL`|`ADUNATA_PUBLIC_URL`|
|`http`|`listen-address`|`ADNT_HTTP_LISTEN_ADDRESS`|`ADUNATA_LISTEN_ADDRESS`|
|`http`|`trusted-proxies`|`ADNT_HTTP_TRUSTED_PROXIES`|`ADUNATA_TRUSTED_PROXIES`|
|`database`|`url`|`ADNT_DATABASE_URL`|`ADUNATA_DATABASE_URL`|
|`database`|`password`|`ADNT_DATABASE_PASSWORD`|new|
|`database`|`password-file`|`ADNT_DATABASE_PASSWORD_FILE`|`ADUNATA_DATABASE_PASSWORD_FILE`|
|`smtp`|`host`|`ADNT_SMTP_HOST`|`ADUNATA_SMTP_HOST`|
|`smtp`|`port`|`ADNT_SMTP_PORT`|`ADUNATA_SMTP_PORT`|
|`smtp`|`encryption`|`ADNT_SMTP_ENCRYPTION`|`ADUNATA_SMTP_ENCRYPTION`|
|`smtp`|`username`|`ADNT_SMTP_USERNAME`|`ADUNATA_SMTP_USERNAME`|
|`smtp`|`password`|`ADNT_SMTP_PASSWORD`|new|
|`smtp`|`password-file`|`ADNT_SMTP_PASSWORD_FILE`|`ADUNATA_SMTP_PASSWORD_FILE`|
|`mail`|`sender`|`ADNT_MAIL_SENDER`|`ADUNATA_MAIL_SENDER`|
|`mail`|`max-attempts`|`ADNT_MAIL_MAX_ATTEMPTS`|`ADUNATA_MAIL_MAX_ATTEMPTS`|
|`localization`|`default-language`|`ADNT_LOCALIZATION_DEFAULT_LANGUAGE`|`ADUNATA_DEFAULT_LANGUAGE`|
|`localization`|`time-zone`|`ADNT_LOCALIZATION_TIME_ZONE`|`ADUNATA_TIME_ZONE`|
|`retention`|`comment-days`|`ADNT_RETENTION_COMMENT_DAYS`|`ADUNATA_RETENTION_COMMENT_DAYS`|
|`retention`|`response-days`|`ADNT_RETENTION_RESPONSE_DAYS`|`ADUNATA_RETENTION_RESPONSE_DAYS`|
|`legal`|`privacy-url`|`ADNT_LEGAL_PRIVACY_URL`|`ADUNATA_PRIVACY_URL`|
|`legal`|`imprint-url`|`ADNT_LEGAL_IMPRINT_URL`|`ADUNATA_IMPRINT_URL`|
|`bootstrap`|`organizer-mail`|`ADNT_BOOTSTRAP_ORGANIZER_MAIL`|`ADUNATA_FIRST_ORGANIZER_MAIL`|
|`bootstrap`|`organizer-first-name`|`ADNT_BOOTSTRAP_ORGANIZER_FIRST_NAME`|`ADUNATA_FIRST_ORGANIZER_FIRST_NAME`|
|`bootstrap`|`organizer-last-name`|`ADNT_BOOTSTRAP_ORGANIZER_LAST_NAME`|`ADUNATA_FIRST_ORGANIZER_LAST_NAME`|

The name of a variable follows from table and key, after the relaxed binding of Spring Boot: `ADNT_`, the table, `_` and the key, upper-cased, with each `-` turned into `_`. Spring drops the dashes instead, which would give names such as `ADNT_HTTP_PUBLICURL`. The prefix `ADNT` is shorter than the name of the project and still unique among the variables a host usually sets. The file accepts only the canonical key, so `public_url` or `publicUrl` in `[http]` is an unknown key. The relaxed part is only the translation into variables.

Adunata has no stable release, so every variable takes its new name without migration and without any handling of the old one. An old name that is still set is ignored like any other unknown variable. `ADNT_CONFIG_FILE` names the file and belongs to no table. Variables that only development tooling and tests read, such as `ADUNATA_VITE_PORT` or `ADUNATA_E2E_URL`, are no settings of the server and keep their names. Tooling that reads a setting of the server, such as the Vite configuration for the address of its proxy, follows the new name.

The options of the NixOS module follow the same tables in camel case, such as `services.adunata.http.publicUrl`, `services.adunata.localization.timeZone` and `services.adunata.bootstrap.organizerMail`. The module renames them without `mkRenamedOptionModule`. `database.createLocally` stays, because it configures the host and not Adunata.

The table lives in code as one list of pairs of table and key next to `load`. Every variable name derives from it. The parsers still name their setting as a literal in each problem, because the project writes no macros of its own. A unit test therefore keeps both in step, as described below. The operating documentation shows the same table. `[smtp]` holds only the transport, and `[mail]` holds the delivery through the outbox. `[bootstrap]` holds what Adunata needs only on its first start. `[localization]` is the umbrella for language and region of the instance, so it holds the time zone too.

A string stays as it is. An integer becomes its decimal text. A password and a username take only a string, because TOML reads a bare `0x1F` or `1_000` as a number whose decimal text differs from what the operator wrote. A non-empty array of strings becomes the strings joined by commas, which only `trusted-proxies` takes. Any other type, and an array for any other key, is a problem that names the key. An empty string counts as unset, as it does in the environment. A key or table that is not in the list is a problem. TOML itself rejects a key that appears twice.

A unit test runs `load` with a lookup that answers every name with a non-empty value and records each name it asks for. It asserts that the recorded names equal the derived names and that the derived names are unique. A pair without a parser or a parser without a pair then fails the test. Without non-empty answers, `load` would stop before it asks for the names of the first organizer, so the test would miss them.

Alternatives considered:

- Keys derived from the old variable names by joining tables with `_`. One setting could then be written in several ways, such as `[first_organizer] mail` and `first_organizer_mail`, so the documentation could not show one file, and a second rule had to reject duplicates.
- Keeping the old variable names or the old module options next to the tables. Two naming schemes would need a mapping in code and in the documentation.
- Relaxed spellings in the file, as Spring accepts them. Documentation and support would show several forms again.
- Flat keys only. One spelling, but a long file without grouping.
- A Serde struct that mirrors the settings. It needs a second description of every setting and its own error messages, which drift from the parsers that already exist.

### Ordered sources

Not architecturally significant, because it changes only how the module `configuration` combines its inputs.

`Configuration::load` takes an ordered list of sources instead of one lookup. Each source answers a setting name with an optional value. An ordinary setting takes the value of the first source that has one. The parsers of the single settings stay as they are, and `secret()` walks the sources itself:

1. It takes the first source that sets the plain setting or the `_FILE` setting of a secret.
2. If that source sets both, it reports a problem that names both settings.
3. If that source sets the file, Adunata reads it. An empty file means no secret and does not fall back to the next source.
4. Otherwise it takes the plain value.

So `ADNT_DATABASE_PASSWORD` in the environment wins over a `password-file` in the file. A password from either form keeps winning over a password in `ADNT_DATABASE_URL`, whatever source each comes from. The URL password is part of the connection and not a form of the secret. A relative path in a file setting of a secret resolves against the working directory, wherever it is set.

A single merged lookup cannot tell which source a value came from, so it could not check both forms in one source.

Alternatives considered for both forms in one source:

- The file wins. A rotated plain password would then lose against a forgotten file without any hint.
- The file wins with a warning. Few operators read warnings of a service that starts.

### Problems never show values

Not architecturally significant, because section 08 already forbids values of the configuration in logs.

A file that cannot be read or parsed is the only problem Adunata reports, because every setting from it would otherwise show up as missing. Adunata builds the message from the file name, the line, the column and the message of the parser. It never uses the `Display` of the parser error, which prints the line of the file, including a password. Problems with the type of a key name the key and never the value. A value of the right type that a parser rejects, such as `port = 99999`, is reported under the name of its variable, as today. The documentation says so. They join the problems of the other settings, so an operator sees them all at once.

### Naming the file

Not architecturally significant, because it is a detail of the command line.

`--config <path>` is a global option of the command line, so `serve` and `grant-organizer` read the same file. The feature `env` of `clap` lets the option fall back to `ADNT_CONFIG_FILE`, so containers and systemd units can name the file without changing the command. The option takes a string, and an empty value counts as unset, as for every other variable. A path type would turn `ADNT_CONFIG_FILE=` into a usage error.

### Interplay with the settings files for development

The change `service-independent-worktrees` adds `server/.env.local` and `server/.env` for debug builds. Both changes add sources to the same list. The order is environment, then the named configuration file, then `.env.local`, then `.env`. A file the developer names explicitly wins over files the debug build loads on its own.

That change rewrites `load` to wrap one lookup and words its spec as "a release build reads only the environment". This change lands with a task that rebases it onto main. If `service-independent-worktrees` has merged by then, the task turns its dotenv files into two more sources and rewords the sentence, so that a release build reads no development file but does read a named configuration file. If it has not merged, that change does the same when it rebases, including the new prefix in its `server/.env`, its plugin and its client configuration.

### Architecture documentation and decisions

Each update lands in the commit that changes the behavior it describes.

- `docs/arc42/03-context-and-scope.md`: the operator row names the configuration file next to environment variables and secret files.
- `docs/arc42/05-building-block-view.md`: the module `configuration` reads the environment and an optional file.
- `docs/arc42/07-deployment-view.md`: the container row names the file as an option.
- `docs/arc42/08-cross-cutting-concepts.md`, section Configuration: the sentence "There is no configuration file" goes. It describes the file, the precedence and plain secrets.
- `docs/arc42/09-architecture-decisions.md`: a row for the new record.

Draft of ADR 0014:

> **ADR 0014: Optional configuration file in TOML**
>
> *Context.* Adunata takes its configuration from environment variables and reads each secret from a file a variable names. This fits NixOS, containers and systemd. An operator who wants to keep the configuration in one file, or who has no secret store, has to put every setting into a unit file, a Compose file or a script. Formats for configuration differ in how much they guess about types and whether they allow comments.
>
> *Decision.* We will read an optional configuration file in TOML that the operator names with `--config` or `ADNT_CONFIG_FILE`. The file groups the settings in fixed tables by concern, with one key in kebab-case per setting. Each environment variable derives from table and key, after the relaxed binding of Spring Boot, so both forms share one scheme. The environment wins over the file. Every secret can also be a plain value, and one source must not set both forms of a secret. We considered YAML, which guesses types, JSON, which has no comments, and dotenv, which systemd and Compose already read as environment.
>
> *Status.* Accepted
>
> *Consequences.* Operators choose between a file, the environment and a secret store, or combine them. Every variable changed its name before the first stable release. The record lists each old name next to its new one, from the columns "Replaces" and "Variable" of the table above, so readers of older records such as ADR 0012 find the setting. Adunata gains the dependency `toml`. A plain secret in the environment is visible to processes of the same user, to `docker inspect` and, in a systemd unit, to every local user through `systemctl show`. The documentation names these cases. Every new setting needs a key in a table, or the file cannot set it.

## Risks / Trade-offs

- [An operator puts a password or the first organizer's name and address in a world-readable file or in version control] → The documentation recommends mode `0600`, `password-file` keys and no `[bootstrap]` table in a file under version control. Adunata does not check either.
- [A plain secret in `Environment=` of a systemd unit is readable through `systemctl show`] → The documentation names it and recommends `LoadCredential` with the `_FILE` settings, as the NixOS module does.
- [Two sources make it harder to see where a value comes from] → The documentation states the order in one place. A later change can log the source of each setting at start if operators need it.
- [The sibling change rewrites the same function and requirement] → The task in group 3 rebases and reconciles, as the section above describes.

## Migration Plan

None. Adunata has no stable release. Every variable and the grouped module options take their new names in one commit together with the Nix checks, the systemd example, the client configuration for development, the worktree plugin and the documentation. A worktree created before that commit gets the new names with `git wt refresh` after a rebase.
