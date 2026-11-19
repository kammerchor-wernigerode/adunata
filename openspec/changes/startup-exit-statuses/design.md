# Design

## Context

- `main.rs` exits with 1 when the configuration is invalid and with 2 on wrong arguments, which clap decides.
- `database::connect` returns `StartupError::Unreachable`, `SchemaTooNew` or `Migration`. `main.rs` turns each into status 1. A listen address in use panics with status 101. `grant-organizer` also exits with 1 when `pool.acquire()` or `members::grant_organizer` fails.
- sqlx 0.9 retries a refused connection and the SQLSTATEs `53300` and `57P03` until the timeout and then reports `PoolTimedOut`. It returns every other connection error at once.
- `nix/module.nix` and the example unit `deploy/systemd/adunata.service` set `Restart = "on-failure"` and `RestartSec = 5`. The module orders the service after `postgresql.service`, but nixpkgs creates the role and the database in `postgresql-setup.service`, which runs after it. `postgresql.target` covers both. The VM test waits for `postgresql.service` only.
- No test in `server/tests/startup.rs` uses a database. The helpers that create one are `#[cfg(test)]` inside the binary crate, which has no library target.
- `docs/operating/configuration.md`, `docs/operating/installation.md` and the scenario "Invalid configuration" in `docs/arc42/10-quality-requirements.md` name status 1. The Compose example in `installation.md` sets no restart policy.

## Goals / Non-Goals

**Goals:**

- A supervisor tells the cause of a failed start from the status alone.
- The shipped units restart Adunata only when a restart can help.

**Non-Goals:**

- Statuses for failures after the start.
- A finer status per cause, such as `EX_NOPERM` for a wrong password.
- Statuses of `healthcheck`, which the change `container-healthcheck` defines as 0 and 1.

## Decisions

### 1. Exit statuses from `sysexits.h`

`serve` and `grant-organizer` exit with 78 (`EX_CONFIG`) when the configuration is invalid, with 65 (`EX_DATAERR`) when the data in the database does not fit the build and with 75 (`EX_TEMPFAIL`) when the connection fails for a temporary reason. Everything else keeps 1 and wrong arguments keep 2, as clap does. A supervisor treats these causes differently, because an operator fixes the configuration, fixes the data or lets it retry later. A finer status such as `EX_NOPERM` for a wrong password would lead to no other action. 125 to 127 belong to the container runtime, and 128 and above stand for signals, so both ranges stay free.

A temporary error is `sqlx::Error::Io` of any kind but `InvalidData`, `PoolTimedOut`, `PoolClosed`, `WorkerCrashed` or `sqlx::Error::Database` of SQLSTATE class `08`, `53` or `57`. Class `57` covers `57P01`, which PostgreSQL sends when it shuts down. `sysexits.h` explains `EX_TEMPFAIL` with a connection that could not be created and should be tried again later, which is what every I/O error while connecting is. A missing socket at boot and a mistyped socket path look the same to Adunata, so both give 75, and the log names the error. An allow-list of I/O kinds would misfile cases such as a host name that does not resolve while its container restarts. `InvalidData` is different, because there the connection was made and the peer was rejected.

|Cause|Status|
|-|-|
|Invalid configuration, including an unusable configuration file, mail server settings that cannot be used and development mode outside localhost|78|
|`SchemaTooNew`|65|
|`MigrateError::VersionMissing`, `VersionMismatch`, `VersionNotPresent`, `VersionTooOld`, `VersionTooNew`, `Dirty`|65|
|`MigrateError::Execute` or `ExecuteMigration` with `sqlx::Error::Database` of SQLSTATE class `22`, `23` or `42`, except `42501`|65|
|A temporary error in any phase|75|
|`sqlx::Error::Io` of kind `InvalidData`, which rustls reports for a certificate that fails the check once the database connection uses TLS|1|
|`sqlx::Error::Tls`, such as a server without TLS|1|
|Any other cause, such as a wrong password, a missing privilege or a listen address in use|1|

One pure function takes the parts of a `sqlx::Error` it needs, such as the variant, the I/O kind and the SQLSTATE, and returns whether the error is temporary. `StartupError` gets a method that returns the status. `main.rs` exits with that status in `connect`, `prepare_organizers`, the `pool.acquire()` and the call of `members::grant_organizer` in `grant-organizer`. The configuration helper exits with 78. Binding the listen address moves into a function that returns its error, and `main.rs` logs the address and exits with 1 instead of panicking. The order of the start stays as it is.

Status 75 for every connection error would tell a supervisor that a wrong password is worth a retry. Status 64 (`EX_USAGE`) for wrong arguments would fight the default of clap for no other action.

Architecturally significant, because supervisors and operators act on the status. Drafted below as the record "Exit statuses from sysexits for a failed start".

### 2. Restart policy of the shipped units

`nix/module.nix` and `deploy/systemd/adunata.service` keep `Restart=on-failure` and `RestartSec=5` and add `RestartPreventExitStatus=65 78`. A restart helps after 75 and after a crash, but never after 65 or 78. An operator overrides the policy in `systemd.services.adunata.serviceConfig` or in a drop-in. The module orders and requires `postgresql.target` instead of `postgresql.service` when it creates the database, so Adunata starts after the role and the database exist.

Removing the policy would leave an instance down after a crash or a slow database at boot until someone acts, which hits the operator without operations experience whom the quality goal "Operability" names.

Architecturally significant, because operators see how the service reacts to each status. Part of the record "Exit statuses from sysexits for a failed start".

### 3. Tests

- A unit test covers the classifying function for every row of the table, with no I/O.
- Integration tests in `database::integration` assert the status of `StartupError` against a real database: 65 for a schema ahead of the build, next to `start_after_a_downgrade`, 65 for a changed checksum and 1 for a database that does not exist.
- An integration test occupies a port with a listener and asserts status 1 for the bind function.
- Process tests in `server/tests/startup.rs` cover 78 in the existing tests for invalid configuration, 2 for an unknown command and 75 against `127.0.0.1:1`.
- The NixOS VM test waits for `postgresql.target` and checks that a service that exits with 78 stays stopped.

Not architecturally significant, because tests are tooling.

## Draft decision records

### ADR NNNN: Exit statuses from sysexits for a failed start

**Context.** Supervisors such as systemd and container runtimes act on the exit status of a process. Every failed start of Adunata exits with 1, so the cause is only in the log text. The shipped systemd units restart on every failure, also where a restart never helps. Container runtimes use 125 to 127, and statuses from 128 stand for signals. `sysexits.h` defines 64 to 78 with fixed meanings.

**Decision.** We will exit `serve` and `grant-organizer` with 78 (`EX_CONFIG`) when the configuration is invalid, with 65 (`EX_DATAERR`) when the data in the database does not fit the build and with 75 (`EX_TEMPFAIL`) when the connection to the database fails for a temporary reason. Every other failure keeps 1 and wrong arguments keep 2. The NixOS module and the example unit restart on failure but not after 65 or 78.

This supersedes no record.

**Status.** Accepted

**Consequences.**
- A supervisor can retry on 75 and stop on 65 or 78 without reading logs.
- An invalid configuration no longer exits with 1. Scripts that tested for 1 need to test for 78.
- A wrong password, certificate or privilege exits with 1.
- A mistyped socket path exits with 75 like a socket that is not there yet. The log names the error.
- Operators who want another restart policy override the one of the module or the unit.
- Docker cannot exclude statuses from `on-failure`, so a container restarts after 65 and 78 too.
- Statuses 65, 75 and 78 become part of the interface for operators. Changing them needs a new record.

## Affected arc42 sections

- `03-context-and-scope.md`: the technical context of the operator names the exit status as a channel and links to the table in the operating guide.
- `07-deployment-view.md`: the rows of NixOS and of other distributions name the restart policy.
- `08-cross-cutting-concepts.md`: "Persistence and migrations" links to the table in the operating guide.
- `09-architecture-decisions.md`: a row for the record.
- `10-quality-requirements.md`: the scenario "Invalid configuration" expects status 78.

## Risks / Trade-offs

- [Scripts that tested an invalid configuration for status 1 break.] → The record says so, and the title of the pull request names the breaking change, so `--generate-notes` carries it into the release notes.
- [A database error during a migration outside the listed classes exits with 1, although a fresh database might cure it.] → The log names the SQLSTATE.
- [A mistyped socket path makes systemd retry every five seconds.] → The log names the failed path on every attempt.
