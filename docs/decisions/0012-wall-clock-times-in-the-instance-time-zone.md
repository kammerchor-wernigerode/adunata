# ADR 0012: Wall-clock times in the instance time zone

## Context

A gathering spans whole days. Members state when they arrive and leave, such as 14 March 2027 at 18:00. Organizers read these times in an overview and members read them in a form and in mail. Everyone means the local time at the place of the gathering, whatever time zone their browser or the server uses.

Adunata also compares with the present, for example whether a gathering has begun or ended, and shows instants such as the time of the last change to a response. Both need a time zone. Ensembles live in different time zones, so no default fits most installations. A wrong default would shift silently when links close and gatherings begin.

The crate `time`, which the server uses with `sqlx` and `utoipa`, has no time zone database. Minimal container images carry no `/usr/share/zoneinfo`. Configuration fails before Adunata connects to the database, so the database cannot check the setting.

## Decision

We will store the days of a gathering as `date` and a period of presence as `timestamp without time zone`. The API exchanges them as ISO 8601 strings without an offset, and client, server and mail show them unchanged.

We will require the setting `ADUNATA_TIME_ZONE` as an IANA name and reject an unknown name at start-up. One function in the module `configuration` turns an instant into the date and wall-clock time of the instance. Every comparison with the present and every instant that goes into the API or a mail passes through it. Instants stay `timestamptz` in the database.

We will resolve time zones with the crate `jiff` and its feature `tzdb-bundle-always`, which embeds the time zone database in the binary. `jiff` stays inside the conversion function.

## Status

Accepted

## Consequences

Form, overview and mail show the same times in any browser. The client never converts a time.

Every installation sets one more required setting. Adunata names it with all other missing settings in one round.

A wall-clock time inside the gap of a daylight saving change stays a valid value, because it is stored without an instant.

The binary carries the time zone database and gains a dependency. An update of the database needs a new release of Adunata.

Every module that compares with the present calls the conversion function instead of reading the clock in its own zone.
