"""Gives each linked worktree of Adunata its own ports and database.

A plugin for git-worktree-init. Enable it in the main worktree with
`git config worktree-init.plugin dev/worktree_plugin.py`.
"""

import re
import shutil
import subprocess
from pathlib import Path

from git_worktree_init.options import Option

OPTIONS = [
    Option("port_base", "text", "18000", "port from which the ports of the slots count", ask=False),
    Option("clone_db", "confirm", False, "copy the database of the main worktree"),
]

SANITIZE_ENV = [
    "ADNT_HTTP_LISTEN_ADDRESS",
    "ADNT_HTTP_PUBLIC_URL",
    "ADUNATA_E2E_URL",
    "ADUNATA_VITE_PORT",
    "ADNT_DATABASE_URL",
    "PGHOST",
    "PGPORT",
    "PGUSER",
    "PGPASSWORD",
    "PGDATABASE",
    "DIRENV_DIR",
    "DIRENV_FILE",
    "DIRENV_DIFF",
    "DIRENV_WATCHES",
]

ENVIRONMENT_FILE = ".envrc.local"
DATABASE_IN_ENVIRONMENT = re.compile(r"/adunata_wt(\d+)$", re.MULTILINE)


def slot_of(worktree: Path) -> int | None:
    environment = worktree / ENVIRONMENT_FILE
    if not environment.is_file():
        return None
    match = DATABASE_IN_ENVIRONMENT.search(environment.read_text(encoding="utf-8"))
    return int(match.group(1)) if match else None


def apply(ctx) -> None:
    if ctx.target.resolve() == ctx.main.resolve():
        raise RuntimeError("the main worktree keeps the defaults of the development shell")
    ctx.write(ENVIRONMENT_FILE, environment_of(ctx), backup=True)
    ctx.ensure_ignored(ENVIRONMENT_FILE)
    ctx.ensure_ignored(ENVIRONMENT_FILE + ".bak")
    ctx.ensure_ignored(".direnv")
    prepare_database(ctx)
    in_development_shell(ctx, "npm ci --prefer-offline && npm run build", cwd=ctx.target / "client")
    if has_envrc(ctx) and has_direnv():
        ctx.run(["direnv", "allow", str(ctx.target)])


def next_steps(ctx) -> list[str]:
    port = server_port(ctx)
    steps = [f"Server: http://localhost:{port}", f"Vite: http://localhost:{port + 1}"]
    if not has_envrc(ctx):
        steps.insert(0, "This branch has no .envrc, so nothing loads these ports. Rebase it onto main.")
    elif not has_direnv():
        steps.insert(0, "Install direnv, then run: direnv allow")
    return steps


def teardown(ctx) -> None:
    in_development_shell(ctx, f"dropdb --if-exists --force {database_of(ctx)}", cwd=ctx.main)
    ctx.info(f"dropped database {database_of(ctx)}")


def environment_of(ctx) -> str:
    port = server_port(ctx)
    return (
        f"export ADNT_HTTP_LISTEN_ADDRESS=127.0.0.1:{port}\n"
        f"export ADNT_HTTP_PUBLIC_URL=http://localhost:{port}\n"
        f"export ADUNATA_E2E_URL=http://localhost:{port}\n"
        f"export ADUNATA_VITE_PORT={port + 1}\n"
        f"export ADNT_DATABASE_URL=${{ADNT_DATABASE_URL%/*}}/{database_of(ctx)}\n"
    )


def prepare_database(ctx) -> None:
    name = database_of(ctx)
    query = f"select 1 from pg_database where datname = '{name}'"
    found = in_development_shell(ctx, f'psql -tAc "{query}"', capture=True, check=False)
    if found.returncode != 0:
        raise RuntimeError(f"cannot query the database server: {found.stderr.strip()}")
    if found.stdout.strip() == "1":
        return
    in_development_shell(ctx, f"createdb {name}")
    ctx.info(f"created database {name}")
    if not ctx.option("clone_db"):
        return
    try:
        in_development_shell(ctx, f'pg_dump --no-owner "$PGDATABASE" | psql -q -v ON_ERROR_STOP=1 {name}')
    except subprocess.CalledProcessError:
        in_development_shell(ctx, f"dropdb --if-exists --force {name}")
        raise
    ctx.info(f"copied the main database into {name}")


def in_development_shell(ctx, script: str, cwd: Path | None = None, capture: bool = False, check: bool = True):
    command = ["nix", "develop", str(ctx.main), "--command", "bash", "-o", "pipefail", "-c", script]
    return ctx.run(command, cwd=cwd, check=check, capture=capture)


def server_port(ctx) -> int:
    return int(ctx.option("port_base")) + 10 * ctx.slot


def database_of(ctx) -> str:
    return f"adunata_wt{ctx.slot}"


def has_envrc(ctx) -> bool:
    return (ctx.target / ".envrc").is_file()


def has_direnv() -> bool:
    return shutil.which("direnv") is not None
