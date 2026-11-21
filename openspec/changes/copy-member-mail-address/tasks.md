# Tasks

## 1. Recipient format

- [ ] 1.1 Add `mailRecipient` in `client/src/lib/mailRecipient.ts` that returns `First Last <mail>` and quotes and escapes the name only when it contains a comma or a double quote
- [ ] 1.2 Add `client/src/lib/mailRecipient.test.ts` covering a plain name, a name with a comma, a name with a double quote and a name with a backslash next to a comma

## 2. Copy hook

- [ ] 2.1 Add `useCopyToClipboard` in `client/src/hooks/useCopyToClipboard.ts` with `copy(key, text)` that reports success or failure, resets success after 2000 ms and clears its timer on unmount

## 3. Member list

- [ ] 3.1 Add the keys `members-copy-mail`, `members-mail-copied` and `members-copy-failed` with the texts from design.md to `server/locales/en.ftl` and `server/locales/de.ftl`
- [ ] 3.2 In `client/src/routes/_organizer/members/index.tsx`, add a ghost icon `Button` with `Copy` or `Check` from `lucide-react` next to the mail address in every row, labeled from the catalog
- [ ] 3.3 Add a second `role="status"` region to the page for the copy result, separate from the member count
- [ ] 3.4 Make the existing test in `members.integration.test.tsx` that reads the member count select the count region explicitly

## 4. Tests

- [ ] 4.1 In `members.integration.test.tsx`, cover each scenario of the delta spec: copy and clipboard text, the label and region after success, the reset after 2000 ms, the inactive member's button, failure with a rejected write and without `navigator.clipboard`, and the unchanged member count
- [ ] 4.2 Keep the accessibility check of `violations` free of findings with the new button and region
