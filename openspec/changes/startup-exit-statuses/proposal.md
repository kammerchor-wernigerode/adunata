# Proposal

## Why

Supervisors such as systemd and container runtimes act on the exit status of a process. Every failed start of Adunata exits with 1, so a supervisor cannot tell an invalid configuration from a schema that does not fit or from a database that is down. A listen address in use even panics with 101. The NixOS module and the example unit restart Adunata after every failure, also where a restart never helps, and on a fresh NixOS host Adunata can start before its database exists.

## What Changes

- `serve` and `grant-organizer` exit with a status from `sysexits.h` for the causes a supervisor treats differently. 78 means the configuration is invalid. 65 means the data in the database does not fit this build. 75 means a temporary failure of the database connection. Every other failure exits with 1, and wrong arguments keep 2.
- A listen address in use exits with 1 and names the address instead of panicking.
- **BREAKING** An invalid configuration exits with 78 instead of 1.
- The NixOS module and the example unit no longer restart after 65 or 78, and the module waits until PostgreSQL has created the role and the database.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `service-operation`: the downgrade names its status, and new requirements cover the exit status of a failed start and the restart policy of the shipped units.

## Impact

- `server/src/main.rs` and `server/src/database.rs`.
- `nix/module.nix`, `deploy/systemd/adunata.service` and `nix/test.nix`.
- `docs/operating/`, `docs/arc42/` and `docs/decisions/`.
