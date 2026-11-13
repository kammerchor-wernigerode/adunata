# Design

## Context

See [proposal.md](proposal.md) for the motivation. The repository holds documentation and OpenSpec configuration but no code.

These constraints shape the approach:

- Ensembles host Adunata themselves, often without operations experience.
- The reference installation runs on NixOS. It shares a virtual machine with 2 vCPUs and 4 GB of RAM with an identity provider, a collaborative editor and databases.
- At most about 10 people use one instance at the same time.
- Coding agents write and review almost all code. One maintainer acts as architect and reviews structure and decisions rather than individual lines. That maintainer knows Java, TypeScript and React well and has not worked with Rust.

## Goals / Non-Goals

**Goals:**

- One artifact holds the server and the built browser client.
- The service stays below 150 MB of resident memory while 10 users work at the same time.
- Every level of testing that later features need exists and runs in CI, from unit tests to a NixOS VM test.
- A spike confirms or rejects the stack before the rest of the change builds on it.

**Non-Goals:**

- Sign-in and every domain feature. The change `add-gathering-invitations` adds them.
- Server-side rendering of the client.
- Support for any database other than PostgreSQL.
- Kubernetes manifests or charts.

## Decisions

Decisions 1 to 7 are drafts for records in `docs/decisions/`. No earlier record is superseded. The records for decisions 1 and 2 keep the status Proposed until the spike ends. The others are Accepted when the change lands.

### 1. Rust and React as stack

Record: `0002-rust-and-react-as-stack.md`

The server is written in Rust and the browser client in React with TypeScript. Rust gives a single static binary and a small memory footprint, which fits the shared machine and makes self-hosting simple. Its compiler rules out whole classes of errors before any review takes place. The maintainer has experience with React. The client is also where accessibility and localization live.

Alternatives considered:

- TypeScript on server and client. It is one language that the maintainer can review fully, with shared types and one translation catalog. It was the recommended option and stays the fallback if the spike fails.
- Kotlin or Java on a lightweight framework. It is the maintainer's strongest language, but it uses more memory and needs a generated contract between client and server.
- Go. It has the smallest footprint and a small language, but it would also be new to the maintainer.

The Rust code stays plain. It avoids custom macros, avoids generic traits where a concrete type works and prefers owned data over borrowed data with lifetimes. Strict lints and formatting run in CI and fail the build on any warning. Each new dependency needs a reason. These rules go into `AGENTS.md` so that coding agents follow them.

### 2. Single-page client served by the server

Record: `0003-single-page-client-with-api.md`

The React client runs in the browser and talks to a JSON API. The server embeds the built client and serves it on the same port, so operators still run one artifact. The server describes its API as an OpenAPI document. The binary writes that document to a file on request, so builds need no running server. Orval generates the client's types, TanStack Query hooks, zod schemas and mocks from that file. CI fails when the generated files do not match the document.

Alternatives considered:

- HTML rendered on the server. It works without JavaScript and needs no API, but it does not fit React without a server framework such as Next.js.
- Server-side rendering with React. It needs a JavaScript runtime next to the Rust binary and two processes to operate.

The libraries of the client are decision 7.

### 3. PostgreSQL as database

Record: `0004-postgresql-as-database.md`

Adunata supports PostgreSQL only. Tests run against PostgreSQL as well, each test with its own short-lived database.

Alternatives considered:

- SQLite only. It needs no extra service, but the reference installation already runs PostgreSQL and backs it up.
- SQLite and PostgreSQL. Every migration and test would run twice. The predecessor application supports two databases and maintains separate SQL for each.
- SQLite for development and PostgreSQL in production. Tests would then check a different database than the one in production.

### 4. Nix flake as build definition

Record: `0005-nix-flake-as-build-definition.md`

One Nix flake builds everything. It produces the package, the NixOS module, the OCI image and the development shell. The release binary and the example systemd unit come from the same package. A NixOS VM test starts the module with PostgreSQL and Mailpit and checks the health endpoint. Contributors without Nix get a Compose file that runs PostgreSQL and Mailpit, the same services the development shell starts.

Alternatives considered:

- A Dockerfile next to a Nix package. Two build definitions would drift apart.
- Kubernetes support. No ensemble of the target size runs Kubernetes. Anyone who does can run the OCI image.
- A custom SMTP sink for Compose. The Nix setup and the Compose setup would then use different tools.

### 5. Outbox for mail

Record: `0006-outbox-for-mail.md`

An action that causes a mail writes it to an outbox table in the same transaction. A background task in the same process sends mail from the outbox and retries with increasing delays. This meets the requirements in [mail-delivery](specs/mail-delivery/spec.md).

Alternatives considered:

- Sending during the request. A mail server outage would then fail the action or lose the mail.
- An external queue. It adds a service to operate for a load of a few hundred mails per gathering.

### 6. Modular monolith

Record: `0007-modular-monolith.md`

The server is one process. Its code splits into modules along domain lines, such as access, members, gatherings, invitations and mail. A module uses another only through its public interface. This change creates the modules for configuration, health, mail and localization.

Alternatives considered:

- Separate services. They add deployment and operations work without a benefit at this scale.

### 7. Client libraries

Record: `0008-client-libraries.md`

The client uses shadcn/ui for components, TanStack Router for routing, TanStack Query for server state, React Hook Form for forms and zod for validation. Vite builds the client. Vitest with Testing Library and axe tests components, Playwright runs the end-to-end tests. A library is worth adding when it takes over work that is hard to get right, such as accessible widgets, routing or caching of server state.

shadcn/ui copies its components into the repository. They build on Radix primitives and Tailwind CSS, so the project owns their code and upgrades nothing it does not touch. The server stays the authority for validation. The zod schemas in the client serve the form only. Orval generates them from the OpenAPI document, so each rule exists once.

Alternatives considered:

- MUI. It is a complete design system with a Material look, CSS-in-JS at runtime and a larger bundle. Combining it with shadcn/ui would mean two component systems.
- React Aria Components. Its accessibility and localization are the strongest, but coding agents know it less well and all styling is custom.
- A meta framework such as Next.js. It needs its own server next to the Rust binary.
- `openapi-typescript` with `openapi-fetch` instead of Orval. It generates types only, so query hooks and zod schemas would be written by hand.

### 8. Continuous integration

GitHub Actions runs the checks of the flake in three stages. Every push to a pull request runs formatting, lints, unit tests, slice tests and the contract check. A pull request that is ready for review and every push to `main` also run the end-to-end tests and the NixOS VM test. A release tag also builds and publishes the OCI image and the release binary. A newer push cancels a run that is still in progress for the same pull request.

Coding agents follow these rules for pull requests:

- An agent opens a pull request as a draft and may push to it without asking.
- Before each push the agent runs the fast checks locally. It never pushes a state that fails them.
- The agent pushes once per round of feedback. It collects every remark of a review, fixes them together and then pushes.
- The agent marks the pull request ready for review once it considers the work complete.
- After an approval the agent pushes only after asking.

These rules go into `AGENTS.md` together with the first workflow. This decision needs no record of its own, because it changes no architecture.

### Choices left to the spike

The spike picks the libraries for HTTP, database access and migrations, OpenAPI generation and localization. It also confirms that Orval accepts the OpenAPI document the server produces. The criteria are support for PostgreSQL with migrations, plural rules in both client and mail, active maintenance and code that the maintainer can read. The spike also decides whether client and server share one translation catalog. The preferred outcome is one catalog that the server delivers to the client. Each choice goes into record 0002 or into the arc42 section on cross-cutting concepts.

## Spike

The spike runs first and builds one thin slice through every risky part:

1. one form in the React client that sends data through the API into PostgreSQL, with validation errors shown in the form
2. the same migration applied in CI against a fresh PostgreSQL
3. one mail sent through the outbox to Mailpit, including a retry after a simulated outage
4. one text with a plural form in German and English, both in the client and in the mail
5. an automated accessibility check of the form
6. one binary that serves the client, with its memory measured under 10 concurrent users

The spike succeeds when all six points pass in CI, the memory goal holds and the code follows the rules for plain Rust in decision 1. It fails if any point cannot be met within those rules. In that case a new record for TypeScript on server and client supersedes record 0002. This design is then revised before the change continues.

## Testing

|Level|What it checks|
|-|-|
|Unit|domain logic without I/O|
|Server slice|an API handler called in process with real routing against a real PostgreSQL|
|Client slice|a component against an API mocked by the generated mocks, with automated accessibility checks|
|Contract|the generated client code matches the OpenAPI document, checked at build time|
|End to end|a few complete flows in a browser against the real binary, PostgreSQL and Mailpit|
|NixOS VM|the module starts, migrates the database and reports healthy|

This change sets up every level with at least one test. The end-to-end level gets its flows with the first feature.

## arc42 sections

This change creates `docs/arc42/README.md` and these sections:

|Section|Content after this change|
|-|-|
|1 Introduction and goals|purpose of Adunata, links to the capabilities, top quality goals and stakeholders|
|2 Architecture constraints|self-hosting, shared hardware, NixOS at the reference installation, AGPL-3.0, one maintainer as architect with coding agents writing the code|
|3 Context and scope|browser users, operators, the mail server and PostgreSQL around Adunata as a black box|
|4 Solution strategy|one sentence per decision above, linking to its record|
|5 Building block view|level 1 with the server modules and the client|
|7 Deployment view|NixOS module, OCI image and release binary|
|8 Cross-cutting concepts|configuration, localization, logging, persistence and migrations, testing|
|9 Architecture decisions|index of records 0001 to 0008|
|10 Quality requirements|scenarios for memory, start-up with invalid configuration, mail during an outage and plural forms|
|11 Risks and technical debt|Rust written and reviewed by coding agents for a maintainer new to it, copied UI components without upstream updates|
|12 Glossary|instance, operator, outbox and the domain terms used so far|

Section 6 stays absent until a feature adds a scenario worth showing.

## Risks / Trade-offs

- Coding agents may write Rust that compiles but is hard to change, while the maintainer reviews structure rather than lines → plain Rust rules in `AGENTS.md`, strict lints, module boundaries checked in CI, the spike with its success criteria and TypeScript as the prepared fallback.
- Client and server may need separate translation catalogs in different formats → the spike decides. Two catalogs get a check in CI that both hold the same keys.
- Components copied from shadcn/ui get no upstream fixes by themselves → the project changes them only where it must, so that a newer upstream version can replace them.
- Building the OCI image needs a Linux builder, which a macOS workstation lacks → CI builds images. Local development needs no image.
- Generated API types add a build step → it runs in the development shell and in CI, so nobody runs it by hand.

## Migration Plan

None. Adunata starts with an empty repository and an empty database. Data from the predecessor application is out of scope for this change.
