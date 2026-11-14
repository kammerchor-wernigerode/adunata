# ADR 0004: PostgreSQL as database

## Context

Ensembles host Adunata themselves. The reference installation already runs PostgreSQL and backs it up. On NixOS, PostgreSQL is one line of configuration. With Compose, it is one more container, which operators know from other self-hosted applications.

Supporting several databases means writing, migrating and testing every query for each of them. The predecessor application supports two databases and keeps separate SQL for each.

Tests that run against a different database than production miss differences in types, locking, constraints and time zones.

## Decision

We will support PostgreSQL as the only database. Tests run against PostgreSQL as well, each test with its own empty database that has all migrations applied.

We considered SQLite only, which needs no extra service but does not fit the existing operation of the reference installation. We considered SQLite and PostgreSQL, which doubles every migration and test. We considered SQLite for development and tests with PostgreSQL in production, which tests a different database than the one that runs.

## Status

Accepted

## Consequences

Queries and migrations can use features of PostgreSQL without a lowest common denominator.

Every operator needs a PostgreSQL server. Adunata does not run with a file-based database.

Development and CI need a running PostgreSQL. The development shell, the Compose file and the checks of the flake provide one.
