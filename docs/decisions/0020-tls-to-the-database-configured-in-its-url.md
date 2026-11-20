# ADR 0020: TLS to the database configured in its URL

## Context

Adunata talks to PostgreSQL only without TLS. Operators with a database on another host need encryption and a check against their own authority. [ADR 0016](0016-trust-store-of-the-host-for-mail-server-certificates.md) rules out compiled-in roots under CDLA-Permissive-2.0 and checks the mail server against the trust store of the host. PostgreSQL clients take TLS settings from `sslmode` and `sslrootcert` in the connection URL.

## Decision

We will build sqlx with rustls and the trust store of the host. The operator chooses the TLS mode and an extra authority with `sslmode` and `sslrootcert` in the database URL. Adunata accepts only parameters it supports and checks before the start that the file in `sslrootcert` is readable.

This supersedes no record.

## Status

Accepted

## Consequences

- No new setting. A URL without these parameters uses `prefer`.
- `prefer` and `require` encrypt without a check. `sslrootcert` takes effect only with `verify-ca` or `verify-full`, which the operating guide says. It recommends `verify-full` for a database on another host.
- A server with a broken TLS setup now fails under `prefer`. `sslmode=disable` restores plain text.
- A URL with a parameter Adunata does not support, such as a client certificate, stops the start. A `PGSSL*` variable of libpq in the environment stops it too.
- The container needs the file of the authority mounted read-only.
- `SSL_CERT_FILE` and `SSL_CERT_DIR` now affect the database connection too.
