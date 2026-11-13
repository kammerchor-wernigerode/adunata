# Architecture constraints

## Technical constraints

|Constraint|Explanation|
|-|-|
|Self-hosting|Each ensemble runs its own instance on its own server. There is no central service.|
|Small shared hardware|The reference installation shares a virtual machine with 2 vCPUs and 4 GB of RAM with an identity provider, a collaborative editor and databases.|
|NixOS at the reference installation|The server of the reference installation runs NixOS, so Adunata has to run as a NixOS service.|
|Low concurrency|At most about 10 people use one instance at the same time.|

## Organizational constraints

|Constraint|Explanation|
|-|-|
|License|Adunata is open source under the AGPL-3.0, see [`LICENSE`](../../LICENSE).|
|Team|One maintainer acts as architect. Coding agents write and review almost all code.|
|Hosting of the project|The repository and its CI run on GitHub.|

## Conventions

|Convention|Where it is defined|
|-|-|
|Rules for agents, code style, language and documentation|[`AGENTS.md`](../../AGENTS.md)|
|Planning of changes with OpenSpec|[`openspec/config.yaml`](../../openspec/config.yaml)|
|Architecture decision records|[ADR 0001](../decisions/0001-record-architecture-decisions.md)|
|Formatting of every file|[`.editorconfig`](../../.editorconfig)|
