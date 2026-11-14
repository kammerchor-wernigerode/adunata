# Introduction and goals

Adunata is a web application for choirs and instrumental ensembles. Organizers invite members to gatherings, such as a rehearsal weekend, a rehearsal camp during the week or a concert. Members reply whether and when they attend and whether they need or offer a bed. Ensembles install Adunata on their own servers.

## Requirements overview

The requirements live as capabilities in [`openspec/specs/`](../../openspec/specs/). This document does not repeat them.

|Capability|What it covers|
|-|-|
|[instance-configuration](../../openspec/specs/instance-configuration/spec.md)|How operators configure an instance and how Adunata checks the configuration|
|[localization](../../openspec/specs/localization/spec.md)|Languages of the client and of mail, catalogs and date formats|
|[mail-delivery](../../openspec/specs/mail-delivery/spec.md)|Sending mail through the outbox with encryption and retries|
|[service-operation](../../openspec/specs/service-operation/spec.md)|Installing, starting, migrating, monitoring and stopping an instance|

## Quality goals

The goals are ordered by priority.

|Priority|Goal|What it means for Adunata|
|-|-|-|
|1|Operability|An operator without operations experience installs, configures, updates and monitors an instance by following the documentation.|
|2|Data protection and security|Adunata stores only the personal data that gatherings need, shows each person only what their role allows and keeps secrets and invitation links out of logs and storage.|
|3|Accessibility|Members and organizers use every function with a keyboard, a screen reader or enlarged text, also on a phone.|
|4|Localization|Every text in the client and in mail appears in the reader's language, starting with German and English.|
|5|Efficiency|One instance runs next to other services on a small shared server.|

## Stakeholders

|Role|Expectations of this documentation|
|-|-|
|Organizer|None directly. Organizers are represented through the quality goals and the capabilities.|
|Member|None directly. Members are represented through the quality goals and the capabilities.|
|Operator|Knows what Adunata needs to run and where to find the operating documentation.|
|Maintainer in the architect role|Keeps the structure, the decisions and the risks in view and judges changes against them.|
|Coding agent|Finds where a fact belongs and which constraints, decisions and concepts apply before writing code.|
|Contributor|Understands the structure well enough to propose a change in the right place.|
