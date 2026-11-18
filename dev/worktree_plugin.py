"""Gives each linked worktree of Adunata its own ports and its own Compose project.

A plugin for git-worktree-init. Enable it in the main worktree with
`git config worktree-init.plugin dev/worktree_plugin.py`.
"""

import hashlib
import os
import re
import shutil
from pathlib import Path

from git_worktree_init.options import Option

OPTIONS = [
    Option("port_base", "text", "18000", "port from which the ports of the slots count", ask=False),
]

SANITIZE_ENV = [
    "COMPOSE_PROJECT_NAME",
    "POSTGRES_PORT",
    "MAILPIT_SMTP_PORT",
    "MAILPIT_UI_PORT",
    "ADNT_HTTP_LISTEN_ADDRESS",
    "ADNT_HTTP_PUBLIC_URL",
    "ADNT_DATABASE_URL",
    "ADNT_SMTP_HOST",
    "ADNT_SMTP_PORT",
    "ADNT_SMTP_ENCRYPTION",
    "ADNT_MAIL_SENDER",
    "ADNT_LOCALIZATION_TIME_ZONE",
    "ADNT_CONFIG_FILE",
    "ADUNATA_VITE_PORT",
    "ADUNATA_E2E_URL",
    "ADUNATA_E2E_MAILPIT_URL",
    "DIRENV_DIR",
    "DIRENV_FILE",
    "DIRENV_DIFF",
    "DIRENV_WATCHES",
]

COMPOSE_SETTINGS = ".env"
SERVER_SETTINGS = "server/.env.local"
CLIENT_SETTINGS = "client/.env.local"
SLOT_LINE = re.compile(r"^# Slot (\d+) of git-worktree-init$")
OLD_ENVIRONMENT_FILES = (".envrc.local", ".envrc.local.bak")
DATABASE_IN_OLD_ENVIRONMENT = re.compile(r"/adunata_wt(\d+)$", re.MULTILINE)


def slot_of(worktree: Path) -> int | None:
    settings = worktree / COMPOSE_SETTINGS
    if settings.is_file():
        lines = settings.read_text(encoding="utf-8").splitlines()
        match = SLOT_LINE.match(lines[0]) if lines else None
        if match:
            return int(match.group(1))
    return slot_of_old_environment(worktree)


def slot_of_old_environment(worktree: Path) -> int | None:
    """Reads the slot of a worktree that has not been rebased since the plugin wrote .envrc.local."""
    environment = worktree / OLD_ENVIRONMENT_FILES[0]
    if not environment.is_file():
        return None
    match = DATABASE_IN_OLD_ENVIRONMENT.search(environment.read_text(encoding="utf-8"))
    return int(match.group(1)) if match else None


def apply(ctx) -> None:
    if ctx.target.resolve() == ctx.main.resolve():
        raise RuntimeError("the main worktree keeps the defaults of compose.yaml and server/.env")
    for rel, values in settings_of(ctx).items():
        update_settings(ctx, rel, values)
    if has_tracked_server_settings(ctx):
        for rel in OLD_ENVIRONMENT_FILES:
            if ctx.remove(rel):
                ctx.info(f"removed {rel}")
    ctx.ensure_ignored(".direnv")
    build_client(ctx)
    if (ctx.target / ".envrc").is_file() and shutil.which("direnv"):
        ctx.run(["direnv", "allow", str(ctx.target)])


def next_steps(ctx) -> list[str]:
    base = port_base_of(ctx)
    steps = [
        "Services: docker compose up --detach --wait",
        f"Server: http://localhost:{base}",
        f"Vite: http://localhost:{base + 1}",
        f"Mailpit: http://localhost:{base + 4}",
    ]
    if not has_tracked_server_settings(ctx):
        steps.insert(0, "This branch has no server/.env, so its server reads no settings file. Rebase it onto main, then run: git wt refresh")
    return steps


def teardown(ctx) -> None:
    remove_project(ctx, project_of(ctx))
    remove_project(ctx, end_to_end_project_of(ctx))


def remove_project(ctx, project: str) -> None:
    command = ["docker", "compose", "-p", project, "down", "--volumes"]
    try:
        result = ctx.run(command, cwd=ctx.main, check=False, capture=True)
        problem = result.stderr.strip() if result.returncode != 0 else None
    except FileNotFoundError:
        problem = "docker is not installed"
    if problem:
        ctx.info(f"warning: could not remove the Compose project {project}: {problem}")
        ctx.info(f"remove it later with: {' '.join(command)}")
    else:
        ctx.info(f"removed the Compose project {project} and its volume")


def settings_of(ctx) -> dict[str, dict[str, str]]:
    base = port_base_of(ctx)
    server = f"http://localhost:{base}"
    return {
        COMPOSE_SETTINGS: {
            "COMPOSE_PROJECT_NAME": project_of(ctx),
            "POSTGRES_PORT": str(base + 2),
            "MAILPIT_SMTP_PORT": str(base + 3),
            "MAILPIT_UI_PORT": str(base + 4),
        },
        SERVER_SETTINGS: {
            "ADNT_HTTP_LISTEN_ADDRESS": f"127.0.0.1:{base}",
            "ADNT_HTTP_PUBLIC_URL": server,
            "ADNT_DATABASE_URL": f"postgres://adunata:adunata@localhost:{base + 2}/adunata",
            "ADNT_SMTP_PORT": str(base + 3),
            "MAILPIT_UI_PORT": str(base + 4),
        },
        CLIENT_SETTINGS: {
            "ADUNATA_VITE_PORT": str(base + 1),
            "ADNT_HTTP_LISTEN_ADDRESS": f"127.0.0.1:{base}",
            "ADNT_HTTP_PUBLIC_URL": server,
        },
    }


def update_settings(ctx, rel: str, values: dict[str, str]) -> None:
    path = ctx.target / rel
    existing = path.read_text(encoding="utf-8") if path.is_file() else ""
    slot_line = f"# Slot {ctx.slot} of git-worktree-init" if rel == COMPOSE_SETTINGS else None
    if ctx.write(rel, merged_settings(existing, values, slot_line), backup=True):
        ctx.info(f"wrote {rel}")
    ensure_ignored(ctx, rel)
    ensure_ignored(ctx, rel + ".bak")


def ensure_ignored(ctx, rel: str) -> None:
    """Ignores a file the plugin writes. A file in the root gets a leading slash, so that a file of the same name in a subdirectory stays visible."""
    if "/" in rel:
        ctx.ensure_ignored(rel)
        return
    checked = ctx.run(["git", "check-ignore", "-q", rel], check=False, capture=True)
    if checked.returncode == 0:
        return
    exclude = ctx.common_dir / "info" / "exclude"
    entry = "/" + rel
    existing = exclude.read_text(encoding="utf-8").splitlines() if exclude.exists() else []
    if entry in existing:
        return
    exclude.parent.mkdir(parents=True, exist_ok=True)
    with exclude.open("a", encoding="utf-8") as handle:
        handle.write(f"\n{entry}\n")
    ctx.info(f"added {entry} to info/exclude")


def merged_settings(existing: str, values: dict[str, str], slot_line: str | None) -> str:
    """Replaces the lines of the given keys, keeps every other line and appends missing keys."""
    lines = existing.splitlines()
    if slot_line and lines and SLOT_LINE.match(lines[0]):
        lines = lines[1:]
    merged = []
    for line in lines:
        key = line.split("=", 1)[0]
        merged.append(f"{key}={values[key]}" if key in values else line)
    present = {line.split("=", 1)[0] for line in lines}
    merged += [f"{key}={value}" for key, value in values.items() if key not in present]
    if slot_line:
        merged.insert(0, slot_line)
    return "\n".join(merged) + "\n"


def build_client(ctx) -> None:
    script = "pnpm install --frozen-lockfile --prefer-offline && pnpm run build"
    if shutil.which("nix"):
        command = ["nix", "develop", str(ctx.main), "--command", "bash", "-o", "pipefail", "-c", script]
    else:
        command = ["bash", "-o", "pipefail", "-c", script]
    ctx.run(command, cwd=ctx.target / "client")


def port_base_of(ctx) -> int:
    return int(ctx.option("port_base")) + 10 * ctx.slot


def project_of(ctx) -> str:
    return f"adunata-{port_base_of(ctx)}"


def end_to_end_project_of(ctx) -> str:
    """The Compose project of the end-to-end tests, named as client/e2e/global-setup.ts names it."""
    return f"adunata-e2e-{hashlib.sha256(root_of(ctx).encode()).hexdigest()[:12]}"


def root_of(ctx) -> str:
    """The root of the worktree as git prints it, or the recorded path with its symlinks resolved as git resolves them once the worktree is gone."""
    if ctx.target.is_dir():
        shown = ctx.run(["git", "rev-parse", "--show-toplevel"], cwd=ctx.target, check=False, capture=True)
        if shown.returncode == 0:
            return shown.stdout.strip()
    return os.path.realpath(ctx.target)


def has_tracked_server_settings(ctx) -> bool:
    return (ctx.target / "server" / ".env").is_file()
