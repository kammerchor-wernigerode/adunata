# ADR 0019: Exit statuses from sysexits for a failed start

## Context

Supervisors such as systemd and container runtimes act on the exit status of a process. Every failed start of Adunata exits with 1, so the cause is only in the log text. The shipped systemd units restart on every failure, also where a restart never helps. Container runtimes use 125 to 127. Statuses from 128 stand for signals. `sysexits.h` defines 64 to 78 with fixed meanings.

## Decision

We will exit `serve` and `grant-organizer` with 78 (`EX_CONFIG`) when the configuration is invalid, with 65 (`EX_DATAERR`) when the data in the database does not fit the build and with 75 (`EX_TEMPFAIL`) when the connection to the database fails for a temporary reason. Every other failure keeps 1 and wrong arguments keep 2. The NixOS module and the example unit restart on failure but not after 65 or 78.

A finer status per cause, such as `EX_NOPERM` for a wrong password, would lead an operator to no other action. Status 75 for every connection error would tell a supervisor that a wrong password is worth a retry. Status 64 (`EX_USAGE`) for wrong arguments would fight the default of clap for no other action.

This supersedes no record.

## Status

Accepted

## Consequences

A supervisor can retry on 75 and stop on 65 or 78 without reading logs.

An invalid configuration no longer exits with 1. Scripts that tested for 1 need to test for 78.

A wrong password, certificate or privilege exits with 1.

A mistyped socket path exits with 75 like a socket that is not there yet. The log names the error.

Operators who want another restart policy override the one of the module or the unit.

Docker cannot exclude statuses from `on-failure`, so a container restarts after 65 and 78 too.

Statuses 65, 75 and 78 become part of the interface for operators. Changing them needs a new record.
