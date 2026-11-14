# Quality requirements

The quality goals in [Introduction and goals](01-introduction-and-goals.md) name what matters. The scenarios below make them checkable. Each names the test or check that verifies it.

## Operability

|Scenario|Context|Stimulus|Response|Verified by|
|-|-|-|-|-|
|Invalid configuration|An operator starts a new instance.|Two required settings are missing and one has the wrong format.|Adunata exits with status 1 before it opens a port or touches the database. The log names all three settings in one entry each. It shows no secret value.|[`server/tests/startup.rs`](../../server/tests/startup.rs) and `several_settings_missing` in [`server/src/configuration.rs`](../../server/src/configuration.rs)|
|Mail server outage|The mail server of the ensemble is unreachable for a few minutes.|An organizer triggers a mail during the outage.|The action succeeds. The mail waits in the outbox and arrives once the server is reachable again.|`action_succeeds_while_the_mail_server_is_down` in [`server/src/spike.rs`](../../server/src/spike.rs) and `mail_server_comes_back` in [`server/src/mail.rs`](../../server/src/mail.rs)|

## Efficiency

|Scenario|Context|Stimulus|Response|Verified by|
|-|-|-|-|-|
|Memory under load|The statically linked release binary runs on Linux next to PostgreSQL and Mailpit.|10 connections send the form and load the client for 30 seconds.|Peak resident memory of Adunata stays below 150 MB. The latest measurement was 5 MB.|The check `memory` in [`flake.nix`](../../flake.nix)|

## Localization

|Scenario|Context|Stimulus|Response|Verified by|
|-|-|-|-|-|
|Plural forms|A member reads the client or a mail in German or English.|A text states a count of 0, 1 or 2.|Each count uses the grammatically correct form of the language, such as "Ein Bett benötigt." and "2 Betten benötigt."|`plural_forms_in_german` and `plural_forms_in_english` in [`server/src/localization.rs`](../../server/src/localization.rs) and [`client/src/l10n/bundle.test.ts`](../../client/src/l10n/bundle.test.ts)|
