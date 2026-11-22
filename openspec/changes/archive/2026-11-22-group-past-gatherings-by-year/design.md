# Design

## Context

`list_gatherings` in `server/src/gatherings.rs` returns `upcoming` and `past`, with `past` ordered latest first. `client/src/routes/_organizer/gatherings/index.tsx` renders both with `GatheringSection` from `-components/GatheringSection.tsx`, which holds a heading of level 2, the empty message `gatherings-none` and one bordered list of rows. The shadcn component `Collapsible` is installed in `client/src/components/ui/collapsible.tsx`, and `InviteMembers.tsx` uses it with a `Button` as trigger through `asChild`. `formatSpan` in `client/src/l10n/dates.ts` parses ISO dates as local time. The client already remembers one choice on the device: `readStoredLocale` and `storeLocale` in `client/src/l10n/bundle.ts` keep the language in `localStorage` under the key `adunata.locale` and fall back to the default when storage fails.

## Goals / Non-Goals

**Goals:**
- Group the past gatherings by year in the client from the flat list the API returns today.
- Remember on the device which years the organizer opened or closed.

**Non-Goals:**
- Grouping by project, search, filters and paging.
- Any change to the upcoming gatherings, the API or the rows of a gathering.
- Remembering open years across devices or per organizer account.
- A helper that a later grouping by project could reuse. That grouping builds its own once it exists.

## Decisions

The issue fixed decisions 1 to 9. Decisions 10 to 13 answer the open questions of the issue: intake answered three of them, and the maintainer answered the question about remembering. The remaining decisions cover how to build it.

1. Adunata groups past gatherings by year, because the year is how an organizer looks for a past gathering.
2. Each year is a section that the organizer can open and close. Only the latest year with past gatherings starts open, because the latest gatherings are the ones an organizer opens most.
3. The row of a year shows the year as a heading below "Past" and the number of its gatherings, so that a closed year still says how much it holds.
4. A gathering belongs to the year of its first day, so a gathering over New Year stands in the old year.
5. Years and the gatherings inside a year are ordered with the latest first, as the list is today.
6. The date line of a gathering stays as it is, including the year.
7. The upcoming gatherings stay as they are.
8. The client groups the gatherings. The API keeps the flat list `past`, because a later second grouping can then use the same list.
9. Every control is a shadcn component. If the existing ones do not fit, the shadcn command line tool adds the missing one. No native element gets styles of its own for this.
10. **Component: one `Collapsible` per year.** It is installed and in use, and each instance opens on its own, which decision 11 needs. `Accordion` with `type="multiple"` would do the same but would add a component for no gain.
11. **Several years at once: yes.** Each year opens and closes on its own, so an organizer can compare two years.
12. **Remember open years: yes, as the maintainer decided.** When the organizer leaves the list and comes back, the years they opened or closed keep that state.
13. **Count wording.** The count is the number followed by the noun, with singular and plural in English ("1 gathering", "12 gatherings") and "Treffen" for both in German ("12 Treffen"). It is a new Fluent message `gatherings-count` with `$count` in `server/locales/en.ftl` and `server/locales/de.ftl`. Unlike `members-count`, the singular keeps the digit, as intake decided.
14. **How decisions 2 and 12 fit together.** The client remembers a choice for each year the organizer opened or closed. A year with a remembered choice follows it. A year without one starts open only if it is the latest year with past gatherings. On a first visit no year has a choice, so only the latest year is open, as decision 2 says. A year that becomes the latest later, after a gathering ends, has no choice yet and so starts open too. Remembering only the set of open years would instead leave that new year closed.
15. **Remembered in `localStorage`, not in the URL.** The organizer leaves the list through the navigation, whose link to the gathering list carries no search parameters, so state in the URL would be lost on return. `localStorage` keeps it on the device, as the language choice already does. The entry `adunata.past-years` holds a JSON object from year to `true` for open or `false` for closed. Reading and writing follow `readStoredLocale` and `storeLocale`: an unreadable or malformed entry counts as no choice, and a failed write keeps the state only until the page reloads. The entry holds no personal data, so signing out leaves it in place. Organizers who share a browser share the entry.
16. **Year from the ISO string.** The year is the first four characters of `firstDay`. Parsing the date through `Date` could shift a 1 January to the previous year in a time zone west of UTC. The year is plain text and not a Fluent variable, so Fluent cannot format it as "2,026".
17. **Order follows the API.** The client walks `past` in its order and starts a new group when the year changes. Since `past` is ordered latest first, the years and the gatherings inside them come out latest first without sorting.
18. **Markup of a year.** Each year is a `Collapsible`, keyed by the year, inside a heading of level 3. Its trigger is a shadcn `Button` through `CollapsibleTrigger asChild`, as in `InviteMembers.tsx`: `<h3><CollapsibleTrigger asChild><Button variant="ghost">…</Button></CollapsibleTrigger></h3>`. The button shows the year, the count in muted text and a chevron from `lucide-react` with `aria-hidden` that turns when the year is open. Radix sets `aria-expanded` and `aria-controls` on the trigger. The content is the same bordered list of rows as today. Tests of a closed year check `aria-expanded="false"` on its trigger and that its rows are not shown.
19. **Controlled open state.** `PastYears` reads the remembered choices once when it mounts and keeps them in React state. Each `Collapsible` gets `open` from that state, or the default of decision 14, and `onOpenChange` records the choice for its year in the state and in `localStorage`. A refetch of the list keeps the state, because it does not remount the component.
20. **Components.** The bordered list of rows moves unchanged from `GatheringSection.tsx` into its own file `-components/GatheringList.tsx`. `GatheringSection` keeps the heading of level 2 and the empty message and renders its children when the section has gatherings. A new `-components/PastYears.tsx` groups the past gatherings and renders one `Collapsible` per year around a `GatheringList`. The functions that read and write the remembered choices sit in `-components/pastYearsStorage.ts` next to it, because only this route uses them. `index.tsx` passes `GatheringList` to the section "Upcoming" and `PastYears` to the section "Past". This keeps one component per file, and a row looks the same in both sections.

### Significance

None of these decisions is architecturally significant as ADR 0001 defines it. An operator does not notice them and a running instance behaves the same at its interfaces, because they change only how one page of the client lays out data the API already returns and what that page keeps in the browser. This change drafts no record.

### arc42

No section of `docs/arc42/` changes. The building block view describes the server module `gatherings`, whose split into upcoming and past stays, and the client as one block. The section "Pages and forms" of `08-cross-cutting-concepts.md` states rules for page titles, page actions and forms, and none of them changes. The remembered language has no section of its own either, so the remembered years add none.

## Risks / Trade-offs

- [The entry grows by one key per year the organizer ever toggled] → Ten years of choices take a few hundred bytes, so the client does not prune it.
- [A remembered year has no past gatherings any more, for example after a deletion] → The client renders only years that have gatherings, so the stale key has no effect.
- [Organizers who share a browser see each other's open years] → The state is a view preference with no personal data, so this is acceptable.
