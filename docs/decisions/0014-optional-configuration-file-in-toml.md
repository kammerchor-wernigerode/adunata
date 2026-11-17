# ADR 0014: Optional configuration file in TOML

## Context

Adunata takes its configuration from environment variables and reads each secret from a file a variable names. This fits NixOS, containers and systemd. An operator who wants to keep the configuration in one file, or who has no secret store, has to put every setting into a unit file, a Compose file or a script.

Formats for configuration differ in how much they guess about types and whether they allow comments.

## Decision

We will read an optional configuration file in TOML that the operator names with `--config` or `ADNT_CONFIG_FILE`. Adunata never looks for a file on its own.

The file groups the settings in fixed tables by concern, with one key in kebab-case per setting. Each environment variable derives from table and key, after the relaxed binding of Spring Boot, so both forms share one scheme. `public-url` in the table `http` becomes `ADNT_HTTP_PUBLIC_URL`.

The environment wins over the file. Every secret can also be a plain value, and one source must not set both forms of a secret.

We considered YAML, which guesses types, JSON, which has no comments, and dotenv, which systemd and Compose already read as environment.

## Status

Accepted

## Consequences

Operators choose between a file, the environment and a secret store, or combine them.

Every variable changed its name before the first stable release. Earlier records, such as [ADR 0012](0012-wall-clock-times-in-the-instance-time-zone.md), use the old names:

|Old name|New name|
|-|-|
|`ADUNATA_PUBLIC_URL`|`ADNT_HTTP_PUBLIC_URL`|
|`ADUNATA_LISTEN_ADDRESS`|`ADNT_HTTP_LISTEN_ADDRESS`|
|`ADUNATA_TRUSTED_PROXIES`|`ADNT_HTTP_TRUSTED_PROXIES`|
|`ADUNATA_DATABASE_URL`|`ADNT_DATABASE_URL`|
|`ADUNATA_DATABASE_PASSWORD_FILE`|`ADNT_DATABASE_PASSWORD_FILE`|
|`ADUNATA_SMTP_HOST`|`ADNT_SMTP_HOST`|
|`ADUNATA_SMTP_PORT`|`ADNT_SMTP_PORT`|
|`ADUNATA_SMTP_ENCRYPTION`|`ADNT_SMTP_ENCRYPTION`|
|`ADUNATA_SMTP_USERNAME`|`ADNT_SMTP_USERNAME`|
|`ADUNATA_SMTP_PASSWORD_FILE`|`ADNT_SMTP_PASSWORD_FILE`|
|`ADUNATA_MAIL_SENDER`|`ADNT_MAIL_SENDER`|
|`ADUNATA_MAIL_MAX_ATTEMPTS`|`ADNT_MAIL_MAX_ATTEMPTS`|
|`ADUNATA_DEFAULT_LANGUAGE`|`ADNT_LOCALIZATION_DEFAULT_LANGUAGE`|
|`ADUNATA_TIME_ZONE`|`ADNT_LOCALIZATION_TIME_ZONE`|
|`ADUNATA_RETENTION_COMMENT_DAYS`|`ADNT_RETENTION_COMMENT_DAYS`|
|`ADUNATA_RETENTION_RESPONSE_DAYS`|`ADNT_RETENTION_RESPONSE_DAYS`|
|`ADUNATA_PRIVACY_URL`|`ADNT_LEGAL_PRIVACY_URL`|
|`ADUNATA_IMPRINT_URL`|`ADNT_LEGAL_IMPRINT_URL`|
|`ADUNATA_FIRST_ORGANIZER_MAIL`|`ADNT_BOOTSTRAP_ORGANIZER_MAIL`|
|`ADUNATA_FIRST_ORGANIZER_FIRST_NAME`|`ADNT_BOOTSTRAP_ORGANIZER_FIRST_NAME`|
|`ADUNATA_FIRST_ORGANIZER_LAST_NAME`|`ADNT_BOOTSTRAP_ORGANIZER_LAST_NAME`|

`ADNT_DATABASE_PASSWORD` and `ADNT_SMTP_PASSWORD` are new and have no old name.

Adunata gains the dependency `toml`.

A plain secret in the environment is visible to processes of the same user, to `docker inspect` and, in a systemd unit, to every local user through `systemctl show`. The documentation names these cases.

Every new setting needs a key in a table, or the file cannot set it.
