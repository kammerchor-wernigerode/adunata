## Context

The response page in `client/src/routes/invitation/index.tsx` loads `GET /api/invitation` (`invitation_page` in `server/src/responses.rs`). The secret travels only in the header `Authorization` (ADR 0011), and the extractor `PersonalLink` in `server/src/invitations.rs` resolves it. `link_state` reports a cancelled gathering as `Cancelled` whether it has ended or not. `gatherings` (migration `0002`) has `created_at` but no counter and no time of the last change. Only `change` and `cancel` in `server/src/gatherings.rs` write to the table after insert. The server depends on `sha2` but not on `sha1`.

## Goals / Non-Goals

**Goals:**
- One file per gathering that a calendar recognizes as the same entry on every download and replaces when it is newer.
- Code for the file that a later mail attachment can call without the HTTP layer.

**Non-Goals:**
- Everything the issue leaves out: mail attachment, subscription, the member's own period, optimistic locking, audit trail, mail on change.

## Decisions

All decisions of issue #91 hold as written: whole days, no `METHOD`/`ORGANIZER`/`ATTENDEE`/`URL`/`VALARM`/`VTIMEZONE`, the property list, `STATUS`, `TRANSP:OPAQUE`, the `UID` scheme, the columns `version` and `changed_at`, one rule that every stored change raises `version`, a server route behind the personal link, a writer without new dependency, the wording of the button and the file name. The decisions below settle what the issue left open.

### D1: Namespace of the UID
The namespace is `81aa7074-4fe3-41aa-8af7-5d34430bdd9d`, a random UUID held as a constant in the module `calendar`. The `UID` in the example of the issue is an illustration. The test value is the one in the delta spec. Significant: yes, because every calendar of a member keys its entry on the `UID`, so changing the namespace or the name form duplicates every entry. It is drafted as a record below.

### D2: SHA-1 without a dependency
A UUID of version 5 needs SHA-1. The issue forbids a new dependency, and `sha1` in `Cargo.lock` only comes through an unused backend of sqlx. The module `calendar` implements SHA-1 per FIPS 180-4 in a private function of about 40 lines, tested against the vectors of FIPS 180-4 ("abc" and the empty string) and against the UUID of the delta spec. SHA-1 serves only as a name hash here, so its weakness against collisions does not matter. Alternative: add `sha1` from RustCrypto next to `sha2`, rejected because it breaks a decision of the issue. Significant: no, a running instance cannot tell how the hash is computed.

### D3: Server decides whether the file is offered
`InvitationPage` gains `calendar: bool`. It is true when the response status is accepted or tentative and the last day has not passed (`!gathering.ended`), regardless of `cancelled`. The route uses the same predicate and answers 404 otherwise. The client shows the button only when `calendar` is true. This keeps the time zone of the instance on the server and avoids a second state field next to `LinkState`. Alternative: a new `LinkState` for "cancelled and ended", rejected because it changes how the page treats cancellation. Significant: no, it is an internal contract between server and client.

### D4: Route path and refusal
The route is `GET /api/invitation/calendar`, next to `GET /api/invitation` in `responses.rs`, and joins `PUBLIC` in the test in `server/src/api.rs` and `client/openapi.json`. A declined or pending member gets 404 like an unknown link, so the route reveals nothing that the page does not already show. Significant: no, the API is internal to the single-page client.

### D5: Module `calendar`
A new server module `calendar` holds a pure function that turns a value with title, location, description, days, cancelled, id, `created_at`, `changed_at` and `version` into the text of the file. It does no I/O, so a later mail attachment calls it directly. `gatherings` exposes the three new values through `Gathering` or a dedicated query. The route in `responses` sets `Content-Type: text/calendar; charset=utf-8` and `Content-Disposition: attachment; filename="gathering.ics"`. Significant: no.

### D6: Raising the version
`change` and `cancel` add `version = version + 1, changed_at = now()` to their `UPDATE`. A `PUT` that stores the same values still counts as a change, which follows the one rule of the issue. Significant: no beyond the issue's own decision.

### D7: Client download
The client fetches the route with the secret, turns the body into a blob URL and clicks a temporary anchor with `download="gathering.ics"`. The button is the existing shadcn `Button`. Significant: no.

### Draft record: ADR 0025 Stable calendar UIDs for gatherings

**Context.** A calendar recognizes an entry by its `UID` and replaces it when `SEQUENCE` or `DTSTAMP` is higher (RFC 5546). RFC 7986 recommends a UUID without host or domain. Adunata must give the same `UID` to every member and every download, across a move to another domain and the restore of a dump, without a column or a setting.

**Decision.** We will derive the `UID` of a gathering as a UUID of version 5 in the fixed namespace `81aa7074-4fe3-41aa-8af7-5d34430bdd9d` from the name `gathering:<id>:<created>`, with `created_at` as microseconds since the Unix epoch. We will count changes in `gatherings.version` as `SEQUENCE` and keep the time of the last change in `gatherings.changed_at` as `DTSTAMP` and `LAST-MODIFIED`.

**Consequences.** Entries survive a domain move and a restore. Changing the namespace or the name form gives every gathering a new `UID` and duplicates the entries in members' calendars, so neither may change. Two instances that reuse the same id and creation time would collide, which is practically impossible at microsecond precision. Every stored change, even one without a different value, raises the sequence. Supersedes none.

## arc42

- 05 Building block view: add the server module `calendar` and its use by `responses`.
- 08 Cross-cutting concepts: a short section on the calendar file, linking ADR 0025 for the identity of entries.
- 09 Architecture decisions: add ADR 0025 to its table.

## Risks / Trade-offs

- [Hand-written SHA-1 is wrong] → test vectors of FIPS 180-4 and the fixed UUID of the delta spec.
- [Migration on a live table] → two columns with defaults set from `created_at` in the same migration, no rewrite of other data.
- [Calendars that ignore `SEQUENCE`] → the page tells the member that a new download replaces the entry.

## Migration Plan

Migration `0003` adds `version INTEGER NOT NULL DEFAULT 0` and `changed_at TIMESTAMPTZ`, sets `changed_at = created_at`, then makes it `NOT NULL` with default `now()`. Rollback is not needed for an additive change.
