# Proposal

## Why

Docker, Podman and Compose can tell whether a container is healthy only through a command that runs inside it. The image of Adunata has no shell or HTTP client that such a command could use. `/api/health` already reports readiness, but nothing inside the container can ask it.

## What Changes

- A new command `adunata healthcheck` asks the running instance on its listen address for its readiness. It exits with 0 when the instance is ready and with 1 otherwise, and names the cause in one line.
- The image declares no healthcheck. The operating guide shows how operators wire the command into Compose and Podman.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `service-operation`: new requirements for the command `healthcheck`, its output and the healthcheck that the operator configures.

## Impact

- `server/src/health.rs`, `server/src/main.rs` and `server/tests/startup.rs`.
- `nix/test.nix` and the job `e2e` in `.github/workflows/ci.yaml`.
- `docs/operating/`, `docs/arc42/` and `docs/decisions/`.
- Depends on `verify-both-architectures` for the image start in `e2e` on both architectures.
