# ADR 0001: Record architecture decisions

## Context

Adunata plans each change in OpenSpec. A change's `design.md` explains the decisions behind it, but the change moves to the archive once it is done. Later readers then have to search archived changes to learn why the architecture looks the way it does.

Some decisions outlive the change that made them, such as the choice of language, database or deployment. Revisiting such a decision without its original reasoning risks repeating a rejected option or breaking a constraint nobody remembers.

The architecture documentation follows arc42. Its section on architecture decisions asks for important decisions together with their rationale.

## Decision

We will record each architecturally significant decision as an architecture decision record in `docs/decisions/`, in the format Michael Nygard describes in "Documenting Architecture Decisions" [1]. A decision is architecturally significant if it affects the structure, quality attributes, dependencies, interfaces or construction techniques of Adunata.

Each record is a Markdown file that follows [the template](_decision_template.md). Its file name starts with a four-digit number, followed by the title as a short noun phrase in kebab case, for example `0002-postgresql-as-database.md`. Numbers increase by one and are never reused.

A record is never deleted. When a later decision changes or reverses an earlier one, we will write a new record and mark the earlier one as superseded with a link to the new record.

The arc42 section on architecture decisions links to these records and does not repeat them.

[1]: https://cognitect.com/blog/2011/11/15/documenting-architecture-decisions

## Status

Accepted

## Consequences

The reasoning behind durable decisions stays in one place next to the code. Contributors and coding agents can read it before they propose a change that touches the architecture.

The records keep superseded decisions, so this part of the documentation describes the past. The project's documentation rules list this as an exception.

Every significant decision costs one short document. A record whose status is not updated misleads its readers, so a change that alters a decision has to update the record it replaces.
