# Repository settings

The repository expects these settings, which no file holds:

|Setting|Where|
|-|-|
|Dependabot alerts and security updates|Settings, Advanced Security|
|Secret Protection with push protection|Settings, Advanced Security|
|Ruleset for `main` that requires the check `Verdict`, with a bypass for admins|Settings, Rules|
|Ruleset for tags `v*` that restricts creation, update and deletion, with a bypass for admins|Settings, Rules|
|GitHub App `adunata-client-dependencies` with the repository permission Contents read and write, installed on this repository only|Settings of the organization, Developer settings, GitHub Apps|
|Dependabot secrets `CLIENT_DEPENDENCIES_APP_CLIENT_ID` and `CLIENT_DEPENDENCIES_APP_PRIVATE_KEY` with the client ID and a private key of that App|Settings, Secrets and variables, Dependabot|
|Package `adunata` public, once after the first run of [`publish`](../../.github/workflows/publish.yaml) created it. This cannot be undone.|Packages of the organization, `adunata`, Package settings|
