# Design

## Context

`lettre` is built with the feature `tokio1-rustls-tls`. It bundles `rustls-tls`, which enables the feature `webpki-roots`, so the binary carries the Mozilla root certificates as compiled data. `cargo tree -i webpki-roots` shows `lettre` as the only path.

`shadcn` is a runtime dependency of the client. The client imports only `shadcn/tailwind.css` in `client/src/index.css`. The package also brings its command line tool, and `browserslist` pulls `caniuse-lite` into that tool.

The licenses of every dependency can be read without network access. `cargo metadata` gives an SPDX expression for each crate. `client/package-lock.json` has a `license` field for each of its 619 packages. Apart from the two known cases, all licenses found today are approved by the OSI.

The NixOS VM test sends mail without encryption, so no test covers how the certificate of the mail server is verified.

## Goals / Non-Goals

**Goals:**

- No crate in `server/Cargo.lock` and no package in `client/package-lock.json` with a license outside the approved list, checked on every push.
- One list of approved licenses for server and client.

**Non-Goals:**

- License notices in the artifacts and a software bill of materials. The row in the risk register stays.
- Licenses of what nixpkgs provides, such as glibc and the CA bundle in the image or the tools of the flake. The change on the software bill of materials covers that closure.
- Checks for advisories, banned crates or sources. `cargo-deny` can run them later, but this change enables only its license check.

## Decisions

### Trust store of the host for mail server certificates

`lettre` gets the features `tokio1-rustls`, `ring` and `rustls-native-certs` instead of `tokio1-rustls-tls`. `rustls-native-certs` loads the certificates from the store of the operating system. It honors `SSL_CERT_FILE` and `SSL_CERT_DIR` and otherwise probes the usual paths such as `/etc/ssl/certs`. `ring` stays the crypto provider, so the TLS handshake itself does not change.

Alternatives:
- `native-tls` links OpenSSL dynamically. The check `releaseLinkage` allows only glibc and libgcc_s, see ADR 0015.
- `rustls-platform-verifier` also uses the host's store but adds more crates and gains nothing on Linux.
- Keep `webpki-roots` as an exception to the rule. That weakens a rule that a check is about to enforce.

Significant: yes. An operator notices which certificate authorities the instance trusts and that a private one now works through the host's store.

### Failure kind `encryption` for a rejected certificate

lettre reports a failed TLS handshake as a connection error, so `failure_kind` in `server/src/mail.rs` logs `connection` for an untrusted certificate, the same as for a closed port. `failure_kind` walks the chain of `std::error::Error::source()` and reports `encryption` when it finds a `std::io::Error` of kind `InvalidData`, which is how tokio-rustls hands on an error of rustls. The text of that error stays out of the log, as for every other kind. An I/O error of that kind without an inner error, such as invalid UTF-8 in a reply of the server, stays `connection`. This needs no new dependency. Unit tests cover the walk through the chain with plain I/O errors. Only the VM test shows that lettre and tokio-rustls actually produce that chain for a rejected certificate.

Significant: no. The log already names a failure kind, and this makes it accurate.

### CA bundle in the OCI image

The image has no operating system and therefore no store. Its `Env` sets `SSL_CERT_FILE` to the bundle of `pkgs.cacert`, which is under MPL-2.0. The store path in `Env` pulls the bundle into the image, as `PATH` does for the server today. The release binary and the NixOS module use the store of their host.

Significant: yes, part of the record above. An operator of the image gets a fixed set of authorities from nixpkgs that a new image updates, and can set `SSL_CERT_FILE` to a bundle of their own.

`SSL_CERT_FILE` and `SSL_CERT_DIR` are the conventional variables of OpenSSL, which `rustls-native-certs` reads. They replace the store rather than add to it, so an operator's bundle has to contain the public authorities too if the mail server needs them. Adunata keeps these names instead of an `ADNT_` setting, because every TLS tool on the host reads the same ones. `docs/operating/configuration.md` names them as the one exception to the prefix `ADNT_`. The configuration module does not validate them. A wrong path shows up as failures of kind `encryption`, which the operating guide names under troubleshooting.

Alternatives: a typed setting `ADNT_SMTP_CA_FILE` that adds a file through lettre would be validated at start, but needs more code and still leaves `SSL_CERT_FILE` for the image. A bundle at `/etc/ssl/certs/ca-certificates.crt` that operators mount over still leaves the variables honored and undocumented.

### Copied stylesheet instead of the package `shadcn`

`shadcn/tailwind.css` is copied to `client/src/shadcn.css` with a header that names the source, its version and the copyright notice and text of its MIT license. `index.css` imports the copy. `shadcn` leaves `package.json`. Adding a component later runs the command line tool through `npx shadcn@4.23.0`, the version of the copy, without making it a dependency of the project. That run fetches the tool with `caniuse-lite` outside the lock file and outside the license check, which the README says.

Alternative: keep `shadcn` as a dev dependency. The rule covers tools for build and test, so `caniuse-lite` would still violate it.

Significant: no. Neither an operator nor a running instance notices where the stylesheet comes from.

### `cargo-deny` for the crates

`cargo-deny` from nixpkgs checks the licenses of the crates. Crane's `cargoDeny` runs it offline with `cargoDenyChecks = "licenses"`. crane's `commonCargoSources` keeps every TOML file, so `serverSource` subtracts `server/deny.toml` with `lib.fileset.difference` and the check gets a source of its own that contains it. A change to the list then rebuilds neither the server nor its tests. Its configuration `server/deny.toml` allows only licenses approved by the OSI that a dependency needs, ignores the private crate `adunata` and includes dev dependencies. It sets `unused-allowed-license = "allow"`, because the shared list also holds licenses that only the client needs. A missing or unparsable license fails the check.

Alternative: a script over `cargo metadata`. It would have to evaluate SPDX expressions with `OR`, `AND` and `WITH`, which `cargo-deny` already does correctly.

Significant: no. It is a check in CI.

### Script for the npm packages

`client/scripts/check-licenses.mjs` reads `client/package-lock.json` and takes the allowed licenses from `server/deny.toml`, so both checks use the same list. It skips the root entry `""`, which is the private client itself. It evaluates `OR`, `AND` and parentheses. Any other form, a missing field or an unknown identifier fails the check and names the package. The script has no npm dependencies. `npm run lint:licenses` runs it. The flake check `clientLicenses` is a `runCommand` with only Node.js, the script, `client/package-lock.json` and `server/deny.toml`, so the fileset of the client and with it the build of client and server stay unchanged.

Alternatives: the npm package `license-checker` adds dependencies to check dependencies. A scanner such as Trivy belongs to the later change on vulnerability scanning.

Significant: no. It is a check in CI.

### Allowed licenses

The list in `server/deny.toml` holds the licenses approved by the OSI that the dependencies need once `webpki-roots` and `shadcn` are gone. A license is needed if a dependency has no other approved way to satisfy its expression. A license that appears only next to an approved alternative, such as `BSL-1.0` in `Apache-2.0 OR BSL-1.0`, stays out. Today's count of what appears gives `0BSD`, `Apache-2.0`, `BlueOak-1.0.0`, `BSD-2-Clause`, `BSD-3-Clause`, `BSL-1.0`, `ISC`, `MIT`, `MPL-2.0`, `OFL-1.1`, `Python-2.0`, `Unicode-3.0`, `Unlicense` and `Zlib`. Some of them may leave with `shadcn`, and `rustls-native-certs` may bring others, so the list is counted again after both. Expressions such as `MIT OR CC0-1.0` pass through their approved alternative. A new license enters the list in the commit that needs it, after a check against the list of the OSI.

### Verification in the NixOS VM test

The test builds a test certificate authority and a certificate for the mail server with `openssl` at build time. The certificate names `IP:127.0.0.1` as subject alternative name, because Adunata connects to that address. Each node runs Mailpit with implicit TLS through `--smtp-tls-cert`, `--smtp-tls-key` and `--smtp-require-tls`. Each case requests a sign-in link for a bootstrap organizer with a POST to `/api/sign-in-requests`, as the memory check does.

- The node `nixos` adds the authority to `security.pki.certificateFiles` and uses `encryption = "tls"`. The sign-in mail has to arrive in Mailpit. This covers the hardened unit of the module, whose `ProtectSystem=strict` leaves `/etc/ssl` readable.
- The node `binary` does the same with the unit of the release. It covers the second hardened unit and a binary that runs outside the Nix store.
- The node `container` runs the image twice. Without trusting the authority, Mailpit receives nothing and the log shows `mail delivery failed` with `failure=encryption`. With the authority mounted and `SSL_CERT_FILE` pointing to it, the mail arrives. This shows that the binary honors `SSL_CERT_FILE`, which the bundle of the image and an operator's own bundle rely on. The test also checks that the bundle in the image exists and is not empty.

A public certificate cannot be tested without network access. The check of the bundle in the image covers that path.

### Affected arc42 sections

- `08-cross-cutting-concepts.md`: mail delivery names the host's trust store instead of the Mozilla root certificates. Configuration names `SSL_CERT_FILE` and `SSL_CERT_DIR` as settings that a library reads past the configuration module. The row `lint` in the testing table names the license check.
- `07-deployment-view.md`: the image carries a CA bundle from nixpkgs.
- `09-architecture-decisions.md`: the new record.
- `11-risks-and-technical-debt.md`: the row on the two licenses goes. The row on notices stays.
- `02-architecture-constraints.md` keeps its rule unchanged.

### Draft decision record: ADR 0016 Trust store of the host for mail server certificates

Supersedes none.

**Context.** Adunata verifies the certificate of the mail server. The Mozilla root certificates compiled into the binary come from the crate `webpki-roots` under CDLA-Permissive-2.0, which the OSI has not approved. Every dependency must have an approved license. A fixed set of roots also ignores the host. An operator cannot trust a private certificate authority, and only a new release brings changes to the set. The OCI image has no operating system and no store of its own.

**Decision.** We will verify the certificate of the mail server against the trust store of the host. The release binary and the NixOS module use the store of the distribution. The OCI image ships the CA bundle of nixpkgs under MPL-2.0 and points `SSL_CERT_FILE` at it.

**Status.** Accepted.

**Consequences.**
- No dependency outside the approved licenses remains for TLS.
- An operator trusts a private certificate authority by adding it to the host's store.
- Updates of the distribution's store reach a running instance after a restart, without a new release.
- A host without a store cannot send mail over TLS. Every supported distribution and NixOS have one.
- The image trusts the authorities of the nixpkgs it was built with until a new image replaces it. An operator can point `SSL_CERT_FILE` at their own bundle, which replaces the bundle of the image.
- `SSL_CERT_FILE` and `SSL_CERT_DIR` are the only settings without the prefix `ADNT_`. The configuration module does not validate them, so a wrong path shows only as failed deliveries of kind `encryption`.

## Risks / Trade-offs

- [A minimal host or a hardened unit cannot read the store] → Both units use `ProtectSystem=strict`, which keeps `/etc` readable. The VM test sends TLS mail through both units and the image. The operating guide names the store as a requirement. lettre ignores errors while it loads the store, so a missing store shows only as failed deliveries of kind `encryption`, which the operating guide names under troubleshooting.
- [The image now ships `cacert` under MPL-2.0, which requires a notice] → The remaining row on notices in the risk register covers it until the change on notices.
- [An approved license appears in an unfamiliar form, such as `Apache-2.0 WITH LLVM-exception` on npm] → The script fails closed and names the package. A maintainer decides and extends the list or the script.
- [The `license` field in `package-lock.json` is what a package declares, not what its files say] → Accepted for this check. The later change on notices reads the license files themselves.
- [The copied stylesheet falls behind upstream] → The header names the version. The risk register already covers copied UI components.

## Migration Plan

Operators of the binary or the NixOS module change nothing, as long as their host has a store with the authority of their mail server. Operators of the image change nothing. Operators who relied on a mail server certificate that only the Mozilla set trusts are unaffected, because distributions ship the same set. Rollback is the previous release.
