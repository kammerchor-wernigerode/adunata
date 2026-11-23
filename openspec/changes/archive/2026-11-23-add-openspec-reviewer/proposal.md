# Proposal

## Why

The software factory checks a job's OpenSpec change inside `reviewer-code`, as one bullet among eight. That reviewer has to judge code quality and trace every task, requirement, scenario and design decision in the same pass. OpenSpec ships the workflow `verify` for the second half, but the repository removed it when the factory arrived.

A review lens of its own gives the check against the change an agent that does nothing else and a verdict of its own on the dashboard. The model stays the one that `reviewer-code` uses for the same risk. `reviewer-code` goes back to reviewing code.

## What Changes

- A new review lens `reviewer-openspec` verifies a job's OpenSpec change against the code. It follows the skill `openspec-verify-change`, which it reads from the job's worktree, and writes `round-<N>/review-openspec.md`. A CRITICAL issue, a WARNING and a check that the skill marks "Not verified" each give `VERDICT: CHANGES` and send the job back to the builder. A SUGGESTION goes under Notes and blocks nothing. The approver reads the Notes. The reviewer runs in parallel with the other reviewers.
- The factory schedules `reviewer-openspec` only for a job whose `spec.md` names an OpenSpec change. A small fix does not get it.
- `factory/bin/route` runs a second time after the spec step, because `spec.md` does not exist at intake. The second run reads the `CHANGE:` line of `spec.md`. `factory/config.json` names the reviewer and its model per risk: Haiku for low, Sonnet for medium and high, as for `reviewer-code`.
- `reviewer-code` loses its OpenSpec checks: tasks that are ticked and done, coverage of requirements and scenarios, adherence to the design, `openspec validate` and unchanged artifacts. `reviewer-openspec` takes over each of them. It still reads the change as the spec that the code must meet.
- The OpenSpec profile goes back from `core` to `custom` with the workflow `verify`, so that `openspec update` generates the skill `openspec-verify-change` again. `openspec/README.md` describes that profile.
- The dashboard shows a chip for the new reviewer and sorts its review file within a round.
- `factory/NOTICE.md` names the new reviewer and the second routing run among the main changes.

The change `review-before-publish` merges first. This change merges second and rebases onto it. design.md lists the files that both changes edit.

## Capabilities

### New Capabilities

None. The change sets `skip_specs: true`, because the factory is development tooling and no requirement in `openspec/specs/` describes it.

### Modified Capabilities

None.

## Impact

- New file: `.claude/agents/reviewer-openspec.md`.
- Regenerated files: `.agents/skills/openspec-verify-change/SKILL.md` and the link `.claude/skills/openspec-verify-change`.
- Changed files: `.claude/agents/reviewer-code.md`, `factory/bin/route`, `factory/bin/job`, `factory/config.json`, `factory/dashboard.html`, `factory/NOTICE.md`, `.agents/skills/factory/SKILL.md` and `openspec/README.md`.
- Every person who runs `openspec update` in this repository needs the profile `custom` on their machine first, because the profile is global to the machine.
- A job with an OpenSpec change runs one more reviewer per round, which adds its cost and may add time to the round. After `review-before-publish` that includes each round that follows a remote finding.
- A job in `needs-human` that has an OpenSpec change is finished or rejected before this change merges, because its reviewer set lacks the new reviewer.
- Every changed file under `factory/`, `.claude/agents/` and `.agents/skills/factory/` matches `protected_paths`, so a person builds this change by hand. The factory cannot build it.
- No code of server or client changes. An operator and a running instance notice nothing.
