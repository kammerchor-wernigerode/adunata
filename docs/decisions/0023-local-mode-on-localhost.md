# ADR 0023: Local mode on localhost

## Context

[ADR 0018](0018-development-mode-on-localhost.md) gives a debug run on the own machine a default organizer and sign-in links in the log, behind the option `serve --dev`. Its guard allows it only when the public URL is on `localhost` and Adunata listens on a loopback address.

Test systems on other hosts need an option to confine mail, and `--dev` is the name that fits them. The sign-in shortcut cannot share that option, because its guard refuses every host but `localhost`.

Debug runs happen with debug builds and with release builds, such as the binary of the end-to-end tests. `debug` is also a log level.

## Decision

We will offer the shortcut of ADR 0018 as `adunata serve --local`, with the same behaviour and the same guard. We call it local mode. The name states where the mode runs, which is what its guard enforces. We rejected `--debug`, because the mode also runs in release builds and because the word already names a log level.

We will keep `serve --dev` accepted without effect. It turns on nothing, applies no guard and logs no hint to `--local`. With `--dev` alone Adunata behaves like `serve`, also with a public URL on another host.

This record supersedes ADR 0018.

## Status

Accepted

## Consequences

A person or an agent starts a debug run with `serve --local` and signs in with the link from the log, as before with `--dev`.

Anyone used to `serve --dev` gets no shortcut and no hint. The README and the operating documentation name `--local`.

`--dev` is free for a meaning on test systems on other hosts. Until then it changes nothing.
