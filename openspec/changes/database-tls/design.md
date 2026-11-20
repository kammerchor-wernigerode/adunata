# Design

## Context

- sqlx is built without TLS, so Adunata talks to PostgreSQL only in plain text.
- The sqlx feature `tls-rustls` resolves to the webpki variant and pulls in `webpki-roots` under CDLA-Permissive-2.0, which [ADR 0016](../../../docs/decisions/0016-trust-store-of-the-host-for-mail-server-certificates.md) rules out. `tls-rustls-ring-native-roots` reads the trust store of the host through `rustls-native-certs`, which lettre already locks. It adds no crate, but two edges in `server/Cargo.lock`.
- sqlx parses `sslmode`, `sslrootcert`, `sslcert`, `sslkey`, `host`, `hostaddr`, `port`, `dbname`, `user`, `password`, `application_name`, `options` and `statement-cache-capacity` from the URL, plus aliases such as `ssl-mode`. It drops any other parameter with a warning that logs its value. It also reads `PGSSLMODE`, `PGSSLROOTCERT`, `PGSSLCERT` and `PGSSLKEY` from the environment. An invalid `PGSSLMODE` silently becomes `prefer`, and a client certificate without its key fails the handshake.
- sqlx reads `sslrootcert` only for `verify-ca` and `verify-full`, although its own documentation says `require` checks the authority too. It adds the file to the trust store of the host. It reads the file only when it connects, and a missing file shows as `No such file or directory` without the path. `PgConnectOptions` offers no getter for the file. `allow` behaves like `disable`.
- rustls reports a failed certificate check as `io::Error` of kind `InvalidData`. sqlx reports a server without TLS under `require` or stricter as `sqlx::Error::Tls`.
- PostgreSQL declines TLS on a Unix socket, and sqlx then continues without TLS under `prefer`.
- `nix/test.nix` has `testAuthorityAndMailpitCertificate`, a test authority with a certificate for `127.0.0.1`. The nodes `nixos` and `binary` trust that authority through `security.pki`. The container uses the CA bundle of the image.
- `configuration.rs` already parses `ADNT_DATABASE_URL` and validates every setting before Adunata touches the database. The crate `url` is in `server/Cargo.lock` through sqlx.
- The operating guide and arc42 describe the trust store as a concern of the mail server only.

## Goals / Non-Goals

**Goals:**

- Encrypted connections to PostgreSQL with a check against an authority of the operator.
- A mistake in the TLS parameters stops Adunata before it connects and names the cause.

**Non-Goals:**

- Client certificates. They need their own requirement and a way to hand a private key to a service with `DynamicUser`.
- TLS settings in the environment variables of libpq.
- Settings for TLS outside the database URL.

## Decisions

### 1. TLS settings in the database URL

sqlx gets the feature `tls-rustls-ring-native-roots`. The operator writes `sslmode` and `sslrootcert` into `ADNT_DATABASE_URL`, which sqlx already parses. The configuration module gets no new setting. With `verify-ca` or `verify-full`, an authority in the trust store of the host works without `sslrootcert`, as for the mail server.

The default mode stays `prefer`. A server that offers TLS now gets an encrypted connection without a check. A server that offers TLS with a broken setup now fails the handshake, where it connected in plain text before. `sslmode=disable` restores the old behavior. A Unix socket stays plain.

The operating guide documents only the URL parameters. Adunata refuses to start when one of the `PGSSL*` variables of libpq is set, as decision 2 describes. `tls-native-tls` would link OpenSSL, which the release binary would then need from the distribution. A default of `verify-full` would break installations that connect over TCP without TLS today.

Architecturally significant, because operators configure it and the binary gains a TLS stack for the database. Drafted below as the record "TLS to the database configured in its URL". It applies the principle of ADR 0016 to the database and supersedes nothing.

### 2. Check of the URL parameters before the start

The configuration module reads the query of `ADNT_DATABASE_URL` with the crate `url`, which becomes a direct dependency because the module must see each parameter on its own. It does so before it hands the URL to `PgConnectOptions`, because sqlx logs every parameter it does not know together with its value while it parses. Only a URL without unsupported parameters reaches sqlx. The module accepts the parameters that sqlx parses, including the form `options[<key>]`, except `sslcert` and `sslkey` with their aliases, which Adunata does not support yet. Every other parameter is a problem that names the parameter and not its value, so a secret such as `sslpassword` never reaches the log. If `sslrootcert` is set, the file must exist and be readable, and a problem names the parameter and the path. The module also names each of `PGSSLMODE`, `PGSSLROOTCERT`, `PGSSLCERT` and `PGSSLKEY` that is set in the environment as a problem, because sqlx would read them besides the URL. That keeps a stray mode, authority or half of a client certificate out of the connection. These problems join all other problems of the configuration, so the exit status is the one of an invalid configuration.

Leaving the check to sqlx would report a missing file only on connect, without the path, as a temporary failure, and would log unknown parameters with their values.

Architecturally significant, because operators see which URLs Adunata accepts. Part of the record "TLS to the database configured in its URL".

### 3. Tests

- Unit tests in `configuration.rs` cover an unsupported parameter whose value stays out of the message and out of the log captured with `captured_logs`, a missing `sslrootcert` file, a readable one, a set `PGSSLCERT` without `PGSSLKEY`, which is named as a problem, and an accepted URL with `sslmode`, `options[search_path]` and `statement-cache-capacity`.
- The NixOS VM test reuses `testAuthorityAndMailpitCertificate` and adds a certificate for PostgreSQL on the node `nixos`, which trusts the authority. There it checks `verify-full` with `sslrootcert`, `verify-full` with the trust store of the host alone and `prefer`, each with `SELECT ssl FROM pg_stat_ssl` for the backend of Adunata. The container does not trust the test authority. It checks a refusal with `verify-full` without `sslrootcert`, a connection with `verify-full` and `sslrootcert` set to the test authority for a host name that the certificate names, and a refusal with the same file for a host name that it does not name. So the file alone establishes the trust, and the host name check alone causes the refusal. The node `binary`, whose PostgreSQL has no TLS, checks a refusal under `sslmode=require`. The Unix socket of the module stays plain.

Not architecturally significant, because tests are tooling.

## Draft decision records

### ADR NNNN: TLS to the database configured in its URL

**Context.** Adunata talks to PostgreSQL only without TLS. Operators with a database on another host need encryption and a check against their own authority. [ADR 0016](../../../docs/decisions/0016-trust-store-of-the-host-for-mail-server-certificates.md) rules out compiled-in roots under CDLA-Permissive-2.0 and checks the mail server against the trust store of the host. PostgreSQL clients take TLS settings from `sslmode` and `sslrootcert` in the connection URL.

**Decision.** We will build sqlx with rustls and the trust store of the host. The operator chooses the TLS mode and an extra authority with `sslmode` and `sslrootcert` in the database URL. Adunata accepts only parameters it supports and checks before the start that the file in `sslrootcert` is readable.

This supersedes no record.

**Status.** Accepted

**Consequences.**
- No new setting. A URL without these parameters uses `prefer`.
- `prefer` and `require` encrypt without a check. `sslrootcert` takes effect only with `verify-ca` or `verify-full`, which the operating guide says. It recommends `verify-full` for a database on another host.
- A server with a broken TLS setup now fails under `prefer`. `sslmode=disable` restores plain text.
- A URL with a parameter Adunata does not support, such as a client certificate, stops the start, and so does a `PGSSL*` variable of libpq in the environment.
- The container needs the file of the authority mounted read-only.
- `SSL_CERT_FILE` and `SSL_CERT_DIR` now affect the database connection too.

## Affected arc42 sections

- `03-context-and-scope.md`: the row of PostgreSQL in the technical context names TLS.
- `07-deployment-view.md`: the container row says that its trust store serves the database too and names the file of the authority.
- `08-cross-cutting-concepts.md`: the passages on the trust store cover the database as well as the mail server.
- `09-architecture-decisions.md`: a row for the record.

## Risks / Trade-offs

- [A server that offers TLS with a broken setup now fails under `prefer`, where it connected in plain text before.] → The operating guide names `sslmode=disable` as the way back.
- [`sslrootcert` without `verify-ca` or `verify-full` takes no effect.] → The operating guide says so and recommends `verify-full`.
