#!/usr/bin/env python3
"""Run malvin on a ``fast_tasks/<ID>`` workspace in local Docker; grade on the host.

The agent container mounts a staged copy of ``workspace/`` at ``/app``, plus
(when ``--agent=malvin``) the host ``malvin`` binary and read-only
``cursor-sdk-bridge`` (for ``cursor:`` models). When ``--model`` selects a
``pi:`` id, the host ``pi`` binary is also bind-mounted and ``MALVIN_PI`` is
set (malvin does not bundle Pi). ``--agent=cursor`` skips malvin and runs
``cursor-agent`` instead. ``grade.py``, ``goldens/``, and other grader material
stay on the host and are never bind-mounted or baked into the agent image.

Usage::

    python ops/fast_task.py solve FT-01
    python ops/fast_task.py solve FT-01 --dry-run
    python ops/fast_task.py solve FT-01 --agent=cursor
    python ops/fast_task.py solve FT-01 --main
    python ops/fast_task.py solve FT-01 --model cursor:auto
    python ops/fast_task.py solve FT-01 --model pi:openrouter/~x-ai/grok-latest
    python ops/fast_task.py solve FT-01 --creative
    python ops/fast_task.py tasks
    python ops/fast_task.py self-test

Results default to ``~/.malvin_home/fast_task_results``. Prefer a path under
``$HOME`` for ``--results-dir``: Snap Docker often cannot bind-mount host ``/tmp``.
"""

from __future__ import annotations

import json
import errno
import os
import shutil
import signal
import subprocess
import sys
import tempfile
import threading
import time
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

import click
from click.testing import CliRunner

from toolchain_repos import malvin_repo_root

REPO_ROOT = malvin_repo_root()
FAST_TASKS_ROOT = REPO_ROOT / "fast_tasks"
DEFAULT_IMAGE = "malvin-fast-task:local"
DEFAULT_BASE_IMAGE = "python:3.12-slim"
DEFAULT_AGENT_TIMEOUT_SEC = 10 * 60
TIMEOUT_EXIT_CODE = 124
_KILL_GRACE_SEC = 2.0
_POLL_INTERVAL_SEC = 0.1

AGENT_TIMEOUT_ENV = "MALVIN_FT_AGENT_TIMEOUT_SEC"
MALVIN_BIN_REMOTE = "/root/.cargo/bin/malvin"

CURSOR_SDK_BRIDGE_REMOTE = "/opt/malvin/cursor-sdk-bridge"
CURSOR_SDK_BRIDGE_JS_REMOTE = f"{CURSOR_SDK_BRIDGE_REMOTE}/dist/bridge.js"

PI_BIN_REMOTE = "/opt/malvin/pi"
CODEX_BIN_REMOTE = "/opt/malvin/codex/bin/codex.js"
CODEX_PACKAGE_REMOTE = "/opt/malvin/codex"
NODE_BIN_REMOTE = "/opt/malvin/node"
TOOLCHAIN_PATH = (
    "/root/.cargo/bin:/root/.local/bin:/usr/local/sbin:/usr/local/bin"
    ":/usr/sbin:/usr/bin:/sbin:/bin"
)
AGENT_MALVIN = "malvin"
AGENT_CURSOR = "cursor"
AGENT_CHOICES = (AGENT_MALVIN, AGENT_CURSOR)
EXTERNAL_AGENTS = frozenset({AGENT_CURSOR})
CURSOR_ENV_KEYS = ("CURSOR_AGENT_API_KEY", "CURSOR_API_KEY", "AGENT_API_KEY")
OPENROUTER_ENV_KEYS = ("OPENROUTER_API_KEY", "OPENROUTER_MAX_TOKENS")

PROVIDER_ENV_KEYS = ("OPENAI_API_KEY", "ANTHROPIC_API_KEY")

DOCKER_SECRET_ENV_KEYS = CURSOR_ENV_KEYS + OPENROUTER_ENV_KEYS + PROVIDER_ENV_KEYS
LEAK_NAME_MARKERS = ("grade.py", "goldens", "golden", "solution")

CURSOR_AGENT_SHELL = "cursor-agent --force -p < plan.md"

def ft_resolve_cursor_sdk_bridge_dir() -> Path | None:
    """Host ``cursor-sdk-bridge`` dir with built ``dist/bridge.js``, or None."""
    bridge = (REPO_ROOT / "cursor-sdk-bridge").resolve()
    if (bridge / "dist" / "bridge.js").is_file():
        return bridge
    return None

def ft_resolve_pi_bin() -> Path | None:
    """Host ``pi`` binary (``MALVIN_PI`` or ``PATH``), or None."""
    override = os.environ.get("MALVIN_PI")
    if override:
        path = Path(override).expanduser()
        if path.is_file() and os.access(path, os.X_OK):
            return path.resolve()
        return None
    which = shutil.which("pi")
    if not which:
        return None
    path = Path(which)
    if path.is_file() and os.access(path, os.X_OK):
        return path.resolve()
    return None

def ft_resolve_node_bin() -> Path | None:
    """Host Node.js binary needed by the JavaScript Codex CLI, or None."""
    which = shutil.which("node")
    if not which:
        return None
    path = Path(which)
    if path.is_file() and os.access(path, os.X_OK):
        return path.resolve()
    return None


def ft_resolve_codex_bin() -> Path | None:
    """Host ``codex`` binary (``MALVIN_CODEX`` or ``PATH``), or None."""
    override = os.environ.get("MALVIN_CODEX")
    if override:
        path = Path(override).expanduser()
        if path.is_file() and os.access(path, os.X_OK):
            return path.resolve()
        return None
    which = shutil.which("codex")
    if not which:
        return None
    path = Path(which)
    if path.is_file() and os.access(path, os.X_OK):
        return path.resolve()
    return None


def ft_malvin_args_request_pi(malvin_args: tuple[str, ...]) -> bool:
    """True when ``malvin_args`` select a ``pi:`` ``--model``."""
    for i, arg in enumerate(malvin_args):
        if arg == "--model" and i + 1 < len(malvin_args):
            if malvin_args[i + 1].startswith("pi:"):
                return True
        elif arg.startswith("--model="):
            value = arg.split("=", 1)[1]
            if value.startswith("pi:"):
                return True
    return False

def ft_malvin_args_request_codex(malvin_args: tuple[str, ...]) -> bool:
    """True when ``malvin_args`` select a ``codex:`` ``--model``."""
    for i, arg in enumerate(malvin_args):
        if arg == "--model" and i + 1 < len(malvin_args):
            if malvin_args[i + 1].startswith("codex:"):
                return True
        elif arg.startswith("--model="):
            value = arg.split("=", 1)[1]
            if value.startswith("codex:"):
                return True
    return False


def ft_malvin_args_request_creative(malvin_args: tuple[str, ...]) -> bool:
    """True when ``malvin_args`` include ``--creative``."""
    return "--creative" in malvin_args

def ft_assert_creative_compatible(agent: str, malvin_args: tuple[str, ...]) -> None:
    """Fail when ``--creative`` is paired with cursor agent backends."""
    if not ft_malvin_args_request_creative(malvin_args):
        return
    if ft_normalize_agent(agent) == AGENT_CURSOR:
        raise click.ClickException(
            "--creative and --agent=cursor are mutually exclusive"
        )
    if "--cursor" in malvin_args:
        raise click.ClickException(
            "--creative and --cursor are mutually exclusive"
        )

def ft_normalize_agent(agent: str) -> str:
    """Return a canonical agent id or raise ``click.ClickException``."""
    name = (agent or AGENT_MALVIN).strip().lower()
    if name not in AGENT_CHOICES:
        raise click.ClickException(
            f"Unknown --agent={agent!r}; expected one of {', '.join(AGENT_CHOICES)}"
        )
    return name

def ft_default_results_dir() -> Path:
    """Return ``~/.malvin_home/fast_task_results`` (override with ``FAST_TASK_RESULTS``)."""
    override = os.environ.get("FAST_TASK_RESULTS")
    if override:
        return Path(override).expanduser().resolve()
    return (Path.home() / ".malvin_home" / "fast_task_results").resolve()


def ft_run_root(task_id: str, results_dir: Path | None) -> Path:
    """Create and return a run directory, falling back if the default is read-only."""
    root = (results_dir or ft_default_results_dir()).resolve()
    run_root = root / task_id / ft_timestamp_dir()
    try:
        run_root.mkdir(parents=True, exist_ok=True)
    except OSError as exc:
        if exc.errno != errno.EROFS:
            raise
        fallback = (REPO_ROOT / ".malvin" / "fast_task_results").resolve()
        run_root = fallback / task_id / ft_timestamp_dir()
        run_root.mkdir(parents=True, exist_ok=True)
        click.echo(
            f"Default results directory is not writable; using fallback: {fallback}",
            err=True,
        )
    return run_root

def ft_agent_timeout_sec(explicit: float | None = None) -> float:
    """Resolve agent timeout: explicit arg, else ``MALVIN_FT_AGENT_TIMEOUT_SEC``, else default."""
    if explicit is not None:
        if explicit <= 0:
            raise click.ClickException("timeout_sec must be positive")
        return float(explicit)
    raw = os.environ.get(AGENT_TIMEOUT_ENV)
    if raw:
        try:
            value = float(raw)
        except ValueError as exc:
            raise click.ClickException(
                f"{AGENT_TIMEOUT_ENV} must be a positive number, got {raw!r}"
            ) from exc
        if value <= 0:
            raise click.ClickException(
                f"{AGENT_TIMEOUT_ENV} must be a positive number, got {raw!r}"
            )
        return value
    return float(DEFAULT_AGENT_TIMEOUT_SEC)

def ft_timestamp_dir() -> str:
    """UTC timestamp directory segment, e.g. ``20260715T141523Z``."""
    return datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%SZ")

def ft_list_task_ids() -> list[str]:
    """Sorted task ids under ``fast_tasks/`` (directories named ``FT-*``)."""
    if not FAST_TASKS_ROOT.is_dir():
        return []
    return sorted(
        p.name for p in FAST_TASKS_ROOT.iterdir() if p.is_dir() and p.name.startswith("FT-")
    )

def ft_resolve_task_dir(task_id: str) -> Path:
    """Resolve ``fast_tasks/<task_id>`` or raise ``click.ClickException``."""
    task_dir = (FAST_TASKS_ROOT / task_id).resolve()
    if not task_dir.is_dir():
        known = ", ".join(ft_list_task_ids()) or "(none)"
        raise click.ClickException(
            f"Unknown fast task id {task_id!r}. Known: {known}"
        )
    workspace = task_dir / "workspace"
    plan = workspace / "plan.md"
    grade = task_dir / "grade.py"
    if not plan.is_file():
        raise click.ClickException(f"Missing plan.md: {plan}")
    if not grade.is_file():
        raise click.ClickException(f"Missing grade.py: {grade}")
    return task_dir

def _ft_copy_ignore(_directory: str, names: list[str]) -> set[str]:
    return {n for n in names if n in {"__pycache__", ".pytest_cache", ".git"}}

def ft_stage_workspace(task_dir: Path, run_root: Path) -> Path:
    """Copy ``task_dir/workspace`` into ``run_root/workspace`` (workspace-only)."""
    src = task_dir / "workspace"
    dst = run_root / "workspace"
    if dst.exists():
        shutil.rmtree(dst)
    shutil.copytree(src, dst, ignore=_ft_copy_ignore)
    ft_ensure_staged_git(dst)
    ft_assert_stage_isolated(dst)
    return dst.resolve()

def ft_ensure_staged_git(workspace: Path) -> None:
    """``git init`` staged workspace so malvin uses ``/app/.malvin/checks`` (git layout).

    Non-git workspaces resolve primary checks to ``~/.malvin/checks``, while
    discovery agents typically write the legacy path ``cwd/.malvin/checks``.
    Staging always strips ``.git`` via ``_ft_copy_ignore``, so init here.
    """
    ws = workspace.resolve()
    git_dir = ws / ".git"
    if git_dir.exists():
        return
    proc = subprocess.run(
        ["git", "init"],
        cwd=ws,
        capture_output=True,
        text=True,
        check=False,
    )
    if proc.returncode != 0:
        detail = (proc.stderr or proc.stdout or "").strip()
        raise click.ClickException(
            f"git init failed for staged workspace {ws}: {detail}"
        )

def ft_assert_stage_isolated(staged: Path) -> None:
    """Fail if staged tree contains grader / golden / solution leak markers."""
    staged = staged.resolve()
    for path in staged.rglob("*"):
        name = path.name.lower()
        for marker in LEAK_NAME_MARKERS:
            if marker in name:
                raise click.ClickException(
                    f"Staged workspace must not contain leak marker {marker!r}: {path}"
                )
        if path.is_file() and path.name == "grade.py":
            raise click.ClickException(f"Staged workspace must not contain grade.py: {path}")

def ft_resolve_malvin_binary() -> Path | None:
    """Best-effort host ``malvin`` binary path for run-time bind mount."""
    return _ft_resolve_host_binary("malvin")

def ft_resolve_malvin_main_binary() -> Path | None:
    """Best-effort host ``malvin-main`` binary path for ``--main`` bind mount."""
    return _ft_resolve_host_binary("malvin-main")

def _ft_resolve_host_binary(name: str) -> Path | None:
    """Resolve ``name`` from PATH or ``~/.cargo/bin/<name>``."""
    which = shutil.which(name)
    if which:
        path = Path(which)
        if path.is_file():
            return path.resolve()
    cargo = Path.home() / ".cargo" / "bin" / name
    if cargo.is_file():
        return cargo.resolve()
    return None

def ft_dockerfile_for_agent(base_image: str = DEFAULT_BASE_IMAGE) -> str:
    """Dockerfile text for the reusable fast-task agent image (no grade material).

    Malvin is not baked into the image; ``ft_docker_agent_cmd`` bind-mounts the
    host binary (and a per-run logs dir for mini traces) at run time so evals
    use the current build.
    """
    return f"""\
FROM {base_image}
RUN apt-get update -qq \\
    && DEBIAN_FRONTEND=noninteractive apt-get install -y -qq curl ca-certificates git \\
    && rm -rf /var/lib/apt/lists/*
RUN pip install --no-cache-dir pytest
RUN curl -fsSL https://cursor.com/install | bash
ENV PATH="{TOOLCHAIN_PATH}"
WORKDIR /app
"""

def ft_assert_dockerfile_nonleak(text: str) -> None:
    """Fail if Dockerfile would copy grader / golden / solution material."""
    lower = text.lower()
    for marker in ("grade.py", "goldens", "/goldens", "solution"):
        if marker in lower:
            raise click.ClickException(
                f"Agent Dockerfile must not reference {marker!r}"
            )

def ft_docker_available() -> bool:
    """True when local Docker accepts ``docker info``."""
    try:
        proc = subprocess.run(
            ["docker", "info"],
            capture_output=True,
            text=True,
            check=False,
            timeout=30,
        )
    except (OSError, subprocess.TimeoutExpired):
        return False
    return proc.returncode == 0

def ft_ensure_agent_image(
    *,
    image: str,
    base_image: str,
    dry_run: bool,
) -> str:
    """Build or reuse ``image``; return the tag used."""
    ft_assert_dockerfile_nonleak(ft_dockerfile_for_agent(base_image))
    if dry_run:
        click.echo(f"Would ensure agent image {image} from {base_image}")
        return image
    probe = subprocess.run(
        ["docker", "image", "inspect", image],
        capture_output=True,
        text=True,
        check=False,
    )
    if probe.returncode == 0:
        click.echo(f"Using local agent image {image}")
        return image
    click.echo(
        f"Building agent image {image} from {base_image} "
        f"(cursor-agent; pytest; malvin mounted at run time)..."
    )
    with tempfile.TemporaryDirectory(prefix="malvin-fast-task-img-") as tmp:
        build_dir = Path(tmp)
        (build_dir / "Dockerfile").write_text(
            ft_dockerfile_for_agent(base_image), encoding="utf-8"
        )
        proc = subprocess.run(
            ["docker", "build", "-t", image, str(build_dir)],
            check=False,
        )
        if proc.returncode != 0:
            raise click.ClickException(f"docker build failed for {image}")
    return image

def ft_cursor_env_args() -> list[str]:
    """``docker run -e`` args for Cursor/OpenRouter secrets present on the host."""
    args: list[str] = []
    for key in DOCKER_SECRET_ENV_KEYS:
        value = os.environ.get(key)
        if value:
            args.extend(["-e", f"{key}={value}"])
    return args

def ft_redact_cmd_tokens(cmd: list[str]) -> list[str]:
    """Return *cmd* with Docker secret env values replaced by ``***``."""
    out: list[str] = []
    for token in cmd:
        redacted = token
        for key in DOCKER_SECRET_ENV_KEYS:
            prefix = f"{key}="
            if redacted.startswith(prefix) and len(redacted) > len(prefix):
                redacted = prefix + "***"
                break
        out.append(redacted)
    return out

def ft_redact_cmd_for_display(cmd: list[str]) -> str:
    """Join *cmd* for logs, redacting Docker secret env values."""
    return " ".join(ft_redact_cmd_tokens(cmd))

def ft_run_malvin_logs_dir(workspace: Path) -> Path:
    """Per-run host dir bind-mounted to container ``/root/.malvin_home/logs``.

    Lives next to the staged workspace (``<run>/malvin_logs``) so mini ACP
    traces survive ``docker run --rm`` without exposing the host's full
    ``~/.malvin_home/logs`` tree into the sandbox.
    """
    logs = workspace.resolve().parent / "malvin_logs"
    logs.mkdir(parents=True, exist_ok=True)
    return logs

def ft_docker_agent_cmd(
    *,
    image: str,
    workspace: Path,
    malvin_binary: Path | None = None,
    malvin_args: tuple[str, ...] = (),
    agent: str = AGENT_MALVIN,
) -> list[str]:
    """Agent-phase ``docker run`` argv: workspace + per-run logs mounts."""
    agent_name = ft_normalize_agent(agent)
    ws = workspace.resolve()
    host_logs = ft_run_malvin_logs_dir(ws)
    
    
    volume_mounts: list[str] = [
        "-v",
        f"{ws}:/app",
        "-v",
        f"{host_logs}:/root/.malvin_home/logs",
    ]
    bridge_env: list[str] = []
    if agent_name == AGENT_MALVIN:
        if malvin_binary is None:
            raise click.ClickException(
                "malvin_binary is required when --agent=malvin"
            )
        host_malvin = malvin_binary.resolve()
        if not host_malvin.is_file():
            raise click.ClickException(f"Host malvin binary not found: {host_malvin}")
        volume_mounts = [
            "-v",
            f"{host_malvin}:{MALVIN_BIN_REMOTE}:ro",
            *volume_mounts,
        ]
        host_bridge = ft_resolve_cursor_sdk_bridge_dir()
        if host_bridge is None:
            raise click.ClickException(
                "cursor-sdk-bridge/dist/bridge.js not found under the repo; "
                "run `npm ci && npm run build` in cursor-sdk-bridge/ "
                "(required for cursor: models inside the agent container)"
            )
        volume_mounts = [
            "-v",
            f"{host_bridge}:{CURSOR_SDK_BRIDGE_REMOTE}:ro",
            *volume_mounts,
        ]
        bridge_env = [
            "-e",
            f"MALVIN_CURSOR_SDK_BRIDGE={CURSOR_SDK_BRIDGE_JS_REMOTE}",
        ]
        if ft_malvin_args_request_pi(malvin_args):
            host_pi = ft_resolve_pi_bin()
            if host_pi is None:
                raise click.ClickException(
                    "pi binary not found on PATH (or MALVIN_PI); "
                    "required for pi: models inside the agent container "
                    "(malvin does not bundle pi)"
                )
            volume_mounts = [
                "-v",
                f"{host_pi}:{PI_BIN_REMOTE}:ro",
                *volume_mounts,
            ]
            bridge_env = [
                *bridge_env,
                "-e",
                f"MALVIN_PI={PI_BIN_REMOTE}",
            ]
        if ft_malvin_args_request_codex(malvin_args):
            host_codex = ft_resolve_codex_bin()
            host_node = ft_resolve_node_bin()
            if host_codex is None or host_node is None:
                raise click.ClickException(
                    "codex and node binaries not found on PATH (or MALVIN_CODEX); "
                    "required for codex: models inside the agent container "
                    "(malvin does not bundle codex)"
                )
            codex_package = host_codex.parent.parent
            volume_mounts = [
                "-v",
                f"{codex_package}:{CODEX_PACKAGE_REMOTE}:ro",
                "-v",
                f"{host_node}:{NODE_BIN_REMOTE}:ro",
                *volume_mounts,
            ]
            bridge_env = [
                *bridge_env,
                "-e",
                f"MALVIN_CODEX={CODEX_BIN_REMOTE}",
            ]
    container_path = (
        f"{Path(NODE_BIN_REMOTE).parent}:{TOOLCHAIN_PATH}"
        if ft_malvin_args_request_codex(malvin_args)
        else TOOLCHAIN_PATH
    )
    cmd = [
        "docker",
        "run",
        "--rm",
        *ft_cursor_env_args(),
        *volume_mounts,
        *bridge_env,
        "-e",
        f"PATH={container_path}",
        "-e",
        "MALVIN_FORCE_STDOUT_TEE=1",
        
        "-e",
        "GIT_CONFIG_COUNT=1",
        "-e",
        "GIT_CONFIG_KEY_0=safe.directory",
        "-e",
        "GIT_CONFIG_VALUE_0=/app",
        "-w",
        "/app",
        image,
    ]
    if agent_name == AGENT_CURSOR:
        
        cmd.extend(["sh", "-c", CURSOR_AGENT_SHELL])
    else:
        cmd.extend(["malvin", *malvin_args, "plan.md"])
    ft_assert_agent_cmd_nonleak(cmd, task_parent=ws.parent)
    return cmd

def ft_assert_agent_cmd_nonleak(
    cmd: list[str],
    *,
    task_parent: Path | None = None,
) -> None:
    """Fail if docker argv would expose grade/goldens or mount a task parent."""
    joined = " ".join(cmd)
    lower = joined.lower()
    for marker in ("grade.py", "/goldens", "goldens/", "solution"):
        if marker in lower:
            raise click.ClickException(
                f"Agent docker command must not reference {marker!r}: {joined}"
            )
    for i, token in enumerate(cmd):
        if token != "-v" or i + 1 >= len(cmd):
            continue
        mount = cmd[i + 1]
        host_part = mount.split(":", 1)[0]
        host_path = Path(host_part).resolve()
        if host_path.name in {"goldens"} or host_path.name == "grade.py":
            raise click.ClickException(f"Forbidden mount of {host_path}")
        if task_parent is not None and host_path == task_parent.resolve():
            raise click.ClickException(
                f"Agent must not mount task parent directory: {host_path}"
            )
        if (host_path / "grade.py").is_file() and (host_path / "workspace").is_dir():
            raise click.ClickException(
                f"Agent must not mount task root containing grade.py: {host_path}"
            )

def _ft_kill_process_group(proc: subprocess.Popen[Any]) -> None:
    """SIGTERM then SIGKILL the process group started for *proc*."""
    try:
        os.killpg(proc.pid, signal.SIGTERM)
    except ProcessLookupError:
        return
    grace_deadline = time.monotonic() + _KILL_GRACE_SEC
    while proc.poll() is None and time.monotonic() < grace_deadline:
        time.sleep(_POLL_INTERVAL_SEC)
    if proc.poll() is None:
        try:
            os.killpg(proc.pid, signal.SIGKILL)
        except ProcessLookupError:
            pass

def ft_relay_subprocess_stdout(
    cmd: list[str],
    *,
    timeout_sec: float = DEFAULT_AGENT_TIMEOUT_SEC,
) -> tuple[int, str, bool]:
    """Run *cmd*, stream merged stdout/stderr live.

    Returns ``(exit_code, capture, timed_out)``. On wall-clock expiry the
    process group is killed and ``timed_out`` is True with exit code 124.
    """
    if timeout_sec <= 0:
        return TIMEOUT_EXIT_CODE, "", True
    env = os.environ.copy()
    env.setdefault("MALVIN_FORCE_STDOUT_TEE", "1")
    proc = subprocess.Popen(
        cmd,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
        bufsize=1,
        env=env,
        start_new_session=True,
    )
    chunks: list[str] = []
    assert proc.stdout is not None
    deadline = time.monotonic() + timeout_sec
    timed_out = False

    def _reader() -> None:
        assert proc.stdout is not None
        for line in proc.stdout:
            sys.stdout.write(line)
            sys.stdout.flush()
            chunks.append(line)

    reader = threading.Thread(target=_reader, daemon=True)
    reader.start()
    while proc.poll() is None:
        if time.monotonic() >= deadline:
            timed_out = True
            _ft_kill_process_group(proc)
            proc.wait()
            break
        time.sleep(_POLL_INTERVAL_SEC)
    reader.join(timeout=_KILL_GRACE_SEC)
    if timed_out:
        return TIMEOUT_EXIT_CODE, "".join(chunks), True
    return int(proc.returncode or 0), "".join(chunks), False

def ft_preflight_workspace_mount(*, image: str, workspace: Path) -> None:
    """Fail fast if Docker cannot see ``plan.md`` at ``/app`` (e.g. Snap + ``/tmp``)."""
    ws = workspace.resolve()
    plan = ws / "plan.md"
    if not plan.is_file():
        raise click.ClickException(f"Staged workspace missing plan.md: {plan}")
    cmd = [
        "docker",
        "run",
        "--rm",
        "-v",
        f"{ws}:/app:ro",
        "-w",
        "/app",
        image,
        "test",
        "-f",
        "/app/plan.md",
    ]
    proc = subprocess.run(cmd, capture_output=True, text=True, check=False)
    if proc.returncode != 0:
        raise click.ClickException(
            "Docker cannot see staged plan.md at /app. "
            "Snap Docker often cannot bind-mount host /tmp; "
            f"use a path under $HOME (e.g. {ft_default_results_dir()}). "
            f"host_workspace={ws}"
        )

def ft_grade_on_host(
    task_dir: Path,
    workspace: Path,
    reward_out: Path,
) -> dict[str, Any]:
    """Run ``grade.py`` on the host against the staged workspace."""
    grade_py = task_dir / "grade.py"
    reward_out.parent.mkdir(parents=True, exist_ok=True)
    cmd = [
        sys.executable,
        str(grade_py),
        "--workspace",
        str(workspace.resolve()),
        "--reward-out",
        str(reward_out.resolve()),
    ]
    click.echo(f"Grading on host: {' '.join(cmd)}")
    proc = subprocess.run(cmd, text=True, check=False, capture_output=True)
    if proc.stdout:
        sys.stdout.write(proc.stdout)
        if not proc.stdout.endswith("\n"):
            sys.stdout.write("\n")
    if proc.stderr:
        sys.stderr.write(proc.stderr)
    reward: int | None = None
    if reward_out.is_file():
        text = reward_out.read_text(encoding="utf-8").strip()
        if text in {"0", "1"}:
            reward = int(text)
    return {
        "pass": reward == 1 if reward is not None else False,
        "reward": reward,
        "grader_exit_code": proc.returncode,
    }

def ft_print_evaluation_summary(
    grade_result: dict[str, Any],
    agent_result: dict[str, Any] | None,
    run_root: Path,
) -> None:
    """Print Harbor-style evaluation block including ``reward:``."""
    click.echo("\n=== Evaluation ===")
    click.echo(f"reward: {grade_result.get('reward')}")
    click.echo(f"pass: {grade_result.get('pass')}")
    if agent_result is not None:
        click.echo(f"malvin exit: {agent_result.get('exit_code')}")
        if agent_result.get("timed_out"):
            click.echo("agent timed_out: true")
        seconds = agent_result.get("agent_seconds")
        if isinstance(seconds, (int, float)):
            click.echo(f"agent_seconds: {seconds:.1f}")
    click.echo(f"artifacts: {run_root}")

def ft_exit_from_evaluation(
    grade_result: dict[str, Any],
    agent_result: dict[str, Any] | None,
) -> None:
    """Exit non-zero when the agent failed (timeout excepted).

    Reward is reported in the evaluation summary but does not determine the
    harness exit code: a completed grade with ``reward: 0`` is still success
    for ``solve`` as a runner.
    """
    _ = grade_result  
    if agent_result and not agent_result.get("timed_out"):
        code = agent_result.get("exit_code")
        if code not in (0, None):
            raise SystemExit(int(code))

def ft_run_solve(
    task_id: str,
    *,
    results_dir: Path | None = None,
    docker_image: str | None = None,
    base_image: str = DEFAULT_BASE_IMAGE,
    malvin_args: tuple[str, ...] = (),
    dry_run: bool = False,
    skip_grade: bool = False,
    agent: str = AGENT_MALVIN,
    use_main: bool = False,
    timeout_sec: float = DEFAULT_AGENT_TIMEOUT_SEC,
) -> dict[str, Any]:
    """Stage workspace, run agent in Docker, grade on host; return result dict."""
    agent_name = ft_normalize_agent(agent)
    if agent_name in EXTERNAL_AGENTS and use_main:
        raise click.ClickException(
            f"--agent={agent_name} and --main are mutually exclusive"
        )
    ft_assert_creative_compatible(agent_name, malvin_args)
    if timeout_sec <= 0:
        raise click.ClickException("timeout_sec must be positive")
    task_dir = ft_resolve_task_dir(task_id)
    run_root = ft_run_root(task_id, results_dir)
    workspace = ft_stage_workspace(task_dir, run_root)
    image = ft_ensure_agent_image(
        image=docker_image or DEFAULT_IMAGE,
        base_image=base_image,
        dry_run=dry_run,
    )
    host_malvin: Path | None = None
    if agent_name == AGENT_MALVIN:
        if use_main:
            host_malvin = ft_resolve_malvin_main_binary()
            if host_malvin is None:
                raise click.ClickException(
                    "No host malvin-main binary found "
                    "(PATH or ~/.cargo/bin/malvin-main)"
                )
        else:
            host_malvin = ft_resolve_malvin_binary()
            if host_malvin is None:
                raise click.ClickException(
                    "No host malvin binary found (PATH or ~/.cargo/bin/malvin); "
                    "build malvin on the host or use --agent=cursor / --main"
                )
    cmd = ft_docker_agent_cmd(
        image=image,
        workspace=workspace,
        malvin_binary=host_malvin,
        malvin_args=malvin_args,
        agent=agent_name,
    )
    click.echo(f"Staged workspace: {workspace}")
    click.echo(f"Agent command: {ft_redact_cmd_for_display(cmd)}")
    click.echo(f"Agent timeout: {timeout_sec:.0f}s")

    t0 = time.monotonic()
    if dry_run:
        click.echo("Dry run: skipping docker run")
        agent_result: dict[str, Any] = {
            "exit_code": 0,
            "agent_seconds": 0.0,
            "dry_run": True,
            "timed_out": False,
            "timeout_sec": timeout_sec,
            "stdout": "",
        }
    else:
        if not ft_docker_available():
            raise click.ClickException("Docker daemon is not available")
        ft_preflight_workspace_mount(image=image, workspace=workspace)
        if agent_name == AGENT_CURSOR:
            agent_label = "cursor-agent"
        elif use_main:
            agent_label = "malvin-main"
        else:
            agent_label = "malvin"
        click.echo(f"Running {agent_label} in local Docker (workspace-only mount)...")
        code, captured, timed_out = ft_relay_subprocess_stdout(
            cmd, timeout_sec=timeout_sec
        )
        if timed_out:
            click.echo(f"Agent timed out after {timeout_sec:.0f}s")
        agent_result = {
            "exit_code": code,
            "agent_seconds": time.monotonic() - t0,
            "timed_out": timed_out,
            "timeout_sec": timeout_sec,
            "stdout": captured,
        }

    reward_out = run_root / "reward.txt"
    if skip_grade:
        grade_result: dict[str, Any] = {
            "pass": None,
            "reward": None,
            "skipped": True,
        }
    else:
        grade_result = ft_grade_on_host(task_dir, workspace, reward_out)

    metadata = {
        "task_id": task_id,
        "workspace": str(workspace),
        "image": image,
        "agent": agent_result,
        "agent_name": agent_name,
        "grade": grade_result,
        "docker_cmd": ft_redact_cmd_tokens(cmd),
    }
    (run_root / "metadata.json").write_text(
        json.dumps(metadata, indent=2) + "\n",
        encoding="utf-8",
    )
    ft_print_evaluation_summary(grade_result, agent_result, run_root)
    return {"agent": agent_result, "grade": grade_result, "run_root": run_root}

def ft_cli_list_tasks() -> None:
    """List available fast task ids (Click ``tasks`` command body)."""
    ids = ft_list_task_ids()
    if not ids:
        raise click.ClickException(f"No fast tasks found under {FAST_TASKS_ROOT}")
    for task_id in ids:
        click.echo(task_id)

def ft_cli_solve(
    task_id: str,
    *,
    results_dir: Path | None,
    docker_image: str | None,
    base_image: str,
    dry_run: bool,
    skip_grade: bool,
    malvin_args: tuple[str, ...],
    agent: str = AGENT_MALVIN,
    use_main: bool = False,
    timeout_sec: float | None = None,
) -> None:
    """Run malvin on TASK_ID in Docker; report host-graded reward."""
    result = ft_run_solve(
        task_id,
        results_dir=results_dir,
        docker_image=docker_image,
        base_image=base_image,
        malvin_args=malvin_args,
        dry_run=dry_run,
        skip_grade=skip_grade,
        agent=agent,
        use_main=use_main,
        timeout_sec=ft_agent_timeout_sec(timeout_sec),
    )
    if not skip_grade and not dry_run:
        ft_exit_from_evaluation(result["grade"], result["agent"])

def ft_cli_self_test() -> None:
    """Run fast unit self-tests (no live agent)."""
    run_fast_task_self_tests()
    click.echo("ALL fast_task self-tests OK")

def run_fast_task_self_tests() -> None:
    """Deterministic checks for CLI, staging isolation, docker argv, Dockerfile."""
    _ft_test_list_and_resolve_tasks()
    _ft_test_stage_workspace_isolated()
    _ft_test_dockerfile_nonleak()
    _ft_test_docker_agent_cmd_nonleak()
    _ft_test_docker_agent_cmd_cursor()
    _ft_test_docker_agent_cmd_pi()
    _ft_test_assert_agent_cmd_rejects_task_root()
    _ft_test_grade_on_host_starter_reward_zero()
    _ft_test_solve_help_and_dry_run()
    _ft_test_solve_main_dry_run()
    _ft_test_resolve_malvin_main_binary()
    _ft_test_resolve_agent_helpers()
    _ft_test_relay_streams_before_wait()
    _ft_test_relay_timeout_kills_slow_command()
    _ft_test_print_evaluation_includes_reward()
    _ft_test_helpers_and_cli_surface()
    _ft_test_exit_from_evaluation()
    _ft_test_ensure_agent_image_dry_run()
    _ft_test_default_results_dir()
    _ft_test_redact_cmd_for_display()
    _ft_test_preflight_requires_host_plan()

def _ft_test_list_and_resolve_tasks() -> None:
    ids = ft_list_task_ids()
    assert ids, "expected FT-* task ids"
    assert "FT-01" in ids
    task_dir = ft_resolve_task_dir("FT-01")
    assert (task_dir / "workspace" / "plan.md").is_file()
    assert (task_dir / "grade.py").is_file()

def _ft_test_stage_workspace_isolated() -> None:
    task_dir = ft_resolve_task_dir("FT-01")
    with tempfile.TemporaryDirectory(prefix="ft-stage-") as tmp:
        run_root = Path(tmp)
        staged = ft_stage_workspace(task_dir, run_root)
        assert (staged / "plan.md").is_file()
        assert (staged / ".git").exists(), "staged workspace must be a git repo"
        names = {p.name for p in staged.rglob("*")}
        assert "grade.py" not in names
        assert "goldens" not in names
        parent_listing = {p.name for p in staged.parent.iterdir()}
        assert "grade.py" not in parent_listing
        assert "goldens" not in parent_listing

def _ft_test_dockerfile_nonleak() -> None:
    text = ft_dockerfile_for_agent()
    assert "pytest" in text
    assert "malvin_bin" not in text
    assert MALVIN_BIN_REMOTE not in text
    ft_assert_dockerfile_nonleak(text)
    try:
        ft_assert_dockerfile_nonleak("COPY goldens /leak")
        raise AssertionError("expected leak detection")
    except click.ClickException:
        pass

def _ft_test_docker_agent_cmd_nonleak() -> None:
    with tempfile.TemporaryDirectory(prefix="ft-ws-") as tmp:
        ws = Path(tmp) / "workspace"
        ws.mkdir()
        (ws / "plan.md").write_text("x\n", encoding="utf-8")
        host_malvin = Path(tmp) / "malvin"
        host_malvin.write_bytes(b"\x7fELF")
        cmd = ft_docker_agent_cmd(
            image=DEFAULT_IMAGE,
            workspace=ws,
            malvin_binary=host_malvin,
        )
        assert cmd[0] == "docker"
        assert "plan.md" in cmd
        assert "malvin" in cmd
        assert "cursor-agent" not in " ".join(cmd)
        mounts = [cmd[i + 1] for i, token in enumerate(cmd) if token == "-v"]
        assert len(mounts) == 4
        assert any(m.endswith(":/app") and str(ws.resolve()) in m for m in mounts)
        assert any(
            m == f"{host_malvin.resolve()}:{MALVIN_BIN_REMOTE}:ro" for m in mounts
        )
        host_bridge = ft_resolve_cursor_sdk_bridge_dir()
        assert host_bridge is not None
        assert any(
            m == f"{host_bridge}:{CURSOR_SDK_BRIDGE_REMOTE}:ro" for m in mounts
        )
        assert f"MALVIN_CURSOR_SDK_BRIDGE={CURSOR_SDK_BRIDGE_JS_REMOTE}" in cmd
        host_logs = ft_run_malvin_logs_dir(ws)
        assert any(m == f"{host_logs}:/root/.malvin_home/logs" for m in mounts)
        assert host_logs == (ws.resolve().parent / "malvin_logs")
        assert "malvin_logs" in " ".join(mounts)
        assert str(Path.home() / ".malvin_home" / "logs") + ":/root" not in " ".join(
            mounts
        )
        joined = " ".join(cmd)
        assert "grade.py" not in joined
        assert "goldens" not in joined
        assert "GIT_CONFIG_KEY_0=safe.directory" in cmd
        assert "GIT_CONFIG_VALUE_0=/app" in cmd

def _ft_test_docker_agent_cmd_cursor() -> None:
    with tempfile.TemporaryDirectory(prefix="ft-cursor-") as tmp:
        ws = Path(tmp) / "workspace"
        ws.mkdir()
        (ws / "plan.md").write_text("x\n", encoding="utf-8")
        cmd = ft_docker_agent_cmd(
            image=DEFAULT_IMAGE,
            workspace=ws,
            agent=AGENT_CURSOR,
            malvin_args=("--verbose",),
        )
        joined = " ".join(cmd)
        assert "sh" in cmd
        assert "-c" in cmd
        assert CURSOR_AGENT_SHELL in cmd
        assert "cursor-agent --force -p < plan.md" in joined
        assert "malvin" not in cmd
        assert "--verbose" not in cmd

def _ft_test_docker_agent_cmd_pi() -> None:
    """``pi:`` models bind-mount host pi and set ``MALVIN_PI``."""
    assert ft_malvin_args_request_pi(()) is False
    assert ft_malvin_args_request_pi(("--model", "cursor:auto")) is False
    assert ft_malvin_args_request_pi(("--model", "pi:openai/gpt-4o")) is True
    assert ft_malvin_args_request_pi(("--model=pi:openrouter/x",)) is True
    assert ft_malvin_args_request_pi(("--model=cursor:auto",)) is False

    host_pi = ft_resolve_pi_bin()
    with tempfile.TemporaryDirectory(prefix="ft-pi-") as tmp:
        ws = Path(tmp) / "workspace"
        ws.mkdir()
        (ws / "plan.md").write_text("x\n", encoding="utf-8")
        host_malvin = Path(tmp) / "malvin"
        host_malvin.write_bytes(b"\x7fELF")
        if host_pi is not None:
            cmd = ft_docker_agent_cmd(
                image=DEFAULT_IMAGE,
                workspace=ws,
                malvin_binary=host_malvin,
                malvin_args=("--model", "pi:openai/gpt-4o"),
            )
            mounts = [cmd[i + 1] for i, token in enumerate(cmd) if token == "-v"]
            assert any(
                m == f"{host_pi}:{PI_BIN_REMOTE}:ro" for m in mounts
            ), mounts
            assert f"MALVIN_PI={PI_BIN_REMOTE}" in cmd
            assert "--model" in cmd
            assert "pi:openai/gpt-4o" in cmd
            
            base = ft_docker_agent_cmd(
                image=DEFAULT_IMAGE,
                workspace=ws,
                malvin_binary=host_malvin,
            )
            base_mounts = [base[i + 1] for i, token in enumerate(base) if token == "-v"]
            assert not any(m.endswith(f":{PI_BIN_REMOTE}:ro") for m in base_mounts)
            assert f"MALVIN_PI={PI_BIN_REMOTE}" not in base

        _ft_mod = sys.modules[__name__]
        old_resolve = _ft_mod.ft_resolve_pi_bin
        try:
            _ft_mod.ft_resolve_pi_bin = lambda: None  # type: ignore[assignment]
            try:
                ft_docker_agent_cmd(
                    image=DEFAULT_IMAGE,
                    workspace=ws,
                    malvin_binary=host_malvin,
                    malvin_args=("--model", "pi:openai/gpt-4o"),
                )
                raise AssertionError("expected missing pi rejection")
            except click.ClickException as exc:
                assert "pi binary not found" in str(exc)
        finally:
            _ft_mod.ft_resolve_pi_bin = old_resolve  # type: ignore[assignment]

def _ft_test_assert_agent_cmd_rejects_task_root() -> None:
    task_dir = ft_resolve_task_dir("FT-01")
    bad = [
        "docker",
        "run",
        "--rm",
        "-v",
        f"{task_dir.resolve()}:/app",
        "img",
        "malvin",
        "plan.md",
    ]
    try:
        ft_assert_agent_cmd_nonleak(bad)
        raise AssertionError("expected task-root mount rejection")
    except click.ClickException:
        pass

def _ft_test_grade_on_host_starter_reward_zero() -> None:
    """Host grade path with a tiny stub grader (keeps unit tests under 1.5s)."""
    with tempfile.TemporaryDirectory(prefix="ft-grade-") as tmp:
        root = Path(tmp)
        task_dir = root / "FT-STUB"
        workspace = task_dir / "workspace"
        workspace.mkdir(parents=True)
        (workspace / "plan.md").write_text("stub\n", encoding="utf-8")
        (task_dir / "grade.py").write_text(
            "import argparse\n"
            "from pathlib import Path\n"
            "p = argparse.ArgumentParser()\n"
            "p.add_argument('--workspace', type=Path)\n"
            "p.add_argument('--reward-out', type=Path)\n"
            "a = p.parse_args()\n"
            "a.reward_out.write_text('0\\n', encoding='utf-8')\n"
            "print('FAIL')\n",
            encoding="utf-8",
        )
        staged = root / "staged"
        shutil.copytree(workspace, staged)
        reward_out = root / "reward.txt"
        result = ft_grade_on_host(task_dir, staged, reward_out)
        assert result["reward"] == 0
        assert result["pass"] is False
        assert reward_out.read_text(encoding="utf-8").strip() == "0"

def _ft_test_solve_help_and_dry_run() -> None:
    from toolchain_repos import load_ops_entry

    cli = load_ops_entry("fast_task").fast_task_cli
    runner = CliRunner()
    help_result = runner.invoke(cli, ["solve", "--help"])
    assert help_result.exit_code == 0, help_result.output
    assert "TASK_ID" in help_result.output
    assert "--agent" in help_result.output
    assert "cursor" in help_result.output
    assert "prime" not in help_result.output
    option_lines = [
        line for line in help_result.output.splitlines() if line.startswith("  --")
    ]
    assert not any(line.startswith("  --cursor") for line in option_lines)
    assert "--main" in help_result.output
    assert "--model" in help_result.output
    assert "--creative" in help_result.output
    assert "--cursor" in help_result.output  # mentioned by --creative help
    assert DEFAULT_AGENT_TIMEOUT_SEC == 600
    with tempfile.TemporaryDirectory(prefix="ft-dry-") as tmp:
        result = runner.invoke(
            cli,
            [
                "solve",
                "FT-01",
                "--dry-run",
                "--skip-grade",
                "--results-dir",
                tmp,
            ],
            catch_exceptions=False,
        )
        assert result.exit_code == 0, result.output
        assert "Agent command:" in result.output
        assert "Would ensure agent image" in result.output
        assert f"Agent timeout: {DEFAULT_AGENT_TIMEOUT_SEC}s" in result.output
        meta_paths = list(Path(tmp).glob("FT-01/*/metadata.json"))
        assert meta_paths, result.output
        meta = json.loads(meta_paths[0].read_text(encoding="utf-8"))
        assert meta["agent"]["timeout_sec"] == DEFAULT_AGENT_TIMEOUT_SEC
        assert meta["agent"]["timed_out"] is False

        env_tmp = Path(tmp) / "env-timeout"
        env_tmp.mkdir()
        old = os.environ.get(AGENT_TIMEOUT_ENV)
        os.environ[AGENT_TIMEOUT_ENV] = "900"
        try:
            env_result = runner.invoke(
                cli,
                [
                    "solve",
                    "FT-01",
                    "--dry-run",
                    "--skip-grade",
                    "--results-dir",
                    str(env_tmp),
                ],
                catch_exceptions=False,
            )
        finally:
            if old is None:
                os.environ.pop(AGENT_TIMEOUT_ENV, None)
            else:
                os.environ[AGENT_TIMEOUT_ENV] = old
        assert env_result.exit_code == 0, env_result.output
        assert "Agent timeout: 900s" in env_result.output
        env_metas = list(env_tmp.glob("FT-01/*/metadata.json"))
        assert env_metas, env_result.output
        assert (
            json.loads(env_metas[0].read_text(encoding="utf-8"))["agent"]["timeout_sec"]
            == 900.0
        )
        joined_cmd = " ".join(meta["docker_cmd"])
        assert "malvin" in meta["docker_cmd"]
        assert "plan.md" in meta["docker_cmd"]
        assert "cursor-agent" not in joined_cmd
        assert "grade.py" not in joined_cmd
        assert "goldens" not in joined_cmd
        _ft_assert_solve_model_and_agent_dry_runs(cli, runner, Path(tmp))

def _ft_assert_solve_model_and_agent_dry_runs(cli, runner, tmp: Path) -> None:
    """Model / agent dry-run argv checks (split out for kiss local-variable limits)."""
    model_tmp = tmp / "model"
    model_tmp.mkdir()
    model_result = runner.invoke(
        cli,
        [
            "solve",
            "FT-01",
            "--model",
            "cursor:composer",
            "--dry-run",
            "--skip-grade",
            "--results-dir",
            str(model_tmp),
        ],
        catch_exceptions=False,
    )
    assert model_result.exit_code == 0, model_result.output
    model_meta_paths = list(model_tmp.glob("FT-01/*/metadata.json"))
    assert model_meta_paths, model_result.output
    model_cmd = json.loads(model_meta_paths[0].read_text(encoding="utf-8"))[
        "docker_cmd"
    ]
    assert "malvin" in model_cmd
    mi = model_cmd.index("malvin")
    assert model_cmd[mi + 1 : mi + 3] == ["--model", "cursor:composer"]
    assert model_cmd[-1] == "plan.md"

    if ft_resolve_pi_bin() is not None:
        pi_tmp = tmp / "pi-model"
        pi_tmp.mkdir()
        pi_result = runner.invoke(
            cli,
            [
                "solve",
                "FT-01",
                "--model",
                "pi:openai/gpt-4o",
                "--dry-run",
                "--skip-grade",
                "--results-dir",
                str(pi_tmp),
            ],
            catch_exceptions=False,
        )
        assert pi_result.exit_code == 0, pi_result.output
        pi_meta_paths = list(pi_tmp.glob("FT-01/*/metadata.json"))
        assert pi_meta_paths, pi_result.output
        pi_cmd = json.loads(pi_meta_paths[0].read_text(encoding="utf-8"))[
            "docker_cmd"
        ]
        assert f"MALVIN_PI={PI_BIN_REMOTE}" in pi_cmd
        assert any(token.endswith(f":{PI_BIN_REMOTE}:ro") for token in pi_cmd)
        assert "--model" in pi_cmd
        assert "pi:openai/gpt-4o" in pi_cmd

    cursor_tmp = tmp / "cursor"
    cursor_tmp.mkdir()
    cursor_result = runner.invoke(
        cli,
        [
            "solve",
            "FT-01",
            "--agent=cursor",
            "--dry-run",
            "--skip-grade",
            "--results-dir",
            str(cursor_tmp),
        ],
        catch_exceptions=False,
    )
    assert cursor_result.exit_code == 0, cursor_result.output
    cursor_meta_paths = list(cursor_tmp.glob("FT-01/*/metadata.json"))
    assert cursor_meta_paths, cursor_result.output
    cursor_meta = json.loads(cursor_meta_paths[0].read_text(encoding="utf-8"))
    cursor_cmd = cursor_meta["docker_cmd"]
    cursor_joined = " ".join(cursor_cmd)
    assert "cursor-agent --force -p < plan.md" in cursor_joined
    assert "malvin" not in cursor_cmd
    assert "grade.py" not in cursor_joined
    assert "goldens" not in cursor_joined
    assert cursor_meta.get("agent_name") == AGENT_CURSOR

    rejected = runner.invoke(
        cli,
        [
            "solve",
            "FT-01",
            "--agent=prime",
            "--dry-run",
            "--skip-grade",
            "--results-dir",
            str(tmp / "prime-rejected"),
        ],
    )
    assert rejected.exit_code != 0
    assert "prime" in rejected.output.lower() or "Invalid" in rejected.output

    _ft_assert_solve_creative_dry_runs(cli, runner, tmp)

def _ft_assert_solve_creative_dry_runs(cli, runner, tmp: Path) -> None:
    """``--creative`` forwards to malvin; rejects cursor agent; allows pi:."""
    creative_tmp = tmp / "creative"
    creative_tmp.mkdir()
    creative_result = runner.invoke(
        cli,
        [
            "solve",
            "FT-01",
            "--creative",
            "--dry-run",
            "--skip-grade",
            "--results-dir",
            str(creative_tmp),
        ],
        catch_exceptions=False,
    )
    assert creative_result.exit_code == 0, creative_result.output
    creative_metas = list(creative_tmp.glob("FT-01/*/metadata.json"))
    assert creative_metas, creative_result.output
    creative_cmd = json.loads(creative_metas[0].read_text(encoding="utf-8"))[
        "docker_cmd"
    ]
    assert "malvin" in creative_cmd
    mi = creative_cmd.index("malvin")
    assert "--creative" in creative_cmd[mi:]
    assert creative_cmd[-1] == "plan.md"

    for label, extra in (
        ("agent-cursor", ["--agent=cursor"]),
        ("flag-cursor", ["--cursor"]),
    ):
        conflict = runner.invoke(
            cli,
            [
                "solve",
                "FT-01",
                "--creative",
                *extra,
                "--dry-run",
                "--skip-grade",
                "--results-dir",
                str(tmp / f"creative-{label}"),
            ],
        )
        assert conflict.exit_code != 0, (label, conflict.output)
        assert "mutually exclusive" in conflict.output, (label, conflict.output)

    if ft_resolve_pi_bin() is not None:
        pi_creative_tmp = tmp / "creative-pi"
        pi_creative_tmp.mkdir()
        pi_creative = runner.invoke(
            cli,
            [
                "solve",
                "FT-01",
                "--creative",
                "--model",
                "pi:openai/gpt-4o",
                "--dry-run",
                "--skip-grade",
                "--results-dir",
                str(pi_creative_tmp),
            ],
            catch_exceptions=False,
        )
        assert pi_creative.exit_code == 0, pi_creative.output
        pi_metas = list(pi_creative_tmp.glob("FT-01/*/metadata.json"))
        assert pi_metas, pi_creative.output
        pi_cmd = json.loads(pi_metas[0].read_text(encoding="utf-8"))["docker_cmd"]
        assert "malvin" in pi_cmd
        pmi = pi_cmd.index("malvin")
        assert "--creative" in pi_cmd[pmi:]
        assert "--model" in pi_cmd[pmi:]
        assert "pi:openai/gpt-4o" in pi_cmd[pmi:]

    assert ft_malvin_args_request_creative(()) is False
    assert ft_malvin_args_request_creative(("--creative",)) is True
    try:
        ft_assert_creative_compatible(AGENT_CURSOR, ("--creative",))
        raise AssertionError("expected creative/cursor rejection")
    except click.ClickException as exc:
        assert "mutually exclusive" in str(exc)
    try:
        ft_assert_creative_compatible(AGENT_MALVIN, ("--creative", "--cursor"))
        raise AssertionError("expected creative/--cursor rejection")
    except click.ClickException as exc:
        assert "mutually exclusive" in str(exc)
    ft_assert_creative_compatible(AGENT_MALVIN, ("--creative",))
    ft_assert_creative_compatible(
        AGENT_MALVIN, ("--creative", "--model", "pi:openai/gpt-4o")
    )
    ft_assert_creative_compatible(AGENT_MALVIN, ("--creative", "--pi"))
    ft_assert_creative_compatible(AGENT_CURSOR, ())

def _ft_test_solve_main_dry_run() -> None:
    """``--main`` mounts host malvin-main at the container malvin path."""
    from toolchain_repos import load_ops_entry

    with tempfile.TemporaryDirectory(prefix="ft-main-") as tmp:
        tmp_path = Path(tmp)
        main_bin = ft_resolve_malvin_main_binary()
        path_extra: Path | None = None
        if main_bin is None:
            path_extra = tmp_path / "bin"
            path_extra.mkdir()
            stub = path_extra / "malvin-main"
            stub.write_text("#!/bin/sh\nexit 0\n", encoding="utf-8")
            stub.chmod(0o755)
            main_bin = stub
            os.environ["PATH"] = f"{path_extra}{os.pathsep}{os.environ.get('PATH', '')}"
        try:
            main_bin = ft_resolve_malvin_main_binary()
            assert main_bin is not None, "malvin-main must be resolvable for this test"
            cli = load_ops_entry("fast_task").fast_task_cli
            runner = CliRunner()
            result = runner.invoke(
                cli,
                [
                    "solve",
                    "FT-01",
                    "--main",
                    "--dry-run",
                    "--skip-grade",
                    "--results-dir",
                    tmp,
                ],
                catch_exceptions=False,
            )
            assert result.exit_code == 0, result.output
            meta_paths = list(Path(tmp).glob("FT-01/*/metadata.json"))
            assert meta_paths, result.output
            meta = json.loads(meta_paths[0].read_text(encoding="utf-8"))
            cmd = meta["docker_cmd"]
            joined = " ".join(cmd)
            assert "malvin" in cmd
            assert "plan.md" in cmd
            assert "cursor-agent" not in joined
            mounts = [cmd[i + 1] for i, token in enumerate(cmd) if token == "-v"]
            assert any(
                m == f"{main_bin.resolve()}:{MALVIN_BIN_REMOTE}:ro" for m in mounts
            ), mounts
            conflict = runner.invoke(
                cli,
                [
                    "solve",
                    "FT-01",
                    "--main",
                    "--agent=cursor",
                    "--dry-run",
                    "--skip-grade",
                    "--results-dir",
                    tmp,
                ],
            )
            assert conflict.exit_code != 0
            assert "mutually exclusive" in conflict.output
        finally:
            if path_extra is not None:
                path = os.environ.get("PATH", "")
                prefix = f"{path_extra}{os.pathsep}"
                if path.startswith(prefix):
                    os.environ["PATH"] = path[len(prefix) :]

def _ft_test_resolve_malvin_main_binary() -> None:
    with tempfile.TemporaryDirectory(prefix="ft-main-bin-") as tmp:
        stub_dir = Path(tmp)
        stub = stub_dir / "malvin-main"
        stub.write_text("#!/bin/sh\nexit 0\n", encoding="utf-8")
        stub.chmod(0o755)
        old_path = os.environ.get("PATH", "")
        os.environ["PATH"] = f"{stub_dir}{os.pathsep}{old_path}"
        try:
            path = ft_resolve_malvin_main_binary()
            assert path is not None
            assert path.is_file()
            assert path.name == "malvin-main"
            missing = _ft_resolve_host_binary("malvin-main-does-not-exist-xyz")
            assert missing is None
        finally:
            os.environ["PATH"] = old_path

def _ft_test_resolve_agent_helpers() -> None:
    """Cover agent-id normalize + cursor/pi host-resolve edge branches."""
    assert ft_normalize_agent("CURSOR") == AGENT_CURSOR
    assert ft_normalize_agent("") == AGENT_MALVIN
    try:
        ft_normalize_agent("nope")
        raise AssertionError("expected unknown agent rejection")
    except click.ClickException as exc:
        assert "Unknown --agent" in str(exc)
    try:
        ft_normalize_agent("prime")
        raise AssertionError("expected legacy prime agent rejection")
    except click.ClickException as exc:
        assert "Unknown --agent" in str(exc)

    with tempfile.TemporaryDirectory(prefix="ft-bridge-miss-") as tmp:
        fake_root = Path(tmp)
        (fake_root / "cursor-sdk-bridge").mkdir()
        original_root = globals()["REPO_ROOT"]
        globals()["REPO_ROOT"] = fake_root
        try:
            assert ft_resolve_cursor_sdk_bridge_dir() is None
        finally:
            globals()["REPO_ROOT"] = original_root

    with tempfile.TemporaryDirectory(prefix="ft-pi-resolve-") as tmp:
        stub = Path(tmp) / "pi"
        stub.write_text("#!/bin/sh\n", encoding="utf-8")
        stub.chmod(0o755)
        old_pi = os.environ.get("MALVIN_PI")
        os.environ["MALVIN_PI"] = str(stub)
        try:
            assert ft_resolve_pi_bin() == stub.resolve()
            os.environ["MALVIN_PI"] = str(Path(tmp) / "missing")
            assert ft_resolve_pi_bin() is None
            non_exec = Path(tmp) / "pi-noexec"
            non_exec.write_text("x", encoding="utf-8")
            non_exec.chmod(0o644)
            os.environ["MALVIN_PI"] = str(non_exec)
            assert ft_resolve_pi_bin() is None
        finally:
            if old_pi is None:
                os.environ.pop("MALVIN_PI", None)
            else:
                os.environ["MALVIN_PI"] = old_pi

    old_which = shutil.which
    try:
        shutil.which = lambda _name: None  # type: ignore[assignment]
        
        old_pi = os.environ.pop("MALVIN_PI", None)
        try:
            assert ft_resolve_pi_bin() is None
        finally:
            if old_pi is not None:
                os.environ["MALVIN_PI"] = old_pi
    finally:
        shutil.which = old_which  # type: ignore[assignment]

_FT_RELAY_SPY_SEEN: list[str] = []
_FT_RELAY_SPY_ORIG = sys.stdout.write
_FT_ECHO_CAPTURE: list[str] = []

def _ft_relay_stdout_spy(text: str) -> int:
    _FT_RELAY_SPY_SEEN.append(text)
    return _FT_RELAY_SPY_ORIG(text)

def _ft_echo_capture(msg: str) -> None:
    _FT_ECHO_CAPTURE.append(str(msg))

def _ft_test_relay_streams_before_wait() -> None:
    """Claim: relay writes lines before process exit (live tee, not dump-after)."""
    global _FT_RELAY_SPY_ORIG
    _FT_RELAY_SPY_SEEN.clear()
    _FT_RELAY_SPY_ORIG = sys.stdout.write
    cmd = [sys.executable, "-c", "print('stream-line-1', flush=True)"]
    sys.stdout.write = _ft_relay_stdout_spy  # type: ignore[method-assign]
    try:
        code, captured, timed_out = ft_relay_subprocess_stdout(cmd, timeout_sec=5.0)
    finally:
        sys.stdout.write = _FT_RELAY_SPY_ORIG  # type: ignore[method-assign]
    assert code == 0
    assert timed_out is False
    assert "stream-line-1" in captured
    assert any("stream-line-1" in chunk for chunk in _FT_RELAY_SPY_SEEN)

def _ft_test_relay_timeout_kills_slow_command() -> None:
    """Claim: relay kills a slow child and reports timed_out with exit 124."""
    cmd = [sys.executable, "-c", "import time; time.sleep(30)"]
    t0 = time.monotonic()
    code, _captured, timed_out = ft_relay_subprocess_stdout(cmd, timeout_sec=0.3)
    elapsed = time.monotonic() - t0
    assert timed_out is True
    assert code == TIMEOUT_EXIT_CODE
    assert elapsed < 5.0
    zero_code, zero_out, zero_to = ft_relay_subprocess_stdout(
        ["true"], timeout_sec=0.0
    )
    assert zero_to is True
    assert zero_code == TIMEOUT_EXIT_CODE
    assert zero_out == ""

def _ft_test_print_evaluation_includes_reward() -> None:
    _FT_ECHO_CAPTURE.clear()
    original = click.echo
    click.echo = _ft_echo_capture  # type: ignore[assignment]
    try:
        ft_print_evaluation_summary(
            {"reward": 0, "pass": False},
            {"exit_code": 0, "agent_seconds": 1.25},
            Path("/tmp/artifacts"),
        )
    finally:
        click.echo = original  # type: ignore[assignment]
    text = "\n".join(_FT_ECHO_CAPTURE)
    assert "=== Evaluation ===" in text
    assert "reward: 0" in text
    assert "pass: False" in text

def _ft_test_helpers_and_cli_surface() -> None:
    from toolchain_repos import load_ops_entry

    assert ft_timestamp_dir()
    _ = ft_resolve_malvin_binary()
    _ = ft_resolve_malvin_main_binary()
    _ = ft_docker_available()
    args = ft_cursor_env_args()
    assert isinstance(args, list)
    ops = load_ops_entry("fast_task")
    runner = CliRunner()
    tasks_result = runner.invoke(ops.fast_task_cli, ["tasks"])
    assert tasks_result.exit_code == 0, tasks_result.output
    assert "FT-01" in tasks_result.output
    assert callable(ft_cli_list_tasks)
    assert callable(ft_cli_solve)
    assert callable(ft_cli_self_test)
    assert callable(run_fast_task_self_tests)
    assert callable(ops.fast_task_cli)
    assert callable(ft_preflight_workspace_mount)

def _ft_test_exit_from_evaluation() -> None:
    
    ft_exit_from_evaluation({"pass": False, "reward": 0}, {"exit_code": 0})
    try:
        ft_exit_from_evaluation(
            {"pass": True, "reward": 1},
            {"exit_code": 3, "timed_out": False},
        )
        raise AssertionError("expected SystemExit on agent nonzero")
    except SystemExit as exc:
        assert exc.code == 3
    ft_exit_from_evaluation({"pass": True, "reward": 1}, {"exit_code": 0})
    ft_exit_from_evaluation(
        {"pass": False, "reward": 0},
        {"exit_code": 2, "timed_out": True},
    )

def _ft_test_ensure_agent_image_dry_run() -> None:
    tag = ft_ensure_agent_image(
        image="malvin-fast-task:test-dry",
        base_image=DEFAULT_BASE_IMAGE,
        dry_run=True,
    )
    assert tag == "malvin-fast-task:test-dry"

def _ft_test_default_results_dir() -> None:
    path = ft_default_results_dir()
    assert path.name == "fast_task_results" or "FAST_TASK_RESULTS" in os.environ
    prev = os.environ.get("FAST_TASK_RESULTS")
    os.environ["FAST_TASK_RESULTS"] = "/tmp/ft-results-override"
    try:
        overridden = ft_default_results_dir()
    finally:
        if prev is None:
            os.environ.pop("FAST_TASK_RESULTS", None)
        else:
            os.environ["FAST_TASK_RESULTS"] = prev
    assert overridden == Path("/tmp/ft-results-override").resolve()

def _ft_test_redact_cmd_for_display() -> None:
    cmd = [
        "docker",
        "run",
        "-e",
        "CURSOR_API_KEY=super-secret",
        "-e",
        "OPENROUTER_API_KEY=or-secret",
        "-e",
        "OPENROUTER_MAX_TOKENS=8192",
        "-e",
        "PATH=/bin",
        "img",
    ]
    shown = ft_redact_cmd_for_display(cmd)
    assert "super-secret" not in shown
    assert "or-secret" not in shown
    assert "CURSOR_API_KEY=***" in shown
    assert "OPENROUTER_API_KEY=***" in shown
    assert "OPENROUTER_MAX_TOKENS=***" in shown
    assert "PATH=/bin" in shown

def _ft_test_preflight_requires_host_plan() -> None:
    with tempfile.TemporaryDirectory(prefix="ft-preflight-") as tmp:
        ws = Path(tmp) / "workspace"
        ws.mkdir()
        try:
            ft_preflight_workspace_mount(image=DEFAULT_IMAGE, workspace=ws)
            raise AssertionError("expected missing plan.md failure")
        except click.ClickException as exc:
            assert "plan.md" in str(exc)
