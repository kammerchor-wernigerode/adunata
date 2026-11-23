## Context

`GatheringForm` in `client/src/routes/_organizer/gatherings/-components/GatheringForm.tsx` creates a gathering or, given the prop `gathering`, changes one. Its `save` calls `createGathering` or `changeGathering`, invalidates the queries of the list and the gathering and then navigates to `/gatherings/$gatheringId`. It has one submit button `gathering-save` and a button `member-cancel`. A refusal shows in a paragraph with `role="alert"`. `POST /api/gatherings` already takes every field. `formatSpan` in `client/src/l10n/dates.ts` formats a span as the list and the page of a gathering show it. The section Pages and forms of `docs/arc42/08-cross-cutting-concepts.md` says that a successful save is announced through an element with `role="status"`.

## Goals / Non-Goals

**Goals:**
- Create several gatherings in a row without leaving the form.

**Non-Goals:**
- Copying an earlier gathering, including a cancelled one, or any action "Copy" on the page of a gathering or in the list.
- Suggesting days.
- A second button in the form for a new member.
- Any change to the server apart from the new labels.

## Decisions

The decisions of the issue stand as written:

- A second button next to "Save", labeled "Speichern & Weiter" in German and "Save & add another" in English, only in the form for a new gathering.
- It stores the gathering and keeps the form open. Title, location and description stay. First day and last day become empty.
- "Save" stays the first button and the one Enter triggers. It opens the page of the gathering as today.
- After a save with the second button, a line names the title and the span of the stored gathering and links to its page. Screen readers announce it. It stays until the next save or refusal replaces it.
- After a save with the second button, focus moves to the title.
- Both buttons are disabled while a save runs.
- A refused save behaves as today with either button.
- The second button is the shadcn `Button` and the line is the shadcn `alert`, added with the command line tool of shadcn.
- No action "Copy".

Further decisions:

- **Second button with its own submit handler.** The second button is `type="button"` and calls `form.handleSubmit` with a handler that stays on the form. So Enter keeps triggering "Save", the first submit button, and validation runs the same way for both. Alternative: `type="submit"` with a `name`, rejected because the form would then have two submit buttons and reading the submitter in react-hook-form needs extra code.
- **The line uses `role="status"`.** The shadcn `alert` renders `role="alert"` by default. The line overrides it with `role="status"`, because the arc42 pattern announces a successful save through `role="status"` and an alert would interrupt the screen reader. The existing refusal paragraph stays as it is.
- **Span through `formatSpan`.** The line reads like the row of the list, for example "Probenwochenende, March 12–14, 2027 saved.", with the title as link to the page. The label takes the title and the span as Fluent variables.
- **Reset with `form.reset`.** After the save, the form resets to the stored title, location and description and empty days. This also clears any marks of a field from an earlier refusal.
- **A refusal clears the line.** A refusal or an unexpected error with either button removes the line, so only the latest result shows.
- **Queries are invalidated as today**, so the list shows the stored gathering once the organizer returns to it.

No decision is architecturally significant, because an operator or a running instance notices none of them. They change one form of the client. The change drafts no record.

## Affected arc42 sections

None. Section 8, Pages and forms, already describes the announcement of a successful save through `role="status"`, which this change follows.

## Risks / Trade-offs

- [The form gets two paths after a successful create] → The tests under `gathering form` cover both buttons, the reset, the focus, the line and a refusal through the second button.
