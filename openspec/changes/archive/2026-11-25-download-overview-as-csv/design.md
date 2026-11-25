# Design

## Context

See proposal.md for the motivation. `build_overview` in `server/src/responses.rs` already builds one `OverviewRow` per invitation behind the extractor `Organizer`, in the order of `invitations::of_gathering` (last name, first name, id). `localization::localizer()` formats catalog messages with English as fallback. The `Organizer` extractor reads the session from a cookie, so a plain link sends it. The client stores the interface language on the device, so the server learns it only from the request.

## Goals / Non-Goals

**Goals:**
- One endpoint that renders the overview as CSV, reusing `build_overview`.
- A link with a visible notice on the overview page.

**Non-Goals:**
- Everything the issue lists as out of scope: cars and seats, notes of organizers, counts of full and partial attendance, a file that follows a filter or a line-up (#93), planning rooms or rides.
- Totals, mail state, link state and who changed a response last.

## Decisions

Decisions of the issue, kept as they are:

1. The server builds the file in an endpoint of the module `responses`, because columns and escaping are rules of the domain and need one home. The client holds only the link.
2. One row per invitation and one plain value per cell. Beds needed and beds offered are separate number columns, and arrival and departure are separate columns.
3. No row of totals, because every total follows from the rows.
4. The file always holds every invitation, whatever the status filter shows.
5. RFC 4180 with a comma as separator, UTF-8 with a byte order mark.
6. Column headings and status values are in the language of the organizer's interface.
7. A cell that starts with `=`, `+`, `-` or `@` gets an apostrophe in front, as OWASP recommends.
8. The link carries a notice that the file holds personal data and that Adunata does not delete it.
9. The file name holds the title and the first day of the gathering.
10. The link and its notice are shadcn components. The link is the existing `Button` with `asChild` around an anchor, so no component needs adding.

Answers to the open questions of the issue:

11. **Language in the request.** The query parameter `language` names it, for example `GET /api/gatherings/{id}/overview.csv?language=de`. A missing or unsupported value yields English, like every other missing translation. `Accept-Language` reflects the browser, not the language chosen in the client, so it cannot carry the choice.
12. **Columns.** Last name, first name, status, arrival, departure, beds needed, beds offered, comment. Who changed a response last, the state of the mail and the state of the link serve the work in Adunata, not planning in a spreadsheet, so the file leaves them out and holds less personal data. Last name comes first to match the sort order.
13. **Period not stated.** Empty cells. Filling in the first and last day would invent a time that the member never gave, and the requirement "Period of presence" already says that an empty period means the whole gathering. Times use `YYYY-MM-DD HH:MM` in every language, because spreadsheets of every locale parse this form, unlike the localized dates of the page.
14. **Tab and carriage return.** They get the apostrophe too, as OWASP lists them. The apostrophe comes first and RFC 4180 quoting after it, so a quoted cell still starts with the apostrophe inside the quotes.
15. **File name.** Control characters and `/ \ : * ? " < > |` become `_`, the set that Windows rejects and a superset of what Unix rejects. `Content-Disposition: attachment` carries an ASCII `filename` where every non-ASCII character also becomes `_`, plus `filename*=UTF-8''` with the percent-encoded full name (RFC 6266), so no title can break the header.
16. **Notice.** "The file holds personal data. Adunata does not delete it, so delete it once you no longer need it." German: "Die Datei enthält personenbezogene Daten. Adunata löscht sie nicht, also lösche sie, sobald du sie nicht mehr brauchst." It is plain text next to the link, so it needs no hover and reaches screen readers through `aria-describedby`.

Further calls:

17. **No CSV dependency.** Quoting a cell and joining rows takes a few lines in `responses.rs`. A crate would add a dependency for less than it costs to justify.
18. **Plain link, not fetch.** The browser downloads the file through the anchor, sends the session cookie and saves it under the name of the header. The client needs no download code. The route is still declared with `utoipa`, so the generated client stays complete, but the component builds the address itself from the gathering id and the locale.

Significance: the endpoint is a new interface of a running instance, yet it follows the existing pattern of a route in a module behind the organizer session, so none of these decisions changes the structure, a quality attribute or a dependency of Adunata. No decision is architecturally significant and the change drafts no record.

## Affected arc42 sections

- `03-context-and-scope.md`: the outputs to organizers name the CSV file of the overview.
- `05-building-block-view.md`: the row of the module `responses` names the CSV route.
- `08-cross-cutting-concepts.md`: the section "Retention of personal data" states that a downloaded file lies outside the retention periods and that the page says so at the link.

## Risks / Trade-offs

- [A downloaded file keeps personal data beyond retention] → The notice at the link tells organizers to delete it.
- [The apostrophe shows in the spreadsheet for a value such as `-Smith`] → Accepted, as OWASP recommends, since a formula is worse.
- [`Cache-Control: no-store` covers the file] → Wanted, so no cache keeps it.
