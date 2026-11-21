# Notice

The software factory in this repository is a modified copy of the "Software Factory for Claude Code (v2)" by ZazenCodes, taken from <https://github.com/zazencodes/zazencodes-season-3/tree/main/src/software-factory-claude-code-v2> on 2026-11-21. It is licensed under the GNU General Public License version 3, whose text is in [LICENSE](LICENSE).

These files come from it, and each one was changed for Adunata on 2026-11-21:

- `factory/config.json`, `factory/checks.sh`, `factory/dashboard.html`
- `factory/bin/job`, `factory/bin/gate`, `factory/bin/route`
- `.agents/skills/factory/SKILL.md`
- `.claude/agents/intake.md`, `spec-writer.md`, `builder.md`, `reviewer-code.md`, `reviewer-security.md`, `reviewer-ux.md`, `reviewer-ui.md`, `approver.md`

The changes adapt the factory to Adunata's checks and OpenSpec process, let the merge wait for CI and CodeRabbit, and let the factory run in coding agents other than Claude Code.
