# Proposal

## Why

Every dependency of Adunata must have a license that the OSI has approved. Two do not. The crate `webpki-roots` is under CDLA-Permissive-2.0 and the npm package `caniuse-lite` is under CC-BY-4.0. Nothing checks the rule, so both slipped in unnoticed and the risk register has carried them since. A check in CI closes the gap, so the next such dependency fails its pull request.

## What Changes

- The server verifies the certificate of the mail server against the trust store of the operating system instead of the Mozilla root certificates compiled into the binary. This drops `webpki-roots`. An operator can now trust a private certificate authority by adding it to the host's store.
- The OCI image ships a CA certificate bundle under MPL-2.0, because it has no operating system that provides one.
- The client keeps its own copy of the stylesheet `shadcn/tailwind.css` under its MIT license and no longer depends on the npm package `shadcn`. This drops `caniuse-lite` together with Babel and the rest of the command line tool of `shadcn`.
- The target `lint` gains a license check for every crate in `server/Cargo.lock` and every package in `client/package-lock.json`, including tools for build and test, against one list of approved licenses. It fails on any license outside that list. Packages from nixpkgs, such as glibc in the image or the CA bundle, stay outside this check. They belong to the later change on notices.
- The risk register drops the two licenses. The missing license notices in the artifacts stay a risk for a later change.

Out of scope are a software bill of materials, license notices in the artifacts, automated dependency updates and vulnerability scanning.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `mail-delivery`: The server certificate is verified against the trust store of the host instead of a fixed set of root certificates.

## Impact

- `server/Cargo.toml`: different TLS features of `lettre`. `rustls-native-certs` replaces `webpki-roots`.
- `flake.nix`: CA bundle in the OCI image, license check in the target `lint` with `cargo-deny` from nixpkgs.
- `client/`: copied stylesheet, `shadcn` removed from `package.json` and `package-lock.json`, a script that checks the licenses in `package-lock.json`.
- `nix/test.nix`: the NixOS VM test sends mail over TLS to a server whose certificate only the host's store trusts.
- `server/src/mail.rs`: a rejected certificate is logged as a failure of kind `encryption` instead of `connection`.
- Operators: a host without a CA store can no longer send mail over TLS. Every supported distribution, NixOS and the image have one. Operators of the image can set `SSL_CERT_FILE` to a bundle of their own.
- Documentation: the README, the arc42 sections on mail delivery, testing, deployment and risks, the operating guide and a new decision record.
