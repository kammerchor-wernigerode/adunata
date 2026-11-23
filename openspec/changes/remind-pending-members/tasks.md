## 1. Server

- [x] 1.1 Extract the rotation of a secret plus enqueue from `resend_invitation` in `server/src/invitations.rs` into a function that the resend and the reminder share
- [x] 1.2 Let the invitation mail builder take the message ids of subject and details, and add `reminder-mail-subject` and `reminder-mail-details` to `server/locales/en.ftl` and `server/locales/de.ftl`
- [x] 1.3 Add `POST /api/gatherings/{id}/reminders` for organizers, with `open_gathering`, the recipient query of the design and a 201 response with the count
- [x] 1.4 Add tests for every scenario of `specs/invitations/spec.md`: recipients, revoked links, failed mail, zero count, closed gathering, old link, second reminder, subjects and content
- [x] 1.5 Regenerate the client API from the OpenAPI document

## 2. Client

- [x] 2.1 Add the shadcn component `dialog` with the shadcn command line tool
- [x] 2.2 Add the remind button and its dialog next to the status filter in a component of its own beside `Overview.tsx`, shown only for an open gathering with at least one pending row with an active link
- [x] 2.3 Add the page texts in English and German to `server/locales/*.ftl`
- [x] 2.4 Show the result message and reload the overview after success, keep the dialog open with the failure otherwise
- [x] 2.5 Add integration tests for every scenario of `specs/responses/spec.md`

## 3. Documentation

- [x] 3.1 Update the row of `invitations` in `docs/arc42/05-building-block-view.md`
- [x] 3.2 Update the paragraph on personal links in `docs/arc42/08-cross-cutting-concepts.md`
