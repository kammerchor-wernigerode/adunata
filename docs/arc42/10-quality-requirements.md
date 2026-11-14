# Quality requirements

The quality goals in [Introduction and goals](01-introduction-and-goals.md) name what matters. The scenarios below make them checkable. Each names the test or check that verifies it.

## Operability

|Scenario|Context|Stimulus|Response|Verified by|
|-|-|-|-|-|
|Invalid configuration|An operator starts a new instance.|Two required settings are missing and one has the wrong format.|Adunata exits with status 1 before it opens a port or touches the database. One message lists all three settings and no secret value.|`several_settings_missing` and `malformed_value` in [`server/src/configuration.rs`](../../server/src/configuration.rs)|
|Mail server outage|The mail server of the ensemble is unreachable for a few minutes.|An organizer triggers a mail during the outage.|The action succeeds. The mail waits in the outbox and arrives once the server is reachable again.|`action_succeeds_while_the_mail_server_is_down` in [`server/src/spike.rs`](../../server/src/spike.rs) and `mail_server_comes_back` in [`server/src/mail.rs`](../../server/src/mail.rs)|
