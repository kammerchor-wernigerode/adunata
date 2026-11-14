# ADR 0007: Modular monolith

## Context

Ensembles host Adunata themselves, often without operations experience. At most about 10 people use one instance at the same time. Each additional process or service adds work for every operator.

Adunata covers several domain concerns, such as access, members, gatherings, invitations and mail. Coding agents write most of the code. Without boundaries, code of one concern tends to reach into the internals of another, which makes later changes expensive.

## Decision

We will run the server as one process. Its code splits into modules along domain and technical concerns. A module exposes only what it marks `pub`. Other modules use nothing else. Nobody widens visibility to reach past a module's interface, as the rules for Rust in [`AGENTS.md`](../../AGENTS.md) state.

The Rust compiler enforces these boundaries, so the build fails when code crosses one.

We considered separate services per concern, which add deployment and operations work without a benefit at this scale. We considered a dedicated tool that checks dependencies between modules, which the compiler's visibility rules make unnecessary for now.

## Status

Accepted

## Consequences

Operators run one process. Calls between modules stay function calls within one transaction where needed, such as storing a response and its mail together.

Boundaries hold only as long as nobody widens visibility. A review that sees `pub(crate)` or a new `pub` item has to ask whether it belongs to the module's interface.

The modules do not restrict in which direction they depend on each other. If dependency cycles become a problem, a check for allowed directions can follow.
