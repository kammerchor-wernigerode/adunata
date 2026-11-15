---
name: review-change
description: Review the planning artifacts of an OpenSpec change before implementation, through a subagent that has none of the session's context. It looks for errors, contradictions, misses, quality, accessibility, security and compliance, and may overrule decisions already made. Run only when a person asks for it, for example with "/review-change add-gathering-invitations high".
argument-hint: "[change] [model] [effort]"
disable-model-invocation: true
---

Review a change's planning artifacts with an independent subagent, let the user settle every finding that has more than one reasonable answer, then fold the results into the artifacts. Never edit code.

**Why a subagent:** the session that wrote a plan reads it with its own assumptions. A fresh agent sees only the files and the brief, so it finds what the author no longer sees. Keep the brief free of your own conclusions about the change. Hand over the code and the documents, not your opinion of them.

**Input**: The user's message may carry up to three optional values after the skill's name, in any order. Claude Code also passes them as `$ARGUMENTS`.

- the name of a change
- an effort level, such as `minimal`, `low`, `medium`, `high`, `xhigh` or `max`
- a model, under any name that your agent runtime accepts

A value that names an active change from `openspec list --json` is the change. A value from the list of effort levels is the effort. Any other value is the model. If the runtime does not know that model or effort, say so and ask the user. Do not fall back to another one silently.

## Steps

### 1. Select the change

If a name is given, use it. Otherwise infer it from the conversation. If only one active change exists, take it. If it is still unclear, run `openspec list --json` and ask the user to pick one of the three or four most recently modified changes, with the most recent marked "(Recommended)".

Announce: "Reviewing change: <name>".

### 2. Select model and effort

Use the model and effort from the arguments. For each one that is missing, take the value of the current session. Pass both to the subagent explicitly, because some setups refuse to start an agent without them. If the session's effort level is not visible to you, use `medium`. Use the effort levels your runtime supports, and if it has none for subagents, tell the user that the effort cannot be set.

Announce: "Reviewer: <model>, effort <effort>".

### 3. Collect what the reviewer needs

```bash
openspec status --change "<name>" --json
```

Take the artifact paths from `artifactPaths.<id>.existingOutputPaths`. Do not assume artifact names, the schema decides them. Note which artifacts do not exist yet, so the reviewer does not report their absence as a finding.

Run `openspec validate "<name>" --strict` too. A failing validation is a finding of its own, and the reviewer gets the output.

### 4. Start the reviewer

Start one subagent with the chosen model and effort, through whatever your runtime offers for that. Examples are the tool `Agent` with a general agent type in Claude Code or `spawn_agent` in Codex. The subagent needs to read files and run shell commands, and it must not inherit this session's conversation. If your runtime can start a subagent only as a fork of this session, or not at all, stop and tell the user. A review in the same context loses the distance it exists for.

Wait for the report before you continue, because the next step needs it. Give the subagent the brief below with the placeholders filled in, and nothing else from this session.

````markdown
You review a planned change before anyone implements it. Work read-only. Do not edit, create or commit any file in the repository.

Repository: <absolute path of the repository root>

Change under review: <change root>. Read every one of these files: <existing artifact paths>. These artifacts do not exist yet, so their absence is not a finding: <missing artifacts or "none">. Output of `openspec validate --strict`: <output>.

Read for context and as the yardstick:
- the agent instructions of the repository, such as AGENTS.md or CLAUDE.md
- openspec/config.yaml and the specs in openspec/specs/ that the change builds on or modifies
- the architecture decision records and the architecture documentation, wherever the agent instructions say they live
- the operating documentation, if the change touches settings or installation
- the code, as far as you need it to check claims in the change against what exists

Look for:
- errors and false technical claims
- contradictions between the artifacts, the current specs, the decision records and the architecture documentation
- misses, meaning requirements without a design answer, design decisions without a spec basis, and scenarios that are missing or cannot be tested
- quality and best practices, such as module boundaries, data model and testing strategy
- accessibility under WCAG 2.2 AA, BITV 2.0 and EN 301 549
- security, such as authentication, sessions, cross-site requests, secrets in links and logs, rate limits, enumeration and injection
- compliance, above all the GDPR, meaning data minimisation, retention, deletion and the duty to inform
- how the change works for operators and for developers, including local development and parallel worktrees

You may overrule decisions the change has already made. If you do, say which decision, why it is wrong and what should replace it.

Verify each finding against the files before you report it. Quote the exact passage with file and line. Drop anything you cannot back with evidence.

Report in English:
1. Findings ordered by severity, as blocker, major and minor. For each give id, severity, category, location, evidence, why it matters and recommendation.
2. Mark each finding CLEAR when one fix is obviously right, or DECISION when reasonable people could choose differently. For every DECISION give two to four options with their trade-offs and mark exactly one as your recommendation.
3. A short list of what the change gets right and should keep, so nobody changes it by accident.
Leave out filler and do not summarize the change.
````

### 5. Check the evidence

Read the report. For every blocker and major finding, open the quoted passage and confirm it says what the reviewer claims. Drop a finding whose evidence does not hold, and tell the user which ones you dropped and why. You do not need to recheck every minor finding, but recheck any that looks surprising.

### 6. Let the user decide

Show the user a short overview in their language, with the number of findings per severity and per kind, and the list of things to keep.

Put every DECISION finding to the user with your runtime's tool for structured questions, such as `AskUserQuestion` in Claude Code or `request_user_input` in Codex. Ask as many per call as the tool allows. Without such a tool, ask in the chat, number the questions and wait for every answer before you continue. Each question stands on its own. It names the finding, the problem and its consequence, because the user may see only the question dialog. Offer the reviewer's options with the recommended one first and marked "(Recommended)". Prefer the tool whenever it exists, because an open question in prose gets lost.

If an answer raises a new question instead of choosing an option, answer it and ask again. Do not guess.

### 7. Fold the results into the artifacts

Apply every CLEAR finding and every decision to the planning artifacts. Follow `openspec-update-change` for how to revise artifacts and keep them coherent. A finding may require a new delta spec for a capability the change now modifies. Get its template with `openspec instructions "<artifact-id>" --change "<name>" --json`, and list the capability in the proposal.

Then run `openspec validate "<name>" --strict` again.

## Output

After the review, show:

- the reviewer's model and effort
- the findings applied, grouped by topic rather than by id
- the decisions the user made, especially where they differ from the recommendation, and what the change now says about the resulting risk
- findings dropped because their evidence did not hold
- the result of `openspec validate`
- the next step, usually the next missing artifact from `openspec status`

## Guardrails

- Planning artifacts only. Never edit code, and never commit unless the user asks.
- The reviewer stays read-only. Only this session writes, and only after step 6.
- Keep the brief free of the session's opinions about the change. The reviewer's value is its distance.
- Every DECISION goes to the user. Do not settle one yourself because the recommendation looks obvious.
- If a decision changes what the change is for rather than how it does it, suggest a new change instead of bending this one.
