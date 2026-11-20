# ADR 0022: Healthcheck as a command of the binary

## Context

Docker and Podman run a healthcheck as a command in the container. They read exit status 0 as healthy and 1 as unhealthy. The OCI image has no shell and no HTTP client. `/api/health` already reports readiness, including the database.

How often to check, how long to wait after the start and what to do with an unhealthy container differs between runtimes. Tools act on the result as well. Traefik routes only to healthy containers, and Docker Swarm replaces unhealthy ones.

## Decision

We will add the command `adunata healthcheck`, which asks the instance on its listen address for its readiness and exits with 0 or 1. The image declares no healthcheck. Operators wire the command into their runtime. We will add no separate liveness endpoint.

We considered an HTTP client crate and a static `curl` in the image. Both add dependencies or grow the image for one request. We also considered a healthcheck in the image, which would impose timing that only the operator can judge.

This supersedes no record.

## Status

Accepted

## Consequences

Operators get a healthcheck for Docker, Podman and Compose without a shell in the image, but they have to configure it.

A container whose database is gone turns unhealthy once the operator wired the command.

The command reads the full configuration. A container that passes `--config` needs `ADNT_CONFIG_FILE` instead, because a healthcheck runs without the arguments of the main command. A configuration that became invalid on disk after the start turns a serving container unhealthy.

A platform that separates liveness from readiness would need a new endpoint.
