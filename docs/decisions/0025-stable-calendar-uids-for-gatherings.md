# ADR 0025: Stable calendar UIDs for gatherings

## Context

A calendar recognizes an entry by its `UID` and replaces it when `SEQUENCE` or `DTSTAMP` is higher, as RFC 5546 describes. RFC 7986 recommends a UUID without host or domain. Adunata must give the same `UID` to every member and every download, across a move to another domain and the restore of a dump, without a column or a setting.

## Decision

We will derive the `UID` of a gathering as a UUID of version 5 in the fixed namespace `81aa7074-4fe3-41aa-8af7-5d34430bdd9d` from the name `gathering:<id>:<created>`, with `created_at` as microseconds since the Unix epoch. We will count changes in `gatherings.version` as `SEQUENCE` and keep the time of the last change in `gatherings.changed_at` as `DTSTAMP` and `LAST-MODIFIED`.

## Status

Accepted

## Consequences

Entries survive a domain move and a restore. Changing the namespace or the name form gives every gathering a new `UID` and duplicates the entries in members' calendars, so neither may change. Two instances that reuse the same id and creation time would collide, which is practically impossible at microsecond precision. Every stored change, even one without a different value, raises the sequence.
