# Proposal

## Why

The software factory asks CI and CodeRabbit for their findings only after the approver said APPROVE, inside `job merge`. The approver decides without their results. The state of jobs 001 to 010 in `factory/jobs/` shows what follows:

- Three jobs got a `ci.md` with `CI: FAIL` after an approval. Each approval covered a state that CI or CodeRabbit then refused.
- The builder pushes by itself. Twice it pushed while the check of CodeRabbit was pending, which hits the rate limit of CodeRabbit.
- The builder answers the threads of CodeRabbit and resolves them itself. Of nine threads on pull requests of the factory, three led to a change of code. Nobody checks whether a reply holds. Nobody handled the nitpicks that CodeRabbit lists in the body of its review outside a thread.
- The local checks run up to four times per round: by the builder, by its Stop hook, by the gate and by `gate --merge` after the approval. No round of any job has a `GATE: FAIL`. The Stop hook binds only in Claude Code.
- Four of the nine built jobs ended in round 3. For two of them that was the limit of `max_rounds`. The other two had a limit of 4 after feedback. One job needed five rounds with feedback in between.

This change delivers four things. The approver decides with the results of CI and CodeRabbit in hand. A script makes the one push of a round and waits for CodeRabbit first. A reviewer judges each reply on a thread. The builder no longer resolves threads. The gate confirms the builder's last run. The run of the Stop hook and the run of `gate --merge` go away.

A finding of CI or CodeRabbit still costs a round. This change does not make it cheaper. It moves the finding in front of the approval.

## What Changes

- The pipeline gets a new order: builder, gate, local review, publish, remote, approver, merge. The reviewers see the branch before GitHub does. The approver decides after CI and CodeRabbit have spoken.
- The builder commits only. It never pushes and never opens a pull request. While it works, it runs single flake targets for what it changed, such as `.#checks.<system>.lint` or `.#checks.<system>.serverUnitTests`. It may run `serverIntegrationTests`, `clientIntegrationTests` and `contract` by name. Its last step runs `factory/checks.sh`, which is what the gate runs. Its Stop hook goes.
- The gate and `factory/checks.sh` stay at the flake target `integration`. Without Nix the script keeps needing the services. CI keeps running the integration tests on every push. Readiness adds the long jobs and the review of CodeRabbit.
- A new scripted step `publish` pushes the job branch once per round and opens the pull request or marks it ready. It waits until no check of `ci.wait_for` is pending before it pushes, so the rule about CodeRabbit moves from a prompt into code. A timeout of that wait sends the job to `needs-human`.
- A new scripted step `remote` waits for CI and CodeRabbit without an agent. It writes `round-<N>/review-remote.md` with `VERDICT: PASS` or `VERDICT: CHANGES`, which `job sync` reads like a review. It reuses the waiting, the report and the start of a missing ready run from `job merge`.
- `job merge` no longer runs the gate, pushes or waits for CI. It merges the pull request that `remote` verified. `publish` merges `origin/<base>` before its push. `main` can move after that, so `job merge` tests for a textual conflict with `git merge-tree --write-tree`. A conflict sends the job to `needs-human`, as today. A branch that is behind `main` without a conflict merges, which is the effect of today's flow.
- A gate failure, a local `CHANGES` and a remote `CHANGES` each send the job back to the builder. Each counts as one round. After a remote finding every local reviewer runs again. `max_rounds` rises from 3 to 5, which is a judgment and not a number the job state derives.
- The builder replies to a thread of CodeRabbit but never resolves one. `factory/bin/job` fetches the comments of the threads, keeps only those of the review bots and of the operator and writes them as quoted data into the round folder. `reviewer-code` reads that file, makes no call to GitHub and judges each reply with a short reason. A script resolves the thread after the push when the reply holds or the code fixes the remark. Otherwise the review says `CHANGES`. Nitpicks in the body of a review of CodeRabbit join the same list.
- A thread of CodeRabbit on a file under `openspec/changes/<name>/` gets a reply or a BLOCKED `build.md`. The builder never edits an artifact for it.
- A job that escalates without a pull request stays local. The comment on the issue names the branch and the worktree. A job that escalates with a pull request and unpushed commits pushes them first, after the same wait for pending checks that `publish` uses. The script then marks the pull request ready and posts the decision.
- The dashboard gets the stages `publish` and `remote`, a column Remote between Review and Approve and a `remote` chip in each round row.
- The parser of the reviewer's judgments gets one self-check with `assert`, which a person runs by hand.
- `AGENTS.md`, the skill `factory`, the affected agent files and `factory/NOTICE.md` describe the new flow.

CodeRabbit keeps running only on GitHub. The factory does not use the command line tool of CodeRabbit, which is a proprietary binary that every maintainer would have to install.

One change is planned next to this one and stays out of its scope:

- `add-openspec-reviewer` adds the reviewer `reviewer-openspec`, its OpenSpec profile and its chip on the dashboard. This change adds no reviewer. This change lands first. `add-openspec-reviewer` then rebases onto it. design.md lists the files that both edit.

## Capabilities

### New Capabilities

None. The change sets `skip_specs: true`, because the factory is development tooling and no requirement in `openspec/specs/` describes it.

### Modified Capabilities

None.

## Impact

- `factory/bin/job`: the stages `publish` and `remote`, the command `job publish`, a smaller `job merge`, the push on escalation, the file with the quoted replies, the parser with its self-check and changes to `advance`, `feedback` and `sync`.
- `factory/bin/gate`: no `--merge` mode and no `--hook` mode. `factory/checks.sh` does not change.
- `factory/config.json`: `max_rounds` is 5.
- `factory/dashboard.html`: stages, columns, round rows and file order.
- `factory/NOTICE.md`: the list of main changes against the original.
- `.agents/skills/factory/SKILL.md`, `.claude/agents/builder.md`, `reviewer-code.md` and `approver.md`.
- `AGENTS.md`, sections "Software factory" and "Pull requests". The README does not change.
- All of these paths are in `protected_paths` or are documentation, so a person builds this change by hand. The factory cannot build it.
- No code of server or client changes. An operator and a running instance notice nothing.
