---
name: arc42
description: Write and maintain Adunata's architecture documentation in docs/arc42/ following the arc42 template. Use when creating or editing any file in docs/arc42/, when an OpenSpec change design or task touches architecture or when deciding which arc42 section a fact belongs in.
---

# arc42

Adunata documents its architecture with the arc42 template [1] in `docs/arc42/`. This skill says where each fact belongs, how files look and when a change has to update them.

[1]: https://docs.arc42.org

## Where facts live

arc42 holds how Adunata is built and why. Three other places own facts that arc42 only links to:

|Fact|Home|arc42 does|
|-|-|-|
|Behavior and requirements|`openspec/specs/`|links from section 1|
|Architecture decisions with rationale|`docs/decisions/`|links from sections 4 and 9|
|Work in flight|`openspec/changes/`|nothing, a change updates arc42 when it lands|
|Steps for operators and every setting|`docs/operating/`|links from sections 3, 7 and 8|
|Commands, tools and settings for development|`CONTRIBUTING.md` and `README.md`|links from section 8|
|Workflows of CI, releases and dependency updates|`docs/maintenance/`|links from section 8|

arc42 describes the current architecture and, where planned, the target architecture. It never describes how it got there. Git and the decision records hold that.

## What belongs in arc42

arc42 holds concepts, never instructions. A concept says how Adunata is built and which rule its parts follow. An instruction says which command to run, which job does what or which value a setting has.

The template asks for restraint in its own words. Section 5 describes the system "on an abstract level without disclosing implementation details". Section 8 says "pick only the most-needed topics". Sections 5 and 6 say "prefer relevance over completeness", and section 9 says "avoid redundancy".

Ask three questions of every sentence before you write it:

1. Would the sentence still be true after a refactoring that keeps the structure? A sentence that names a job of CI, a runner, a command, a color value or the order of entries in a menu fails this question.
2. Does another place already own the fact? Then link to it.
3. Does an architect, an operator or a contributor need the fact to understand the structure? A fact that only helps to carry out a task belongs in the guide for that task.

|Detail|Home|
|-|-|
|A command, a tool or its version|`CONTRIBUTING.md`|
|A job, a runner or a workflow of CI|`docs/maintenance/`|
|A setting, its default or its allowed values|`docs/operating/configuration.md`|
|A limit, a period or a text that users see|`openspec/specs/`|
|A color, a size or the layout of one page|the code or `openspec/specs/`|

## Files

`docs/arc42/README.md` is the entry point. It names the document and lists every existing section file as a link, in order.

Each section is one Markdown file. A section file exists only once it has content, so no file holds template placeholders and every link resolves.

|File|Heading|
|-|-|
|`01-introduction-and-goals.md`|Introduction and goals|
|`02-architecture-constraints.md`|Architecture constraints|
|`03-context-and-scope.md`|Context and scope|
|`04-solution-strategy.md`|Solution strategy|
|`05-building-block-view.md`|Building block view|
|`06-runtime-view.md`|Runtime view|
|`07-deployment-view.md`|Deployment view|
|`08-cross-cutting-concepts.md`|Cross-cutting concepts|
|`09-architecture-decisions.md`|Architecture decisions|
|`10-quality-requirements.md`|Quality requirements|
|`11-risks-and-technical-debt.md`|Risks and technical debt|
|`12-glossary.md`|Glossary|

Headings use sentence case and carry no section numbers, because the file name already carries the number. Each file starts with one `#` heading. Subsections use `##` and deeper.

Write only content. Do not copy the template's help text into a file. Leave out a subsection that has nothing to say yet.

## Sections

Each entry says what the section holds and what it does not.

**1 Introduction and goals.** It holds a short requirements overview that links to the capabilities in `openspec/specs/`, the top three to five quality goals in priority order and a stakeholder table with roles and what each role needs from this documentation. It does not repeat requirements from the specs. Quality goals are goals for the architecture, not project goals. Stakeholders are roles, never named people.

**2 Architecture constraints.** It holds conditions that were given and not chosen, such as licensing, self-hosting, hardware limits, the operating system of known installations or the size of the team. Group them as technical, organizational and conventions if the list grows. A constraint has no alternatives to weigh. If Adunata could have chosen otherwise, it is a decision and belongs in `docs/decisions/`.

**3 Context and scope.** It shows Adunata as a black box with every communication partner, such as users in their roles, the mail server or an identity provider. The business context names what each partner sends and receives. The technical context names channels and protocols. It does not describe Adunata's insides or its hosting.

**4 Solution strategy.** It holds a short summary of the fundamental decisions, such as technology, top-level decomposition and how the architecture reaches each quality goal. One or two sentences per item, each linking to its decision record and to the section with the details. It never holds the full reasoning, which lives in the decision record.

**5 Building block view.** It holds the static decomposition. The view is a hierarchy. Level 1 is the white box of the whole system with an overview diagram, the motivation for the decomposition and a black box description of each contained building block. It holds the few blocks that someone names first when asked what Adunata consists of, which are the client and the server. A module of the server or a folder of the client never appears on level 1. Level 2 opens one block of level 1 as a white box. The white box of the server lists every module that `server/src/main.rs` declares, grouped by kind. Level 3 opens a module only when it is important, surprising, risky or complex. A black box description is one sentence for the responsibility and one for the interface. It names no feature, no route and no rule of the domain. A diagram of this section shows only what a block contains and what it depends on. A box inside a box is a part, and an arrow without a label means that one block depends on the public interface of the other. It shows no data flow, no protocol and no communication partner, which belong in sections 6, 7 and 3. It does not describe behavior over time, which belongs in section 6, or concerns that span many blocks, which belong in section 8.

**6 Runtime view.** It holds a few architecturally relevant scenarios that show how building blocks work together, such as a key use case, an interaction at an external interface, start-up, a background task or an error case. Pick a representative few, not every use case. It does not restate the scenarios in the specs. It shows which blocks carry them out. Rules of the domain, such as the states of a page or the allowed values of a field, stay in the specs.

**7 Deployment view.** It holds the infrastructure that runs Adunata, meaning nodes, channels and which artifact runs where. Describe each supported way of running Adunata. Operating instructions for administrators belong in user-facing documentation, which this section links to.

**8 Cross-cutting concepts.** It holds rules and solutions that apply across many building blocks, such as the data model, authentication, authorization, localization, accessibility, personal data, error handling, persistence or testing. Give each concept its own `##` heading and cover only the concepts Adunata needs. A concept states the rule and names the block that enforces it. It leaves limits, defaults and texts to the place that owns them. The testing concept names the levels of tests and what each level proves. It names no job, no runner and no command. A concept that applies to a single building block belongs with that block in section 5.

**9 Architecture decisions.** It holds a table of all records in `docs/decisions/` with number, title as a link and status, ordered by number. The template file is not a record and stays out of the table. It holds no reasoning of its own.

**10 Quality requirements.** It refines the quality goals from section 1 into concrete, testable scenarios. Each scenario names its context, the stimulus and a measurable response, for example "A member opens the form on a phone with a screen reader. Every field has a label that the screen reader announces." Section 1 names the goal. This section makes it checkable.

**11 Risks and technical debt.** It holds known technical risks and debt in priority order, each with its planned mitigation. Remove an entry once it is resolved. Open questions of a change in flight belong in that change's proposal, not here.

**12 Glossary.** It holds the domain and technical terms people use when they talk about Adunata, as a table with term and definition. Specs and other documents use these terms exactly as defined here.

## Common confusions

|If the fact is|It belongs in|Not in|
|-|-|-|
|a given condition|2|`docs/decisions/`|
|a chosen option with reasoning|`docs/decisions/`, summarized in 4|2 or 9|
|who talks to Adunata|3|7|
|where Adunata runs|7|3|
|a part of Adunata and its job|5|8|
|what two blocks exchange and in which order|6|5|
|a protocol or a channel|3 or 7|5|
|a module of the server|5, level 2|5, level 1|
|a rule that many parts follow|8|5|
|a quality goal|1|10|
|a measurable quality scenario|10|1|
|what Adunata does for users|`openspec/specs/`|6|
|what a level of tests proves|8|`CONTRIBUTING.md`|
|how to run a test or a check|`CONTRIBUTING.md`|8|
|what a job of CI does|`docs/maintenance/`|8|
|a setting and its default|`docs/operating/`|8|

## Links

Links are relative and must resolve. Link to another section by file, such as `[Building block view](05-building-block-view.md)`. Link to a subsection with its GitHub anchor, such as `05-building-block-view.md#whitebox-overall-system`.

GitHub builds an anchor from a heading. It lowercases the text, removes punctuation except hyphens and spaces. Then it replaces each space with a hyphen. A repeated heading in the same file gets the suffix `-1`, `-2` and so on. Keep headings stable once other files link to them.

Link to a decision record as `[ADR 0002](../decisions/0002-postgresql-as-database.md)` and to a capability as `[member invitations](../../openspec/specs/<capability>/spec.md)`.

## Diagrams

Diagrams are Mermaid code blocks, which GitHub renders. Use these types:

|Section|Diagram|
|-|-|
|3 Context and scope|`flowchart` with Adunata as one node and its partners around it|
|5 Building block view|`flowchart` with a `subgraph` per white box and arrows without labels for dependencies|
|6 Runtime view|`sequenceDiagram`|
|7 Deployment view|`flowchart` with a `subgraph` per node|
|8 Cross-cutting concepts|`erDiagram` for the data model|

Do not use Mermaid's C4 diagrams, which are experimental. Introduce each diagram with a sentence that says what it shows. Name every element again in the text or a table below the diagram, so the section is complete without the picture.

## When a change updates arc42

An OpenSpec change that affects the architecture names the affected sections in its `design.md`. Its `tasks.md` holds a task to update those sections and to write or supersede the decision records.

A change to the development environment alone affects no section and needs no decision record. That covers services for development, the development shell, direnv, tooling for worktrees and the workflows of CI. `README.md`, `CONTRIBUTING.md` and `docs/maintenance/` document it. Only the parts that an operator or the running system notices count, such as a rule for how the server reads its configuration. The testing concept in section 8 changes only when a level of tests appears, disappears or proves something else.

|The change|Update|
|-|-|
|adds or removes a building block or an interface|3, 5|
|adds a significant scenario or changes how blocks cooperate|6|
|changes how or where Adunata runs|7|
|adds or changes a rule that spans blocks|8|
|makes, changes or reverses a significant decision|`docs/decisions/` and 9, plus 4 if the decision is fundamental|
|adds or changes a quality goal or scenario|1, 10|
|creates, resolves or changes a risk|11|
|introduces a domain or technical term|12|

## Checklist

Before you finish an edit in `docs/arc42/`, check that:

- each new fact sits in the section the tables above name and nowhere else
- each new sentence passes the three questions under "What belongs in arc42"
- section 5 keeps its levels apart, the white box of the server lists every module of `server/src/main.rs` and every arrow matches a use in the code
- facts owned by specs or decision records appear only as links
- every relative link and anchor resolves
- `README.md` lists every section file that exists
- the text follows the documentation rules in `AGENTS.md`
