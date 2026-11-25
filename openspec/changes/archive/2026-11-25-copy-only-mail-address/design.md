## Context

The member list is the only caller of `mailRecipient`, which builds `First Last <mail>` and quotes a name with a comma or a double quote. The hook `useCopyToClipboard` writes the value and drives the confirmation and the status region.

## Goals / Non-Goals

**Goals:**
- The copy button puts exactly the mail address on the clipboard.

**Non-Goals:**
- Restricting which characters a name may hold.
- The names in mail that Adunata sends itself.

## Decisions

- **Copy `member.mail` directly.** The row passes the mail address to the existing copy hook. No helper is needed for a field access. Not architecturally significant, because it changes only a value in the browser of an organizer.
- **Delete `mailRecipient` and its unit test.** Nothing else uses them. Not architecturally significant, for the same reason.
- **Keep label, confirmation, status region and error message.** The issue decides this. Not architecturally significant.

- **Keep the scenarios "Name with a comma" and "Name with a double quote" with the bare address as outcome.** The issue drops them, but `openspec validate` rejects a MODIFIED requirement that omits a current scenario. Kept with the new outcome, they guard that no name leaks into the clipboard. Not architecturally significant.

## arc42

No section of `docs/arc42/` describes the copied value, so none changes.

## Risks / Trade-offs

- A pasted recipient no longer shows the member's name in the mail client. The issue accepts this.
