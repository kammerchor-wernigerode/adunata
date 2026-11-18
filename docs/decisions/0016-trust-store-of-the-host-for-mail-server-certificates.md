# ADR 0016: Trust store of the host for mail server certificates

## Context

Adunata verifies the certificate of the mail server. The Mozilla root certificates compiled into the binary come from the crate `webpki-roots` under CDLA-Permissive-2.0, which the OSI has not approved. Every dependency must have an approved license. A fixed set of roots also ignores the host. An operator cannot trust a private certificate authority, and only a new release brings changes to the set. The OCI image has no operating system and no store of its own.

## Decision

We will verify the certificate of the mail server against the trust store of the host. The release binary and the NixOS module use the store of the distribution. The OCI image ships the CA bundle of nixpkgs under MPL-2.0 and points `SSL_CERT_FILE` at it.

## Status

Accepted

## Consequences

- No dependency outside the approved licenses remains for TLS.
- An operator trusts a private certificate authority by adding it to the host's store.
- Updates of the distribution's store reach a running instance after a restart, without a new release.
- A host without a store cannot send mail over TLS. Every supported distribution and NixOS have one.
- The image trusts the authorities of the nixpkgs it was built with until a new image replaces it. An operator can point `SSL_CERT_FILE` at their own bundle, which replaces the bundle of the image.
- `SSL_CERT_FILE` and `SSL_CERT_DIR` are the only settings without the prefix `ADNT_`. The configuration module does not validate them, so a wrong path shows only as failed deliveries of kind `encryption`.
