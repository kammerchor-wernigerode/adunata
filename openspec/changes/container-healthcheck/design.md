# Design

## Context

- `/api/health` in the module `health` answers readiness with 200 or 503 and names an unavailable database.
- Docker and Podman run a healthcheck as a command in the container and read 0 as healthy and 1 as unhealthy. Docker reserves 2. How often a runtime checks, how long it waits after the start and what it does with an unhealthy container depends on the runtime and its settings. Traefik routes only to healthy containers, and Docker Swarm replaces unhealthy ones.
- The image has no shell and no HTTP client. It sets `ADNT_HTTP_LISTEN_ADDRESS` to `0.0.0.0:8080`.
- Every command of the binary loads its settings through one helper in `main.rs`, which exits on its own when they are invalid.
- After `verify-both-architectures`, the job `e2e` starts the image of each architecture against PostgreSQL.

## Goals / Non-Goals

**Goals:**

- Operators can check the readiness of an instance from inside its container, on both architectures.

**Non-Goals:**

- A healthcheck in the image. How often to check and what to do on failure is up to the operator and the runtime.
- A separate liveness endpoint. No supported installation runs on Kubernetes, and the process has no state in which it runs but cannot answer.

## Decisions

### 1. Healthcheck as a command of the binary

The module `health` gets a function that sends `GET /api/health HTTP/1.0` over a `tokio` TCP stream, reads the status line and returns the outcome. It takes the deadline as a parameter. `main.rs` dispatches the command `healthcheck` to it with ten seconds. The command loads the same configuration as `serve`, but through a path that returns the problems instead of exiting, so an invalid configuration exits with 1 too. It turns an unspecified listen address such as `0.0.0.0` or `::` into the loopback address of the same family. It exits with 0 on status 200 and with 1 otherwise. On failure it writes one line that names the cause and never the body or a setting value.

An HTTP client crate such as `reqwest` or the client of `hyper-util` would add dependencies for one request. A static `curl` in the image would grow the image and its notices.

Architecturally significant, because operators run the command. Drafted below as the record "Healthcheck as a command of the binary".

### 2. No healthcheck in the image

The image config declares no `Healthcheck`. The operating guide shows `healthcheck:` in Compose with `adunata healthcheck` and `--health-cmd` for Podman, and it says that a container started with `--config` needs `ADNT_CONFIG_FILE` instead, because a healthcheck runs without the arguments of the command. It names that a start period covers migrations at start and that some tools act on an unhealthy container.

A healthcheck in the image would impose an interval, a start period and their effect on tools such as Traefik and Docker Swarm, which only the operator can judge.

Architecturally significant, because operators wire the healthcheck themselves. Part of the record "Healthcheck as a command of the binary".

### 3. Tests

- Unit tests cover the mapping of `0.0.0.0` and `::` to loopback.
- Integration tests in `server/src/health.rs` serve the router on `127.0.0.1:0` and call the function: 200 against a fresh database, 503 against an unreachable pool, and a listener that accepts and never answers with a short deadline. A port chosen by the system keeps parallel worktrees apart.
- Process tests in `server/tests/startup.rs` run `healthcheck` against `127.0.0.1:1` and with a required setting missing, and expect status 1 both times.
- The NixOS VM test runs the container with Podman and `--health-cmd 'adunata healthcheck'`, checks that `podman healthcheck run` succeeds and that it fails after PostgreSQL stops.
- The image start in `e2e` waits until `docker exec` of `adunata healthcheck` in the container exits with 0, on both architectures.

Not architecturally significant, because tests are tooling.

## Draft decision records

### ADR NNNN: Healthcheck as a command of the binary

**Context.** Docker and Podman run a healthcheck as a command in the container and read 0 as healthy and 1 as unhealthy. The OCI image has no shell and no HTTP client. `/api/health` already reports readiness, including the database. How often to check, how long to wait after the start and what to do with an unhealthy container differs between runtimes and tools such as Traefik and Docker Swarm.

**Decision.** We will add the command `adunata healthcheck`, which asks the instance on its listen address for its readiness and exits with 0 or 1. The image declares no healthcheck. Operators wire the command into their runtime. We will add no separate liveness endpoint.

We considered an HTTP client crate or a static `curl` in the image, which add dependencies or grow the image for one request, and a healthcheck in the image, which would impose timing that only the operator can judge.

This supersedes no record.

**Status.** Accepted

**Consequences.**
- Operators get a healthcheck for Docker, Podman and Compose without a shell in the image, but they have to configure it.
- A container whose database is gone turns unhealthy once the operator wired the command.
- The command reads the full configuration. A container that passes `--config` needs `ADNT_CONFIG_FILE` instead. A configuration that became invalid on disk after the start turns a serving container unhealthy.
- A platform that separates liveness from readiness would need a new endpoint.

## Affected arc42 sections

- `03-context-and-scope.md`: the technical context of the operator names the command `healthcheck`.
- `05-building-block-view.md`: the block `health` gains the command as an interface.
- `08-cross-cutting-concepts.md`: "Health" describes the command.
- `09-architecture-decisions.md`: a row for the record.

## Risks / Trade-offs

- [Operators who never configure the healthcheck get none.] → The operating guide shows it in the Compose example.
- [The command is tested in a running container only through `docker exec` and the VM test, not through the healthcheck machinery of Docker.] → The runtimes only run the command and read its status, which both tests cover.
