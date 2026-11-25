# Quality requirements

The quality goals in [Introduction and goals](01-introduction-and-goals.md) name what matters. The scenarios below make them checkable. Each names the test or check that verifies it.

## Operability

|Scenario|Context|Stimulus|Response|Verified by|
|-|-|-|-|-|
|Invalid configuration|An operator starts a new instance.|Two required settings are missing and one has the wrong format.|Adunata exits with status 78 before it opens a port or touches the database. The log names all three settings in one entry each. It shows no secret value.|[`server/tests/startup.rs`](../../server/tests/startup.rs) and `several_settings_missing` in [`server/src/configuration.rs`](../../server/src/configuration.rs)|
|Mail server outage|The mail server of the ensemble is unreachable for a few minutes.|An organizer invites members during the outage.|The invitations exist at once. Their mails wait in the outbox and arrive once the server is reachable again.|`invitation_succeeds_while_the_mail_server_is_down` in [`server/src/invitations.rs`](../../server/src/invitations.rs) and `mail_server_comes_back` in [`server/src/mail.rs`](../../server/src/mail.rs)|
|Update|An operator installs a newer release over a running instance.|The service starts.|Adunata applies the pending migrations.|`start_after_an_update` in [`server/src/database.rs`](../../server/src/database.rs)|
|Downgrade|An operator starts an older release against a database that a newer release migrated.|The service starts.|Adunata exits with status 65 and changes nothing, so systemd does not restart it.|`start_after_a_downgrade` in [`server/src/database.rs`](../../server/src/database.rs)|

## Security

|Scenario|Context|Stimulus|Response|Verified by|
|-|-|-|-|-|
|Unknown address at sign-in|Someone tries to learn who organizes the ensemble.|A sign-in link is requested for an address that belongs to no active organizer.|Adunata answers exactly as for an organizer and sends no mail.|`unknown_address` in [`server/src/access.rs`](../../server/src/access.rs)|
|Request from another origin|An organizer is signed in and visits a page on another origin.|That page makes the browser send a request that changes data.|Adunata rejects it with status 403 and changes nothing.|`request_from_another_origin` in [`server/src/access.rs`](../../server/src/access.rs)|
|Guessed link|Someone tries secrets to reach invitations.|A request carries a secret that matches no invitation.|Adunata answers with status 404 and an empty body, the same as for a revoked link.|`guessed_link` in [`server/src/responses.rs`](../../server/src/responses.rs)|
|Personal link absent from logs|A member opens the response page and answers through it.|Adunata handles the requests.|No line that Adunata logs contains the secret, and no request line carries it, because it sits in the fragment.|`request_with_a_personal_link` in [`server/src/responses.rs`](../../server/src/responses.rs) and [`client/src/routes/invitation/invitation.integration.test.tsx`](../../client/src/routes/invitation/invitation.integration.test.tsx)|
|Expiring comments|The operator kept the default retention.|A gathering ended 31 days ago.|Its responses exist without their comments. After 365 days its invitations and responses are gone while the gathering stays.|`default_retention` and `responses_expire` in [`server/src/responses.rs`](../../server/src/responses.rs)|
|Organizer route without a session|A developer adds an organizer route and forgets the extractor.|The integration tests run.|The test that calls every operation without a session names the route and fails.|`every_organizer_operation_requires_a_session` in [`server/src/api.rs`](../../server/src/api.rs)|

## Accessibility

|Scenario|Context|Stimulus|Response|Verified by|
|-|-|-|-|-|
|Member form at 200 % on a phone|A member with low vision opens the personal link on a phone 390 px wide with the text enlarged to 200 %.|The member answers.|The response form and the overview scroll only vertically, axe finds no violation in light or dark mode and no control breaks the Content Security Policy.|[`client/e2e/invitation.e2e.ts`](../../client/e2e/invitation.e2e.ts)|
|Calendar by keyboard|A member uses a keyboard and a screen reader.|The member picks the day of arrival.|The calendar offers only the days of the gathering, moves with the arrow keys and names its grid and each day.|`operates the calendar by keyboard and offers only the days of the gathering` in [`client/src/routes/invitation/invitation.integration.test.tsx`](../../client/src/routes/invitation/invitation.integration.test.tsx)|

## Localization

|Scenario|Context|Stimulus|Response|Verified by|
|-|-|-|-|-|
|Plural forms|A member reads the client or a mail in German or English.|A text states a count of 0, 1 or 2.|Each count uses the grammatically correct form of the language, such as "braucht eins" and "braucht 2"|`plural_forms_in_german` and `plural_forms_in_english` in [`server/src/localization.rs`](../../server/src/localization.rs) and [`client/src/l10n/bundle.test.ts`](../../client/src/l10n/bundle.test.ts)|

## Changeability

|Scenario|Context|Stimulus|Response|Verified by|
|-|-|-|-|-|
|Change of the API|A contributor changes a route or a type of the server.|The checks run.|The contract test fails until the tracked OpenAPI document and the generated client code match the server again.|`the_client_holds_the_current_document` in [`server/src/api.rs`](../../server/src/api.rs) and `pnpm run check:generated` in `client/`|
