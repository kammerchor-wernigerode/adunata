## Why

The copy button of the member list copies a member as `First Last <mail>`. Some characters in a name break that form in a mail client, and quoting the name correctly needs the rules of RFC 5322. Copying only the mail address removes the name and with it every quoting problem.

## What Changes

- The button "Copy mail address" in each row of the member list copies only the mail address, such as `ada@example.org`, without name and without angle brackets.
- The quoting of names with a comma or a double quote goes away.
- Label, confirmation, status region and error message of the button stay as they are.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `member-roster`: the requirement "Member list" describes the copied value as the bare mail address and drops the rule for quoting names.

## Impact

- `client/src/routes/_organizer/members/index.tsx` copies `member.mail`.
- `client/src/lib/mailRecipient.ts` and `client/src/lib/mailRecipient.test.ts` are deleted, because the member list is their only user.
- `client/src/routes/_organizer/members/members.integration.test.tsx` expects the bare address.
