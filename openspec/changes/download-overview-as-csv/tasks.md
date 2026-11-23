# Tasks

## 1. Server

- [x] 1.1 Add the catalog keys for the column headings to `server/locales/en.ftl` and `server/locales/de.ftl`, with the texts of the requirement "Language of the CSV file", and reuse `status-*` for the statuses. Verify both catalogs parse in the existing tests of `localization.rs`.
- [x] 1.2 Write a function in `server/src/responses.rs` that renders the rows of `build_overview` as CSV in a given language, with the columns, time format, quoting and apostrophe guard of the delta spec. Verify with unit tests that cover each scenario of "Columns of the CSV file", "Format of the CSV file" and "Guard against CSV injection".
- [x] 1.3 Write a function that derives the file name and the `Content-Disposition` value from title and first day. Verify with a unit test for the scenario "Title with umlaut, slash and quote".
- [x] 1.4 Add the route `GET /api/gatherings/{id}/overview.csv` with the query parameter `language` behind `Organizer`, declared with `utoipa` and registered in `router()`, answering `text/csv; charset=utf-8`. Verify with integration tests next to `overview_of` for every invitation regardless of status, the empty gathering, 401, 404, the German and English headings and the fallback for `fr`.

## 2. Client

- [x] 2.1 Regenerate `client/src/api/generated/` with `pnpm run generate` and verify `pnpm run check:generated` passes.
- [x] 2.2 Add the catalog keys for the link and its notice in both languages with the texts of the requirement "Download link on the overview".
- [x] 2.3 Add a component `DownloadOverview.tsx` in `client/src/routes/_organizer/gatherings/$gatheringId/-components/` that renders `Button` with `asChild` around an anchor to `/api/gatherings/{id}/overview.csv?language=<locale>` with the notice next to it, linked through `aria-describedby`, and place it in `Overview.tsx`. Verify in `overview.integration.test.tsx` that the link and notice show in English, that the address names `de` in German and that both show for a cancelled gathering.

## 3. Documentation

- [x] 3.1 Update `docs/arc42/03-context-and-scope.md`, `05-building-block-view.md` and the section "Retention of personal data" of `08-cross-cutting-concepts.md` as design.md lists, and verify every relative link resolves.
