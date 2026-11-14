# Proposal

## Why

Adunata has no code yet. Every feature needs the same base first. That base is a running service that ensembles can install on their own servers, configure, update and monitor. It also needs a way to send mail reliably and to show text in more than one language. The chosen stack carries a known risk, so a spike has to confirm it before the first feature builds on it.

## What Changes

- Adunata runs as one service with one artifact that serves both the API and the browser client.
- Operators install Adunata as a NixOS module, as an OCI image or as a release binary with a systemd unit. Kubernetes is not supported.
- Operators configure an instance through environment variables and secret files. The service refuses to start with a clear message when the configuration is invalid.
- The service applies database migrations on start and reports its health on an endpoint.
- Mail goes through an outbox. The service retries failed deliveries, so a temporary mail server outage loses no mail.
- All user-visible text and every mail come from translation catalogs in German and English.
- Contributors without Nix get a Compose setup with the same services as the Nix development shell.
- A spike tests the stack end to end before the change continues. Its success criteria are in [design.md](design.md).
- Adunata is licensed under the AGPL-3.0.
- The first arc42 sections and decision records 0002 and up describe the result.

Out of scope are all domain features, including sign-in. The change `add-gathering-invitations` builds on this one.

## Capabilities

### New Capabilities

- `service-operation`: installing, starting, migrating, health reporting and stopping an Adunata instance
- `instance-configuration`: how operators configure an instance and how the service validates that configuration
- `mail-delivery`: reliable sending of mail through a configurable mail server, with retries
- `localization`: choosing the language for a user and showing text and mail in that language

### Modified Capabilities

None.

## Impact

- The repository gains a Nix flake as the single build definition. It produces the package, the NixOS module, the OCI image and a development shell.
- The repository gains the server, the browser client, GitHub Actions workflows, a Compose file and tests at every level from unit to a NixOS VM test.
- The repository gains a `LICENSE` file with the AGPL-3.0.
- PostgreSQL becomes the only supported database.
- `docs/arc42/` gets its first sections. `docs/decisions/` gets records 0002 to 0008 for the decisions in [design.md](design.md).

## Open questions

- Do the client and the server share one translation catalog? The spike decides. The preferred outcome is one catalog that the server delivers to the client.
