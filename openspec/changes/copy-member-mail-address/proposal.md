# Proposal

## Why

Organizers who want to write to a member retype the mail address or select it by hand in the member table. One click should put the member as a recipient on the clipboard, ready to paste into a mail client.

## What Changes

- Each row of the member list gets an icon button next to the mail address that copies `First Last <mail>` to the clipboard, for active and inactive members.
- The name is quoted as RFC 5322 describes when it contains a comma or a double quote, so the pasted text stays one recipient.
- The icon turns into a check mark for two seconds after a successful copy. A live region on the page announces success or failure to screen readers, apart from the member count.
- The English and German catalogs get the label, the confirmation and the failure text.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `member-roster`: the requirement "Member list" gains copying a member's mail address as a recipient.

## Impact

- `client/src/routes/_organizer/members/index.tsx` renders the button and the live region.
- A new pure function in `client/src/lib/` and a new hook in `client/src/hooks/`.
- `server/locales/en.ftl` and `server/locales/de.ftl` get three texts each.
- No server API, no new dependency, no change for operators.
