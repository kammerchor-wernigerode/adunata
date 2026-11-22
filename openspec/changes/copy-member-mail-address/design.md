# Design

## Context

The member list lives in `client/src/routes/_organizer/members/index.tsx`. Its mail cell shows `member.mail` as plain text. Above the table a `<p role="status">` announces the member count, and `members.integration.test.tsx` reads it with `getByRole('status')`. The client has no clipboard code and no toast component. The shadcn `Button` in `client/src/components/ui/button.tsx` has the variant `ghost` and the size `icon`, and `lucide-react` is installed. Texts come from `server/locales/en.ftl` and `server/locales/de.ftl` through `useTranslate`.

## Goals / Non-Goals

**Goals:**

- Copy one member as a mail recipient with one click from the member list.
- Confirm the copy where the user looks and announce it to screen readers.
- Show a failed copy instead of failing silently.

**Non-Goals:**

- Copying the addresses of several members at once.
- Copying from any page other than the member list.
- Toasts and any new dependency.
- Quoting for other characters that RFC 5322 treats as special in a name, such as `<`, `>`, `:`, `;`, `@` and parentheses.
- A decision record and changes to `docs/arc42/`.

## Decisions

Each decision below is not architecturally significant, because neither an operator nor a running instance notices a change confined to one page of the browser client. The change therefore drafts no decision record and affects no section of `docs/arc42/`.

### Recipient format

The copied text is `[first name] [last name] <mail address>`, because mail clients recognize that form as a recipient. The order never changes. The name goes in double quotes, with `"` and `\` escaped by a preceding `\`, only when it contains a comma or a double quote, because only those characters break the recipient. Other names stay unquoted. This decision comes from the issue.

The formatting is the pure function `mailRecipient(member)` in `client/src/lib/mailRecipient.ts`, with a unit test in `client/src/lib/mailRecipient.test.ts`. The file is named after its single export like `dataOf.ts` and `succeeded.ts`. It takes a `Member` of the generated API and returns a string.

### Button

An icon button next to the address copies it. It is a shadcn `Button` with the variant `ghost` and the size `icon`, and it shows the icon `Copy` of `lucide-react`. After a successful copy the icon changes to `Check` for two seconds, because the change sits where the user looks. The issue says about two seconds. The design fixes exactly 2000 ms so that a test with fake timers can check it. The button has the accessible name "Copy mail address" and, after success, "Mail address copied". No toast and no new dependency confirm the copy, because the icon change does. These decisions come from the issue.

The button appears for active and inactive members. The issue left this open with yes as the default, and an organizer may still want to write to an inactive member.

### Hook

The hook `useCopyToClipboard` in `client/src/hooks/useCopyToClipboard.ts` returns `{ copiedKey, failed, copy }`, where `copy(key, text)` writes the text with `navigator.clipboard.writeText`. `copiedKey` holds the key of the last successful copy, `failed` tells whether the last attempt failed, and the hook resets `copiedKey` after 2000 ms. It clears its timer when the page unmounts. A second click restarts the timer. The page uses one instance of the hook, keyed by member id, so only one row shows `Check` at a time and one live region suffices. A missing `navigator.clipboard` counts as a failure like a rejected write, because outside a secure context the property is absent.

### Live region

The page renders one second element with `role="status"`, always present and empty while idle, separate from the member count paragraph. It shows "Mail address copied" after success and the failure text after a failure. The success text clears together with the icon after 2000 ms. The failure text stays until the next copy attempt, so a reader has time to read it. Keeping the count in its own region means a copy never replaces the count announcement, as intake required. The existing test that reads `getByRole('status')` must then pick the count region explicitly.

### Texts

The catalogs get three keys in `server/locales/en.ftl` and `server/locales/de.ftl`, next to the other `members-` keys:

|Key|English|German|
|-|-|-|
|`members-copy-mail`|Copy mail address|Mailadresse kopieren|
|`members-mail-copied`|Mail address copied|Mailadresse kopiert|
|`members-copy-failed`|Could not copy the mail address. Select and copy it by hand.|Die Mailadresse ließ sich nicht kopieren. Markiere und kopiere sie von Hand.|

The German texts follow the existing wording "Mailadresse".

## Risks / Trade-offs

- Every row's button has the same accessible name. The issue fixed the label, and the button sits in the row next to the address it copies, so the row gives the context.
- A name with `<`, `>`, `@` or similar characters is copied unquoted and may confuse a mail client. The issue limits quoting to a comma and a double quote, and such names are unlikely in a choir roster.
