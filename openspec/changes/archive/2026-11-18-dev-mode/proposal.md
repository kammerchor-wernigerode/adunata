# Proposal

## Why

A manual debug run against a fresh database starts with no organizer, so nobody can sign in until someone sets the bootstrap settings or runs `grant-organizer`. Even then the sign-in link only arrives in Mailpit. A person or an agent that starts the server to try something has to take these steps every time before the actual work begins.

## What Changes

- `adunata serve` gets the option `--dev`. Every other setting and behavior of `serve` stays as it is. The option works in every build, but only when the public base URL is on `localhost` and the listen address is a loopback address, so a production instance cannot turn it on.
- In development mode the first organizer falls back to a fixed default member when the bootstrap settings are not set. As today, Adunata creates the first organizer only when the instance has no active organizer.
- In development mode Adunata writes a fresh sign-in link for the active organizer whose member entry is the oldest to its log at start-up. A person opens it in the browser. An agent reads it from standard output and opens it or redeems its secret through the API.
- In development mode Adunata also writes the link of every sign-in link it sends to its log. The mail still goes out.
- In development mode the limit of sign-in links per address does not apply, so frequent restarts do not block sign-in. The limit per network address stays.
- The rule that logs contain no secrets gets an exception for sign-in links in development mode.

## Capabilities

### New Capabilities

- `development-mode`: A mode of `serve` for manual debug runs on `localhost` that provides a default organizer and makes sign-in links available without a mailbox.

### Modified Capabilities

- `service-operation`: The requirement "Logs for operators" allows sign-in links in the log while development mode is on.
- `organizer-access`: The requirements "First organizer", "Sign-in by mail link" and "Limited sign-in requests" each get an exception for development mode.

## Impact

- Server: command-line interface and start-up in `server/src/main.rs`, the first organizer and the sending of sign-in links in `server/src/access.rs`.
- The client, the API and the database schema stay unchanged.
- Documentation: the README section on running Adunata, the operating documentation of `serve`, arc42 sections 5, 8 and 9 and a new decision record.
