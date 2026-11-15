# ADR 0009: Optional tooling for parallel worktrees

## Context

Coding agents and people work on several branches at once, each in its own Git worktree. Every worktree binds the same ports and writes to the same development database. Git runs no hook when a worktree goes away, so resources a worktree sets up outlive it.

git-worktree-init [1] prepares new worktrees through a plugin and tears its resources down on removal or on the next prune. It is a separate project and not part of the flake, see [ADR 0005](0005-nix-flake-as-build-definition.md). Contributors without it must still be able to develop.

[1]: https://github.com/V1ncNet/git-worktree-init

## Decision

We will load the development shell through direnv and a tracked `.envrc` that sources an untracked `.envrc.local`. We will ship a plugin for git-worktree-init that gives each worktree its own ports and database. Using the tool stays optional. Without it, development works as before with the defaults of the development shell.

## Status

Accepted

## Consequences

Worktrees run side by side on one PostgreSQL server and one Mailpit. Database names depend only on the slot, so only one clone of the repository per PostgreSQL server can use the plugin.

A developer installs git-worktree-init and direnv outside the flake and opts in through local Git configuration.

The plugin follows the plugin interface of the tool. A breaking change there breaks the plugin but not the build.

Databases of worktrees removed without the tool remain until the next sweep of the tool, which `git wt prune`, `git wt add` and `git wt refresh` run.

The repository gains its first Python file.
