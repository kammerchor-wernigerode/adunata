# Solution strategy

## Fundamental decisions

|Topic|Approach|Record|
|-|-|-|
|Stack|The server is written in plain Rust and the client in React with TypeScript. The server compiles to one native binary with a small memory footprint, which fits a small shared server.|[ADR 0002](../decisions/0002-rust-and-react-as-stack.md)|
|Client model|A single-page client talks to a JSON API. One binary serves both on one port. The client code for the API is generated from an OpenAPI document.|[ADR 0003](../decisions/0003-single-page-client-with-api.md)|
|Database|PostgreSQL is the only database, also in tests.|[ADR 0004](../decisions/0004-postgresql-as-database.md)|
|Mail|Every mail goes through an outbox in the database. A background task sends it and retries with growing delays up to a configured number of attempts.|[ADR 0006](../decisions/0006-outbox-for-mail.md)|
|Decomposition|One process whose modules use each other only through their public interface. The compiler enforces the boundaries.|[ADR 0007](../decisions/0007-modular-monolith.md)|
|Client libraries|Accessible widgets, routing, server state and forms come from established libraries. The API client is generated.|[ADR 0008](../decisions/0008-client-libraries.md)|
|Sign-in|Organizers sign in with a single-use link by mail and keep a session cookie whose hash the database stores. Every request that changes data must come from the origin of the instance.|[ADR 0010](../decisions/0010-sign-in-by-mail-link.md)|
|Personal links|Members answer through a personal link whose secret sits in the fragment of the URL, so no request line, log or referrer carries it. The database stores only its hash.|[ADR 0011](../decisions/0011-personal-links-in-the-url-fragment.md)|
|Build and deployment|One Nix flake builds the NixOS module, the OCI image and a release binary for glibc and runs every check except the end-to-end tests.|[ADR 0005](../decisions/0005-nix-flake-as-build-definition.md), [ADR 0015](../decisions/0015-release-binary-linked-against-glibc.md)|

## How the architecture reaches the quality goals

The goals are those of [Introduction and goals](01-introduction-and-goals.md#quality-goals), in their order.

|Goal|Approach|Details|
|-|-|-|
|Operability|One binary holds the server, the client and the migrations. It checks the whole configuration at start, names every problem at once and exits with a status that tells a service manager whether a restart helps. A health endpoint reports which dependency is unavailable.|[Configuration](08-cross-cutting-concepts.md#configuration), [health](08-cross-cutting-concepts.md#health), [ADR 0014](../decisions/0014-optional-configuration-file-in-toml.md), [ADR 0019](../decisions/0019-exit-statuses-from-sysexits-for-a-failed-start.md), [ADR 0022](../decisions/0022-healthcheck-as-a-command-of-the-binary.md)|
|Data protection and security|The database stores sessions and links only as hashes of their secrets. A mail that waits in the outbox still holds its link. It loses its content once its delivery ends. Responses expire after a retention period. Logs carry no personal data.|[Personal links](08-cross-cutting-concepts.md#personal-links), [retention of personal data](08-cross-cutting-concepts.md#retention-of-personal-data), [logging](08-cross-cutting-concepts.md#logging), [ADR 0013](../decisions/0013-mail-loses-its-content-once-its-delivery-ends.md)|
|Accessibility|Widgets come from a library that implements the keyboard and screen reader patterns. Rules for focus, errors and contrast hold for every page. Tests check them on two levels.|[User interface](08-cross-cutting-concepts.md#user-interface), [testing](08-cross-cutting-concepts.md#testing), [ADR 0008](../decisions/0008-client-libraries.md)|
|Localization|Server and client read the same catalog per language, so each text exists once. No module holds a text for people outside a catalog.|[Localization](08-cross-cutting-concepts.md#localization), [validation and error handling](08-cross-cutting-concepts.md#validation-and-error-handling)|
|Efficiency|The server is one native process with a small pool of database connections. It needs no service besides PostgreSQL and a mail server.|[Process model](08-cross-cutting-concepts.md#process-model), [ADR 0002](../decisions/0002-rust-and-react-as-stack.md), [ADR 0007](../decisions/0007-modular-monolith.md)|
