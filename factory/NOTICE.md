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
- An approved pull request merges only after CI passes and CodeRabbit has no open remark. Otherwise the job goes back to the builder.
- The factory runs in coding agents other than Claude Code too.
- `factory/bin/job` keeps the job state out of Git through `info/exclude`, holds no lock while it waits for CI and prices long prompts of Haiku 5.5 at their own rate.
- The dashboard is fixed for keyboard use and for answers that arrive out of order.
