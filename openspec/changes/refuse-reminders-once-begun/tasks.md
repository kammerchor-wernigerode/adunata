# Tasks

## 1. Server

- [x] 1.1 Add the field `begun` to `Gathering` in `server/src/gatherings.rs`, computed in `from_row` from the first day and today, and replace the method `has_begun` with it, including its unit test
- [x] 1.2 Read `begun` in `link_state` of `server/src/responses.rs`
- [x] 1.3 Refuse in `remind_pending` of `server/src/invitations.rs` with `ErrorCode::Begun` once the gathering has begun, after `open_gathering`, and name the refusal in the OpenAPI description of the route `remind`
- [x] 1.4 Add tests in `server/src/invitations.rs`: a reminder on the first day answers 422 with code `begun` and changes no link and queues no mail, a reminder on the day before still queues, and invite and resend still succeed on the first day

## 2. Client

- [x] 2.1 Regenerate `client/src/api/generated/` from the OpenAPI document
- [x] 2.2 Show the remind button in `Overview.tsx` only while the gathering is neither cancelled nor begun, and keep the resend action on its current rule
- [x] 2.3 Add `begun` to the fixtures of the client tests and a test in `overview.integration.test.tsx` that a gathering with `begun: true` shows no remind button
