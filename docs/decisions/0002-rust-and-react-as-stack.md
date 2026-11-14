# ADR 0002: Rust and React as stack

## Context

Ensembles host Adunata themselves, often without operations experience. The reference installation shares a virtual machine with 2 vCPUs and 4 GB of RAM with other services. At most about 10 people use one instance at the same time.

Coding agents write and review almost all code. The maintainer acts as architect and reviews structure and decisions rather than individual lines. The maintainer knows Java, TypeScript and React well and has not worked with Rust.

The client carries most of the work for accessibility and localization, which are among the quality goals.

## Decision

We will write the server in Rust and the browser client in React with TypeScript.

The Rust code stays plain, following the rules in [`AGENTS.md`](../../AGENTS.md). It uses no custom macros, introduces traits and generics only for at least two real implementations and prefers owned data over lifetime parameters. Formatting and lints run in CI and fail on any warning.

A spike confirms this decision before further work builds on it. The spike succeeds when one form travels from the client through the API into PostgreSQL, a mail survives a mail server outage through the outbox, plural forms render in German and English in the client and in mail, the form passes an automated accessibility check, one binary serves the client and memory stays below 150 MB with 10 concurrent users. The code has to follow the rules for plain Rust throughout. If the spike fails, a new record for TypeScript on server and client supersedes this one.

The spike met every criterion. Peak resident memory of the release build on Linux stayed at 8 MB while 10 connections sent the form and loaded the client for 30 seconds.

We will use these libraries:

|Concern|Library|
|-|-|
|HTTP server|axum on tokio|
|Database access and migrations|sqlx, with plain SQL and no compile-time query checks|
|API description|utoipa|
|Mail|lettre|
|Localization|Fluent, with fluent-bundle on the server and @fluent/bundle in the client|
|Embedding the client|rust-embed|
|Client build|Vite and TypeScript, linted by oxlint|
|Nix build of the server|crane|

We considered TypeScript on server and client, which is one language with shared types and one translation catalog. We considered Kotlin or Java on a lightweight framework, which is the maintainer's strongest language but needs more memory. We considered Go, which has the smallest footprint but would be as new to the maintainer as Rust.

## Status

Accepted

## Consequences

Adunata ships as one static binary with a small memory footprint, which keeps self-hosting simple and fits the shared server.

The Rust compiler rules out whole classes of errors before any review. Code that compiles can still be hard to change. The maintainer cannot judge idiomatic Rust at first. The rules for plain Rust and strict lints limit that risk.

Server and client use two languages. The contract between them has to be generated. Both share one Fluent catalog, which the server delivers to the client.

Rust builds take longer than builds in the alternatives, which makes CI slower. A cache of the Nix store keeps a run of unchanged dependencies to about a minute.

Fluent in Rust has no date formatting. The catalog spells out date ranges with localized month names, which covers ranges within one month.
