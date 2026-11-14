# ADR 0003: Single-page client with API

## Context

The browser client is written in React, see [ADR 0002](0002-rust-and-react-as-stack.md). The server is a Rust binary. Operators should run a single artifact without a second runtime next to it.

Rendering HTML on the server would let the response form work without JavaScript. React renders on the server only with a JavaScript runtime and usually with a framework such as Next.js.

## Decision

We will build the client as a single-page application that runs in the browser and talks to a JSON API. The server embeds the built client and serves it on the same port as the API.

The server describes its API as an OpenAPI document. The binary writes that document to a file on request, so builds need no running server. Orval generates the client's types, query hooks, validation schemas and test mocks from that file. CI fails when the generated code does not match the document.

We considered HTML rendered on the server, which works without JavaScript and needs no API but does not fit React. We considered server-side rendering with React, which needs a JavaScript runtime next to the binary and two processes to operate.

## Status

Accepted

## Consequences

Operators still run one artifact on one port.

The client needs JavaScript. Accessibility depends on the components and on automated checks instead of plain HTML.

The API becomes an explicit contract with a generated client. A change to the API that the client does not follow fails the build.

The generated code adds a build step. The development shell and CI run it, so nobody runs it by hand.
