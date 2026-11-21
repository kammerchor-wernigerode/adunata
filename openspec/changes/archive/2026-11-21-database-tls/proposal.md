# Proposal

## Why

Adunata talks to PostgreSQL only in plain text, because sqlx is built without TLS. Operators whose database runs on another host need an encrypted connection and a check of the certificate against their own authority.

## What Changes

- sqlx gains TLS through rustls and the trust store of the host. It adds no crate.
- The operator sets `sslmode` and `sslrootcert` in `ADNT_DATABASE_URL`, as for other PostgreSQL clients.
- Without these parameters Adunata uses `prefer`. It encrypts if the server offers TLS, without a check, and connects in plain text otherwise. A Unix socket stays plain.
- Before it starts, Adunata rejects URL parameters it does not support, without printing their values, and checks that the file in `sslrootcert` is readable.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `instance-configuration`: the database URL may set the TLS mode and an authority, and Adunata checks its parameters before the start.

## Impact

- `server/Cargo.toml` and `server/Cargo.lock`: one sqlx feature and `url` as a direct dependency.
- `server/src/configuration.rs`: the check of the URL parameters.
- `nix/test.nix`: PostgreSQL with TLS.
- `docs/operating/`, `docs/arc42/` and `docs/decisions/`.
