# ADR 0008: Client libraries

## Context

The client is a single-page application in React with TypeScript, see [ADR 0002](0002-rust-and-react-as-stack.md) and [ADR 0003](0003-single-page-client-with-api.md). Accessibility and localization are quality goals. Coding agents write most of the code, so libraries they know well produce better results.

Hand-built widgets such as radio groups, dialogs or menus often miss keyboard handling and announcements for screen readers. Hand-built routing and caching of server state repeat work that libraries have solved.

## Decision

We will build the client with these libraries:

|Concern|Library|
|-|-|
|Components|shadcn/ui on Radix primitives, styled with Tailwind CSS|
|Routing|TanStack Router|
|Server state|TanStack Query|
|Forms|React Hook Form|
|Validation|zod|
|API client, schemas and mocks|Orval, generated from the OpenAPI document|
|Localization|@fluent/bundle with a small binding of our own|
|Build|Vite|
|Lint|oxlint|
|Tests|Vitest, Testing Library, axe-core and Mock Service Worker|

shadcn/ui copies its components into the repository, so the project owns their code and upgrades only what it touches. The server stays the authority for validation. The zod schemas in the client serve the form only. Orval generates them from the OpenAPI document so that each rule exists once.

A library is worth adding when it takes over work that is hard to get right, such as accessible widgets, routing or caching of server state.

We considered MUI, a complete design system with a Material look, CSS-in-JS at runtime and a larger bundle. We considered React Aria Components, whose accessibility and localization are the strongest but which coding agents know less well. We considered Base UI, which shadcn/ui now offers by default but which is younger than Radix. We considered a meta framework such as Next.js, which needs its own server next to the Rust binary. We considered `openapi-typescript` with `openapi-fetch`, which generates types only.

## Status

Accepted

## Consequences

Accessible widgets, routing, server state and forms come from libraries that coding agents use well.

Copied components get no fixes from shadcn/ui by themselves. The project changes them only where it must, so that a newer version can replace them.

Generated code for the API is committed. A check fails the build when it no longer matches the server.

The client depends on JavaScript. Without it, Adunata shows nothing.
