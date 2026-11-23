## 1. Copy the mail address

- [x] 1.1 In `client/src/routes/_organizer/members/index.tsx`, copy `member.mail` instead of `mailRecipient(member)` and drop the import
- [x] 1.2 Delete `client/src/lib/mailRecipient.ts` and `client/src/lib/mailRecipient.test.ts`

## 2. Tests

- [x] 2.1 In `client/src/routes/_organizer/members/members.integration.test.tsx`, expect `ada@example.org` for the copy and make both cases of names with a comma and with a double quote expect `ada@example.org`
