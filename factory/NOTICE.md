# Notice

The software factory in this repository is a modified copy of the "Software Factory for Claude Code (v2)" by ZazenCodes, taken from <https://github.com/zazencodes/zazencodes-season-3/tree/main/src/software-factory-claude-code-v2> on 2026-11-21. It is licensed under the GNU General Public License version 3, whose text is in [LICENSE](LICENSE).

These files come from it, and each one was changed for Adunata from 2026-11-21 on:

- `factory/config.json`, `factory/checks.sh`, `factory/dashboard.html`
- `factory/bin/job`, `factory/bin/gate`, `factory/bin/route`
- `.agents/skills/factory/SKILL.md`
- `.claude/agents/intake.md`, `spec-writer.md`, `builder.md`, `reviewer-code.md`, `reviewer-security.md`, `reviewer-ux.md`, `reviewer-ui.md`, `approver.md`

The main changes:

- The checks are Adunata's checks up to integration, with and without Nix.
- A feature becomes an OpenSpec change on the job branch, which replaces the spec of the original. The reviewers and the approver check the code against it.
- CI and CodeRabbit speak before the approver decides. A failing check or an open remark sends the job back to the builder, and an approved pull request merges at once.
- The factory runs in coding agents other than Claude Code too.
- `factory/bin/job` keeps the job state out of Git through `info/exclude`, holds no lock while it waits for CI and prices long prompts of Haiku 5.5 at their own rate.
- `factory/bin/job` marks a pull request ready only after CI has started a run for the pushed commit and starts a missing ready run of CI, so the wait does not hang on a check that never comes.
- `factory/bin/job feedback` also sends an approved job back to the builder, with the person's feedback in place of the approver's decision.
- The dashboard is fixed for keyboard use and for answers that arrive out of order.
- A job has up to five rounds instead of three.
- The builder has no Stop hook. It runs single checks while it works and `factory/checks.sh` as its last step. The gate confirms that run.
- Two steps follow the reviews. `factory/bin/job publish` waits for pending checks, syncs the base branch, pushes once per round, opens the pull request ready and writes `round-N/review-remote.md` from CI and CodeRabbit. `factory/bin/job merge` then only merges the verified head.
- The builder commits and never pushes. Only the scripts push.
