#!/usr/bin/env python3
"""Run malvin against a DeepSWE Harbor task and grade with the official verifier.

Phase-0/1 harness from ``deepswe.md``. ``solve TASK_NAME`` runs malvin in a Modal
sandbox with a Cursor API CIDR allowlist, harvests the workspace, then grades in a
separate Modal sandbox with ``block_network=True``. Modal agent runs require a Cursor
API key (``CURSOR_AGENT_API_KEY``, ``CURSOR_API_KEY``, or ``AGENT_API_KEY``) in the
shell that launches the command; interactive ``agent login`` is not available inside
Modal sandboxes. ``solve --local TASK_NAME`` runs agent and grade in separate local
Docker containers (agent image built from Harbor + malvin/cursor-agent).
``--runtime host`` runs malvin on the host and grades via Docker; ``--runtime in-sandbox``
runs both phases in the current environment (Modal sandbox or an outer ``docker run``).

Harbor per-phase timeouts from ``task.toml`` (``agent.timeout_sec``, ``verifier.timeout_sec``)
are enforced in ``run_task()`` via monotonic phase deadlines covering prep, plan, config,
malvin, and grade. Default ``solve`` (Modal) and ``solve --local`` (Docker) invoke
in-sandbox ``run_task()`` per exec; inner enforcement is primary. Modal sandbox lifetime
and local Docker ``subprocess.run`` timeouts are outer backstops with 900s headroom.

Before the agent phase, ``prepare_task_sandbox`` (``sandbox_prep.py``) replays Harbor
Dockerfile editable-install steps against the mounted workspace.

Reused DeepSWE workspaces may accumulate root-owned sandbox dirs (``.stestr``,
``.malvin/acp_spawn``); ``reset_workspace`` removes them via Docker when
the host user cannot unlink them.

Examples::

    python ops/deepswe_run.py tasks
    python ops/deepswe_run.py solve bandit-interprocedural-taint-checks
    python ops/deepswe_run.py solve --local bandit-interprocedural-taint-checks
    python ops/deepswe_run.py hello bandit-interprocedural-taint-checks  # Modal auth + CIDR smoke (no grade)
    python ops/deepswe_run.py run --task ../deep-swe/tasks/bandit-interprocedural-taint-checks
    python ops/deepswe_run.py run --task ../deep-swe/tasks/bandit-interprocedural-taint-checks --grade-only
    python ops/deepswe_run.py run --task /task --workspace /app --runtime in-sandbox --command code

Local unit tests (no agent run)::

    python ops/deepswe_run.py self-test
"""

from __future__ import annotations

import ast
import json
import os
import re
import shutil
import signal
import subprocess
import sys
import tempfile
import time
from dataclasses import dataclass
from datetime import datetime, timezone
from pathlib import Path
from typing import Any
from unittest.mock import MagicMock, patch

import click

from sandbox_prep import prepare_task_sandbox
from toolchain_repos import (
    malvin_repo_root,
    resolve_malvin_cmd,
    validate_toolchain_repos,
)

try:
    import tomllib
except ModuleNotFoundError:  # pragma: no cover - py310
    import tomli as tomllib  # type: ignore[no-redef]


MALVIN_CMD = resolve_malvin_cmd()
IN_SANDBOX_TESTS_DIR = Path("/tests")
IN_SANDBOX_LOGS_DIR = Path("/logs")
DEEPSWE_OPS_REMOTE = "/opt/malvin/ops"
DEEPSWE_RUN_REMOTE = f"{DEEPSWE_OPS_REMOTE}/deepswe_run.py"
SANDBOX_PREP_REMOTE = f"{DEEPSWE_OPS_REMOTE}/sandbox_prep.py"
TOOLCHAIN_REPOS_REMOTE = f"{DEEPSWE_OPS_REMOTE}/toolchain_repos.py"
MALVIN_TOOLCHAIN_REMOTE = "/opt/toolchain/malvin"
TOOLCHAIN_PATH = (
    "/root/.cargo/bin:/root/.local/bin:/usr/local/sbin:/usr/local/bin"
    ":/usr/sbin:/usr/bin:/sbin:/bin"
)
CURSOR_ENV_KEYS = ("CURSOR_AGENT_API_KEY", "CURSOR_API_KEY", "AGENT_API_KEY")
TIMEOUT_EXIT_CODE = 124
LOCAL_DOCKER_HEADROOM_SEC = 900
_KILL_GRACE_SEC = 2.0
_POLL_INTERVAL_SEC = 0.1


def _docker_backstop_timeout_sec(configured: float) -> float:
    """Host-side Docker ``subprocess.run`` backstop: configured phase cap plus headroom."""
    return configured + LOCAL_DOCKER_HEADROOM_SEC


def _remaining_sec(deadline: float) -> float:
    """Seconds until monotonic *deadline*; floor at 0."""
    return max(0.0, deadline - time.monotonic())


@dataclass
class SubprocessResult:
    exit_code: int
    timed_out: bool
    elapsed_sec: float
    output: str | None = None


def _kill_process_group(pgid: int, proc: subprocess.Popen[Any]) -> None:
    try:
        os.killpg(pgid, signal.SIGTERM)
    except ProcessLookupError:
        return
    grace_deadline = time.monotonic() + _KILL_GRACE_SEC
    while proc.poll() is None and time.monotonic() < grace_deadline:
        time.sleep(_POLL_INTERVAL_SEC)
    if proc.poll() is None:
        try:
            os.killpg(pgid, signal.SIGKILL)
        except ProcessLookupError:
            pass


def _run_with_timeout(
    cmd: list[str],
    *,
    cwd: Path | None = None,
    timeout_sec: float,
    stream: bool = False,
    inherit_stdio: bool = False,
    env: dict[str, str] | None = None,
) -> SubprocessResult:
    """Run *cmd* under a wall-clock cap; kill the process group on expiry."""
    if timeout_sec <= 0:
        return SubprocessResult(
            exit_code=TIMEOUT_EXIT_CODE,
            timed_out=True,
            elapsed_sec=0.0,
            output="" if stream and not inherit_stdio else None,
        )
    t0 = time.monotonic()
    deadline = t0 + timeout_sec
    popen_kwargs: dict[str, Any] = {
        "args": cmd,
        "start_new_session": True,
    }
    if cwd is not None:
        popen_kwargs["cwd"] = str(cwd)
    if env is not None:
        popen_kwargs["env"] = env
    chunks: list[str] = []
    if stream and not inherit_stdio:
        popen_kwargs["stdout"] = subprocess.PIPE
        popen_kwargs["stderr"] = subprocess.STDOUT
        popen_kwargs["text"] = True
        popen_kwargs["bufsize"] = 1
    elif not inherit_stdio:
        popen_kwargs["stdout"] = subprocess.PIPE
        popen_kwargs["stderr"] = subprocess.PIPE
        popen_kwargs["text"] = True

    proc = subprocess.Popen(**popen_kwargs)
    pgid = proc.pid
    timed_out = False
    while proc.poll() is None:
        if time.monotonic() >= deadline:
            timed_out = True
            _kill_process_group(pgid, proc)
            proc.wait()
            break
        if stream and not inherit_stdio and proc.stdout is not None:
            line = proc.stdout.readline()
            if line:
                sys.stdout.write(line)
                sys.stdout.flush()
                chunks.append(line)
        else:
            time.sleep(_POLL_INTERVAL_SEC)

    elapsed = time.monotonic() - t0
    if timed_out:
        return SubprocessResult(
            exit_code=TIMEOUT_EXIT_CODE,
            timed_out=True,
            elapsed_sec=elapsed,
            output="".join(chunks) if stream and not inherit_stdio else None,
        )

    if inherit_stdio:
        exit_code = int(proc.returncode or 0)
        return SubprocessResult(
            exit_code=exit_code,
            timed_out=False,
            elapsed_sec=elapsed,
            output=None,
        )

    stdout = proc.stdout.read() if proc.stdout is not None else ""
    stderr = proc.stderr.read() if proc.stderr is not None else ""
    if stream:
        output = "".join(chunks)
        if proc.stdout is not None:
            output += proc.stdout.read() or ""
    else:
        output = (stdout or "") + (stderr or "")
    return SubprocessResult(
        exit_code=int(proc.returncode or 0),
        timed_out=False,
        elapsed_sec=elapsed,
        output=output,
    )


def default_deepswe_tasks_root() -> Path:
    """Default DeepSWE task tree (``DEEPSWE_TASKS`` or sibling ``../deep-swe/tasks``)."""
    override = os.environ.get("DEEPSWE_TASKS")
    if override:
        return Path(override).resolve()
    return malvin_repo_root().parent / "deep-swe" / "tasks"


def default_deepswe_results_dir() -> Path:
    """Eval artifact root outside the malvin repo so quality gates are not polluted."""
    return Path.home() / ".malvin_home" / "deepswe-results"


def resolve_local_task_dir(task_name: str) -> Path:
    """Resolve a DeepSWE task id to a task directory under ``default_deepswe_tasks_root()``."""
    task_dir = (default_deepswe_tasks_root() / task_name).resolve()
    if not task_dir.is_dir():
        raise click.ClickException(
            f"DeepSWE task {task_name!r} not found at {task_dir} "
            f"(set DEEPSWE_TASKS or clone deep-swe next to malvin)"
        )
    return task_dir


def read_task_language(task_dir: Path) -> str:
    """Return ``metadata.language`` from a task directory's ``task.toml``."""
    toml_path = task_dir / "task.toml"
    raw = tomllib.loads(toml_path.read_text(encoding="utf-8"))
    language = raw.get("metadata", {}).get("language")
    if isinstance(language, str) and language.strip():
        return language.strip()
    return "?"


def list_deepswe_tasks() -> list[str]:
    """Return sorted DeepSWE task ids under ``default_deepswe_tasks_root()``."""
    return [task_id for task_id, _language in list_deepswe_tasks_with_language()]


def list_deepswe_tasks_with_language() -> list[tuple[str, str]]:
    """Return sorted ``(task_id, language)`` pairs under ``default_deepswe_tasks_root()``."""
    tasks_root = default_deepswe_tasks_root()
    if not tasks_root.is_dir():
        return []
    entries: list[tuple[str, str]] = []
    for entry in tasks_root.iterdir():
        if not entry.is_dir() or not (entry / "task.toml").is_file():
            continue
        entries.append((entry.name, read_task_language(entry)))
    return sorted(entries, key=lambda pair: pair[0])


@dataclass(frozen=True)
class TaskSpec:
    task_dir: Path
    task_id: str
    base_commit: str
    docker_image: str
    dockerfile: Path
    instruction: Path
    tests_dir: Path
    test_sh: Path
    solution_patch: Path | None
    repository_url: str | None
    agent_timeout_sec: float
    verifier_timeout_sec: float
    environment_memory_mb: int


def _agent_timeout_result(spec: TaskSpec, *, agent_seconds: float = 0.0) -> dict[str, Any]:
    return {
        "exit_code": TIMEOUT_EXIT_CODE,
        "timed_out": True,
        "timeout_sec": spec.agent_timeout_sec,
        "agent_seconds": agent_seconds,
    }


def _grade_timeout_result(spec: TaskSpec) -> dict[str, Any]:
    return {
        "pass": False,
        "reward": 0,
        "timed_out": True,
        "timeout_sec": spec.verifier_timeout_sec,
        "verifier_exit_code": TIMEOUT_EXIT_CODE,
    }


def parse_task_dir(task_dir: Path) -> TaskSpec:
    task_dir = task_dir.resolve()
    toml_path = task_dir / "task.toml"
    if not toml_path.is_file():
        raise click.ClickException(f"Missing task.toml: {toml_path}")
    raw = tomllib.loads(toml_path.read_text(encoding="utf-8"))
    meta = raw.get("metadata", {})
    env = raw.get("environment", {})
    agent = raw.get("agent", {})
    verifier = raw.get("verifier", {})
    task_id = meta.get("task_id") or task_dir.name
    base_commit = meta.get("base_commit_hash")
    if not base_commit:
        raise click.ClickException(f"task.toml missing metadata.base_commit_hash: {toml_path}")
    docker_image = env.get("docker_image")
    if not docker_image:
        raise click.ClickException(f"task.toml missing environment.docker_image: {toml_path}")
    instruction = task_dir / "instruction.md"
    if not instruction.is_file():
        raise click.ClickException(f"Missing instruction.md: {instruction}")
    tests_dir = task_dir / "tests"
    test_sh = tests_dir / "test.sh"
    solution = task_dir / "solution" / "solution.patch"
    return TaskSpec(
        task_dir=task_dir,
        task_id=task_id,
        base_commit=base_commit,
        docker_image=docker_image,
        dockerfile=task_dir / "environment" / "Dockerfile",
        instruction=instruction,
        tests_dir=tests_dir,
        test_sh=test_sh,
        solution_patch=solution if solution.is_file() else None,
        repository_url=meta.get("repository_url"),
        agent_timeout_sec=float(agent.get("timeout_sec", 5400.0)),
        verifier_timeout_sec=float(verifier.get("timeout_sec", 1800.0)),
        environment_memory_mb=int(env.get("memory_mb", 4096)),
    )


def validate_verifier_paths(spec: TaskSpec) -> None:
    """Confirm verifier files exist. Call before grading, not during parsing."""
    if not spec.test_sh.is_file():
        raise click.ClickException(f"Missing tests/test.sh: {spec.test_sh}")


def timestamp_dir() -> str:
    return datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%SZ")


def run_cmd(
    cmd: list[str],
    *,
    cwd: Path | None = None,
    env: dict[str, str] | None = None,
    dry_run: bool = False,
    check: bool = True,
) -> subprocess.CompletedProcess[str]:
    click.echo(f"$ {' '.join(cmd)}" + (f"  (cwd={cwd})" if cwd else ""))
    if dry_run:
        return subprocess.CompletedProcess(cmd, 0, "", "")
    merged = os.environ.copy()
    if env:
        merged.update(env)
    proc = subprocess.run(
        cmd,
        cwd=str(cwd) if cwd else None,
        env=merged,
        text=True,
        capture_output=True,
        check=False,
    )
    if proc.stdout:
        sys.stdout.write(proc.stdout)
    if proc.stderr:
        sys.stderr.write(proc.stderr)
    if check and proc.returncode != 0:
        raise click.ClickException(
            f"Command failed ({proc.returncode}): {' '.join(cmd)}\n{proc.stderr or proc.stdout}"
        )
    return proc


def git_run(workspace: Path, *args: str, dry_run: bool = False) -> None:
    ws = str(workspace.resolve())
    run_cmd(
        ["git", "-c", f"safe.directory={ws}", *args],
        cwd=workspace,
        dry_run=dry_run,
    )


def docker_daemon_available() -> bool:
    """True when the local Docker daemon accepts ``docker info``."""
    try:
        proc = subprocess.run(
            ["docker", "info"],
            capture_output=True,
            text=True,
            check=False,
        )
    except OSError:
        return False
    return proc.returncode == 0


def skip_docker_selftests() -> bool:
    """True when Docker-backed self-tests should no-op (timing gate, offline runs)."""
    return os.environ.get("DEEPSWE_SKIP_DOCKER_SELFTESTS", "") == "1"


def deepswe_test_fast_grade_enabled() -> bool:
    """Stub Harbor grade in self-tests so docker-marked cases stay under the timing budget."""
    return os.environ.get("DEEPSWE_TEST_FAST_GRADE", "") == "1"


def _fast_grade_selftest_result(logs_dir: Path) -> dict[str, Any]:
    logs_dir.mkdir(parents=True, exist_ok=True)
    verifier_dir = logs_dir / "verifier"
    verifier_dir.mkdir(parents=True, exist_ok=True)
    (verifier_dir / "reward.txt").write_text("1\n", encoding="utf-8")
    return {"pass": True, "reward": 1, "fast_selftest_stub": True}


EPHEMERAL_CACHE_DIR_NAMES = (
    "__pycache__",
    ".pytest_cache",
    ".mypy_cache",
    ".ruff_cache",
    ".hypothesis",
    "coverage",
)
SANDBOX_ARTIFACT_DIR_NAMES = (
    ".stestr",
    ".kiss",
)
MALVIN_EPHEMERAL_SUBDIR_NAMES = (
    "acp_spawn",
    "checks",
)
DOCKER_EPHEMERAL_PURGE_IMAGE = "alpine:latest"
DOCKER_RUN_FAST_ARGS = ("--pull=never", "--network", "none", "--memory", "64m", "--rm")


def ephemeral_cache_find_expr() -> str:
    parts = " -o ".join(f"-name {name}" for name in EPHEMERAL_CACHE_DIR_NAMES)
    return f"\\( {parts} \\)"


def sandbox_artifact_find_expr() -> str:
    parts = " -o ".join(f"-name {name}" for name in SANDBOX_ARTIFACT_DIR_NAMES)
    return f"\\( {parts} \\)"


def docker_purge_shell() -> str:
    """Shell run inside Alpine to remove root-owned sandbox caches and artifacts."""
    prune = (
        "\\( -path /app/.git -o -path /app/.deepswe_tombstones "
        "-o -path '/app/.deepswe_tombstones/*' \\) -prune -o "
    )
    cache_expr = ephemeral_cache_find_expr()
    artifact_expr = sandbox_artifact_find_expr()
    malvin_subdirs = " ".join(
        f"/app/.malvin/{name}" for name in MALVIN_EPHEMERAL_SUBDIR_NAMES
    )
    return (
        f"find /app {prune}{cache_expr} -type d -prune -exec rm -rf {{}} + ; "
        f"find /app {prune}{artifact_expr} -type d -prune -exec rm -rf {{}} + ; "
        f"rm -rf {malvin_subdirs} 2>/dev/null || true"
    )


def purge_root_owned_ephemeral_caches(workspace: Path, *, dry_run: bool = False) -> bool:
    """Remove sandbox bytecode caches and runtime artifacts via Docker when the host user cannot unlink them."""
    if dry_run or not docker_daemon_available():
        return False
    ws = str(workspace.resolve())
    shell = docker_purge_shell()
    cmd = [
        "docker",
        "run",
        *DOCKER_RUN_FAST_ARGS,
        "-v",
        f"{ws}:/app",
        DOCKER_EPHEMERAL_PURGE_IMAGE,
        "sh",
        "-c",
        shell,
    ]
    proc = subprocess.run(cmd, capture_output=True, text=True, check=False)
    if proc.returncode != 0:
        click.echo(
            "Warning: ephemeral cache purge via Docker failed: "
            f"{proc.stderr or proc.stdout}",
            err=True,
        )
        return False
    return True


def git_clean(workspace: Path, *, dry_run: bool = False) -> bool:
    ws = str(workspace.resolve())
    proc = run_cmd(
        ["git", "-c", f"safe.directory={ws}", "clean", "-fdx"],
        cwd=workspace,
        dry_run=dry_run,
        check=False,
    )
    return proc.returncode == 0


def materialize_workspace(spec: TaskSpec, workspace: Path, *, dry_run: bool) -> None:
    workspace = workspace.resolve()
    if workspace.exists() and any(workspace.iterdir()):
        click.echo(f"Reusing existing workspace: {workspace}")
        return
    if not spec.repository_url:
        raise click.ClickException(
            "Workspace missing and task.toml has no metadata.repository_url; "
            "provide --workspace with an existing checkout."
        )
    workspace.parent.mkdir(parents=True, exist_ok=True)
    run_cmd(
        ["git", "clone", spec.repository_url, str(workspace)],
        dry_run=dry_run,
    )
    git_run(workspace, "checkout", spec.base_commit, dry_run=dry_run)


def reset_workspace(spec: TaskSpec, workspace: Path, *, dry_run: bool) -> None:
    git_run(workspace, "reset", "--hard", spec.base_commit, dry_run=dry_run)
    if dry_run:
        git_run(workspace, "clean", "-fdx", dry_run=True)
        return
    purge_root_owned_ephemeral_caches(workspace)
    if git_clean(workspace):
        return
    click.echo("git clean failed; retrying after Docker ephemeral purge", err=True)
    if purge_root_owned_ephemeral_caches(workspace) and git_clean(workspace):
        return
    raise click.ClickException(
        "git clean -fdx failed after reset (likely root-owned untracked files). "
        "Ensure Docker is available or remove the workspace checkout."
    )


def canonical_tool(line: str) -> str:
    """First whitespace-delimited token, lowercased (for deduping check command lines)."""
    parts = line.strip().split()
    return parts[0].lower() if parts else ""


def parse_yaml_scalar(raw: str) -> str:
    s = raw.strip()
    if len(s) >= 2 and s[0] == s[-1] and s[0] in "\"'":
        return s[1:-1].strip()
    return s


def precommit_hook_entries(root: Path) -> list[str]:
    path = root / ".pre-commit-config.yaml"
    if not path.is_file():
        return []
    out: list[str] = []
    for line in path.read_text(encoding="utf-8").splitlines():
        trimmed = line.strip()
        if not trimmed.startswith("entry:"):
            continue
        cmd = parse_yaml_scalar(trimmed[len("entry:") :])
        if cmd:
            out.append(cmd)
    return out


def next_makefile_recipe(lines_iter: list[str], index: int) -> tuple[str | None, int]:
    while index < len(lines_iter):
        line = lines_iter[index]
        if not line.strip():
            index += 1
            continue
        if not line.startswith("\t"):
            break
        recipe = line.strip()
        index += 1
        if recipe and not recipe.startswith("#"):
            return recipe, index
        return None, index
    return None, index


def makefile_gate_targets(root: Path) -> list[str]:
    for name in ("Makefile", "makefile", "GNUmakefile"):
        path = root / name
        if not path.is_file():
            continue
        raw_lines = path.read_text(encoding="utf-8").splitlines(keepends=False)
        out: list[str] = []
        index = 0
        while index < len(raw_lines):
            line = raw_lines[index]
            trimmed = line.rstrip()
            if not trimmed or trimmed.lstrip().startswith("#"):
                index += 1
                continue
            target = trimmed[:-1] if trimmed.endswith(":") else trimmed
            if target.strip() not in ("lint", "test"):
                index += 1
                continue
            recipe, index = next_makefile_recipe(raw_lines, index + 1)
            if recipe:
                out.append(recipe)
        return out
    return []


def gate_tool_signals(line: str) -> list[str]:
    trimmed = line.strip()
    out: list[str] = []
    if "cargo clippy" in trimmed:
        out.append("cargo-clippy")
    tool = canonical_tool(trimmed)
    if tool == "ruff":
        out.append("ruff")
    if tool == "pytest":
        out.append("pytest")
    if tool == "cargo":
        if "nextest" in trimmed:
            out.append("cargo-nextest")
        elif " test" in trimmed:
            out.append("cargo-test")
    return out


def dedupe_check_lines(lines: list[str]) -> list[str]:
    out: list[str] = []
    seen: set[str] = set()
    for line in lines:
        trimmed = line.strip()
        if not trimmed:
            continue
        tool = canonical_tool(trimmed)
        if tool in seen:
            continue
        seen.add(tool)
        out.append(trimmed)
    return out


def supplement_makefile_signals(precommit: list[str], makefile: list[str]) -> list[str]:
    merged = list(precommit)
    for line in makefile:
        signals = gate_tool_signals(line)
        if not signals:
            continue
        if all(
            any(sig in gate_tool_signals(existing) for existing in merged)
            for sig in signals
        ):
            continue
        merged.append(line)
    return merged


def visit_source_files(root: Path) -> list[Path]:
    skip_dirs = {".git", "target", "__pycache__"}
    found: list[Path] = []

    def walk(directory: Path) -> None:
        try:
            entries = list(directory.iterdir())
        except OSError:
            return
        for entry in entries:
            if entry.is_symlink():
                if entry.is_file():
                    found.append(entry)
                continue
            if entry.is_file():
                found.append(entry)
            elif entry.is_dir():
                if entry.name.startswith(".") or entry.name in skip_dirs:
                    continue
                walk(entry)

    walk(root)
    return found


def existing_malvin_checks_lines(root: Path) -> list[str]:
    path = root / ".malvin" / "checks"
    if not path.is_file():
        return []
    return [
        line.strip()
        for line in path.read_text(encoding="utf-8").splitlines()
        if line.strip()
    ]


def discover_deepswe_check_lines(root: Path) -> list[str]:
    """DeepSWE checks discovery: pre-commit/Makefile signals and existing checks only."""
    precommit = precommit_hook_entries(root)
    makefile = makefile_gate_targets(root)
    if precommit:
        signal_lines = supplement_makefile_signals(precommit, makefile)
    else:
        signal_lines = list(makefile)
    signal_lines.extend(existing_malvin_checks_lines(root))
    return dedupe_check_lines(signal_lines)


def discover_deepswe_checks(workspace: Path) -> str:
    """Build default DeepSWE ``.malvin/checks`` from repo signals only."""
    if not workspace.is_dir():
        return "\n"
    lines = discover_deepswe_check_lines(workspace)
    if not lines:
        return "\n"
    return "\n".join(lines) + "\n"


_MONKEYPATCH_HOOK_RE = re.compile(
    r"(?:monkeypatch\.setattr|mocker\.patch\.object|patch\.object)\s*\(\s*"
    r"([^,\)]+)\s*,\s*[\"']([^\"']+)[\"']",
    re.MULTILINE,
)


_PATCH_SURFACE_PROBE_COMMAND = "python3 .malvin/patch_surface_probe.py"
_PATCH_SURFACE_PROBE_MAX_TARGETS = 40


def _python_module_name(path: Path, workspace: Path) -> str:
    rel = path.relative_to(workspace)
    return ".".join(rel.with_suffix("").parts)


def scan_class_level_attributes(workspace: Path) -> list[tuple[str, str]]:
    """Return sorted ``(qualified_class, attr)`` for class-body assignments in source."""
    results: set[tuple[str, str]] = set()
    skip_parts = {".git", "__pycache__", ".malvin", "tests", "test", "deprecated"}
    for path in visit_source_files(workspace):
        if path.suffix != ".py":
            continue
        if any(part in skip_parts or part.startswith("test_") for part in path.parts):
            continue
        try:
            tree = ast.parse(path.read_text(encoding="utf-8"))
        except (SyntaxError, UnicodeDecodeError):
            continue
        module = _python_module_name(path, workspace)
        for node in tree.body:
            if not isinstance(node, ast.ClassDef):
                continue
            qual = f"{module}.{node.name}"
            for item in node.body:
                if isinstance(item, ast.Assign):
                    for target in item.targets:
                        if isinstance(target, ast.Name) and not target.id.startswith("__"):
                            results.add((qual, target.id))
                elif isinstance(item, ast.AnnAssign) and isinstance(item.target, ast.Name):
                    if not item.target.id.startswith("__"):
                        results.add((qual, item.target.id))
    return sorted(results)


def _hook_class_names(hooks: list[tuple[str, str]]) -> set[str]:
    names: set[str] = set()
    for target, _attr in hooks:
        token = target.strip().split(".")[-1]
        if token:
            names.add(token)
    return names


def patch_surface_targets(
    workspace: Path,
    *,
    hooks: list[tuple[str, str]] | None = None,
) -> list[tuple[str, str]]:
    """Select class attributes Harbor-style tests are likely to monkeypatch."""
    hooks = hooks if hooks is not None else scan_pytest_monkeypatch_hooks(workspace)
    hook_names = _hook_class_names(hooks)
    hook_pairs = {
        (target, attr)
        for target, attr in hooks
        if "." in target and not target.startswith("monkeypatch")
    }
    by_class: dict[str, list[str]] = {}
    for qual, attr in scan_class_level_attributes(workspace):
        by_class.setdefault(qual, []).append(attr)
    selected: set[tuple[str, str]] = set(hook_pairs)
    for qual, attrs in by_class.items():
        short = qual.rsplit(".", 1)[-1]
        if short in hook_names or len(attrs) >= 2:
            selected.update((qual, attr) for attr in attrs)
    ordered = sorted(selected)
    return ordered[:_PATCH_SURFACE_PROBE_MAX_TARGETS]


def render_patch_surface_probe(targets: list[tuple[str, str]]) -> str:
    """Render an offline gate script that verifies monkeypatch.setattr still works."""
    lines = [
        "#!/usr/bin/env python3",
        '"""Verify baseline class attributes remain monkeypatch-settable (DeepSWE gate)."""',
        "from __future__ import annotations",
        "",
        "import importlib",
        "import sys",
        "",
        "TARGETS: list[tuple[str, str]] = [",
    ]
    for qual, attr in targets:
        lines.append(f'    ({qual!r}, {attr!r}),')
    lines.extend(
        [
            "]",
            "",
            "",
            "def _import_class(qual: str):",
            '    module_name, _, class_name = qual.rpartition(".")',
            "    mod = importlib.import_module(module_name)",
            "    return getattr(mod, class_name)",
            "",
            "",
            "def main() -> int:",
            "    errors: list[str] = []",
            "    for qual, attr in TARGETS:",
            "        try:",
            "            cls = _import_class(qual)",
            "        except Exception as exc:",
            '            errors.append(f"{qual}: import failed: {exc}")',
            "            continue",
            "        if not hasattr(cls, attr):",
            '            errors.append(f"{qual}.{attr}: missing class attribute")',
            "            continue",
            "        sentinel = object()",
            "        old = getattr(cls, attr)",
            "        try:",
            "            setattr(cls, attr, sentinel)",
            "            if getattr(cls, attr) is not sentinel:",
            '                errors.append(f"{qual}.{attr}: setattr did not stick")',
            "        except Exception as exc:",
            '            errors.append(f"{qual}.{attr}: not patchable: {exc}")',
            "        finally:",
            "            setattr(cls, attr, old)",
            "    if errors:",
            "        for err in errors:",
            '            print(err, file=sys.stderr)',
            "        return 1",
            '    print(f"patch surface ok ({len(TARGETS)} targets)")',
            "    return 0",
            "",
            "",
            'if __name__ == "__main__":',
            "    raise SystemExit(main())",
            "",
        ]
    )
    return "\n".join(lines)


def scan_pytest_monkeypatch_hooks(workspace: Path) -> list[tuple[str, str]]:
    """Return sorted unique ``(target, attr)`` pairs from visible pytest patch patterns."""
    hooks: set[tuple[str, str]] = set()
    tests_root = workspace / "tests"
    if not tests_root.is_dir():
        return []
    for path in tests_root.rglob("*.py"):
        try:
            text = path.read_text(encoding="utf-8")
        except (OSError, UnicodeDecodeError):
            continue
        for match in _MONKEYPATCH_HOOK_RE.finditer(text):
            target = match.group(1).strip()
            attr = match.group(2).strip()
            if target and attr:
                hooks.add((target, attr))
    return sorted(hooks)


DEFAULT_MALVIN_MEM_LIMIT_GB = 4


def malvin_mem_limit_gb(environment_memory_mb: int) -> int:
    """Map Harbor ``environment.memory_mb`` to malvin ``mem_limit_gb`` (round up)."""
    mb = max(0, int(environment_memory_mb))
    if mb <= DEFAULT_MALVIN_MEM_LIMIT_GB * 1024:
        return DEFAULT_MALVIN_MEM_LIMIT_GB
    return (mb + 1023) // 1024


def ensure_deepswe_malvin_config(spec: TaskSpec, *, dry_run: bool) -> None:
    """Seed ``~/.malvin_home/config.toml`` so malvin USS cap matches the task envelope."""
    mem_gb = malvin_mem_limit_gb(spec.environment_memory_mb)
    if mem_gb <= DEFAULT_MALVIN_MEM_LIMIT_GB:
        return
    config_path = Path.home() / ".malvin_home" / "config.toml"
    click.echo(f"Seeding malvin USS cap mem_limit_gb={mem_gb} ({config_path})")
    if dry_run:
        return
    config_path.parent.mkdir(parents=True, exist_ok=True)
    config_path.write_text(f"mem_limit_gb = {mem_gb}\n", encoding="utf-8")


def write_plan_and_checks(
    spec: TaskSpec,
    workspace: Path,
    *,
    command: str,
    checks_override: str | None,
    dry_run: bool,
    deadline: float | None = None,
) -> bool:
    """Write workspace plan and checks. Returns False if *deadline* is exhausted."""
    plan = workspace / "plan.md"
    if not dry_run:
        shutil.copyfile(spec.instruction, plan)
    malvin_dir = workspace / ".malvin"
    if not dry_run:
        malvin_dir.mkdir(parents=True, exist_ok=True)
    checks = checks_override
    if checks is None:
        if deadline is not None and _remaining_sec(deadline) <= 0:
            return False
        checks = discover_deepswe_checks(workspace)
    if not dry_run:
        if deadline is not None and _remaining_sec(deadline) <= 0:
            return False
        patch_targets = patch_surface_targets(workspace)
        if patch_targets:
            probe_path = malvin_dir / "patch_surface_probe.py"
            probe_path.write_text(
                render_patch_surface_probe(patch_targets),
                encoding="utf-8",
            )
            probe_path.chmod(probe_path.stat().st_mode | 0o111)
            if _PATCH_SURFACE_PROBE_COMMAND not in checks:
                checks = checks.rstrip() + f"\n{_PATCH_SURFACE_PROBE_COMMAND}\n"
    if not checks.endswith("\n"):
        checks += "\n"
    checks_path = malvin_dir / "checks"
    click.echo(f"Writing {checks_path}: {checks.strip()!r}")
    if not dry_run:
        checks_path.write_text(checks, encoding="utf-8")
    return True


def apply_patch(workspace: Path, patch: Path, *, dry_run: bool) -> None:
    run_cmd(["git", "apply", "--whitespace=nowarn", str(patch)], cwd=workspace, dry_run=dry_run)


def harbor_test_patch_path(spec: TaskSpec) -> Path | None:
    path = spec.tests_dir / "test.patch"
    return path if path.is_file() else None


_HARBOR_NEW_MODE_PYTEST_RE = re.compile(
    r'elif\s*\[\s*"\$MODE"\s*=\s*"new"\s*\];\s*then\s*\n\s*(?P<cmd>[^\n]+)',
    re.MULTILINE,
)


def harbor_new_tests_check_line(workspace: Path) -> str | None:
    """Return the pytest command from workspace ``test.sh`` new mode, if present."""
    test_sh = workspace / "test.sh"
    if not test_sh.is_file():
        return None
    match = _HARBOR_NEW_MODE_PYTEST_RE.search(test_sh.read_text(encoding="utf-8"))
    if not match:
        return None
    cmd = match.group("cmd").strip()
    if not cmd or cmd.startswith("echo"):
        return None
    return cmd


def apply_harbor_test_patch(spec: TaskSpec, workspace: Path, *, dry_run: bool) -> bool:
    """Apply Harbor ``test.patch`` so the agent can run hidden integration tests."""
    patch_path = harbor_test_patch_path(spec)
    if patch_path is None:
        return False
    click.echo(f"Applying Harbor test patch: {patch_path}")
    apply_patch(workspace, patch_path, dry_run=dry_run)
    return True


def resolve_docker_image(
    spec: TaskSpec,
    image_override: str | None,
    *,
    dry_run: bool = False,
) -> str:
    if image_override:
        return image_override
    if dry_run:
        return spec.docker_image
    probe = subprocess.run(
        ["docker", "image", "inspect", spec.docker_image],
        capture_output=True,
        text=True,
    )
    if probe.returncode == 0:
        return spec.docker_image
    local_tag = f"deepswe-{spec.task_id}:local"
    probe_local = subprocess.run(
        ["docker", "image", "inspect", local_tag],
        capture_output=True,
        text=True,
    )
    if probe_local.returncode == 0:
        click.echo(f"Using locally built image {local_tag}")
        return local_tag
    if not spec.dockerfile.is_file():
        raise click.ClickException(
            f"Docker image {spec.docker_image!r} not present and no Dockerfile at {spec.dockerfile}"
        )
    click.echo(f"Building local image {local_tag} from {spec.dockerfile} (this may take several minutes)...")
    run_cmd(
        [
            "docker",
            "build",
            "-t",
            local_tag,
            "-f",
            str(spec.dockerfile),
            str(spec.dockerfile.parent),
        ],
    )
    return local_tag


def local_agent_image_tag(task_id: str) -> str:
    return f"deepswe-{task_id}:agent"


def _toolchain_copy_ignore(src: str, names: list[str], *, extra: tuple[str, ...]) -> set[str]:
    skip = {".git", "target", "__pycache__", ".cargo", "experiments", "results", "reports"}
    skip.update(extra)
    return {name for name in names if name in skip}


def _copy_toolchain_tree(src: Path, dst: Path, *, extra_skip: tuple[str, ...] = ()) -> None:
    ignore = lambda directory, names: _toolchain_copy_ignore(  # noqa: E731
        directory, names, extra=extra_skip
    )
    shutil.copytree(src, dst, ignore=ignore, dirs_exist_ok=True)


def build_local_agent_image(
    spec: TaskSpec,
    base_image: str,
    *,
    malvin_repo: Path,
    dry_run: bool,
) -> str:
    """Extend the Harbor base image with Linux malvin and cursor-agent."""
    agent_tag = local_agent_image_tag(spec.task_id)
    if not dry_run:
        probe = subprocess.run(
            ["docker", "image", "inspect", agent_tag],
            capture_output=True,
            text=True,
        )
        if probe.returncode == 0:
            click.echo(f"Using local agent image {agent_tag}")
            return agent_tag
    if dry_run:
        click.echo(f"Would build local agent image {agent_tag} from {base_image}")
        return agent_tag
    dockerfile = f"""\
FROM {base_image}
RUN apt-get update -qq && DEBIAN_FRONTEND=noninteractive apt-get install -y -qq \\
    curl build-essential pkg-config libssl-dev python3-pip
RUN pip3 install --break-system-packages click
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${{PATH}}"
COPY malvin {MALVIN_TOOLCHAIN_REMOTE}
RUN RUSTC_WRAPPER= cargo install --path {MALVIN_TOOLCHAIN_REMOTE} --locked
RUN curl -fsSL https://cursor.com/install | bash
ENV PATH="{TOOLCHAIN_PATH}"
"""
    click.echo(
        f"Building local agent image {agent_tag} from {base_image} "
        "(malvin/cursor-agent; may take several minutes)..."
    )
    with tempfile.TemporaryDirectory(prefix="deepswe-agent-") as tmp:
        build_dir = Path(tmp)
        (build_dir / "Dockerfile").write_text(dockerfile, encoding="utf-8")
        _copy_toolchain_tree(
            malvin_repo,
            build_dir / "malvin",
            extra_skip=(".malvin", ".kissignore"),
        )
        run_cmd(["docker", "build", "-t", agent_tag, str(build_dir)])
    return agent_tag


def cursor_env_docker_args() -> list[str]:
    args: list[str] = []
    for key in CURSOR_ENV_KEYS:
        value = os.environ.get(key)
        if value:
            args.extend(["-e", f"{key}={value}"])
    return args


def _docker_common_mounts(
    *,
    workspace: Path,
    run_root: Path,
    deepswe_run_py: Path,
) -> list[str]:
    """Volume mounts shared by both agent and grade containers."""
    logs_mount = run_root / "verifier_logs"
    return [
        "-v", f"{workspace.resolve()}:/app",
        "-v", f"{deepswe_run_py.resolve()}:{DEEPSWE_RUN_REMOTE}:ro",
        "-v", f"{deepswe_run_py.resolve().parent / 'sandbox_prep.py'}:{SANDBOX_PREP_REMOTE}:ro",
        "-v", f"{deepswe_run_py.resolve().parent / 'toolchain_repos.py'}:{TOOLCHAIN_REPOS_REMOTE}:ro",
        "-v", f"{logs_mount.resolve()}:/logs",
        "-v", f"{run_root.resolve()}:/run",
    ]


def _docker_pip_preamble() -> str:
    return (
        "pip3 install --break-system-packages click >/dev/null 2>&1 || "
        "pip install --break-system-packages click >/dev/null 2>&1 || true; "
    )


def docker_local_agent_cmd(
    *,
    image: str,
    spec: TaskSpec,
    task_dir: Path,
    workspace: Path,
    run_root: Path,
    deepswe_run_py: Path,
    malvin_command: str,
    malvin_args: tuple[str, ...],
    reset_workspace_flag: bool,
    checks_override: str | None,
) -> list[str]:
    """Docker command for the agent phase — no /tests or /task/solution mounted."""
    inner = [
        "python3", DEEPSWE_RUN_REMOTE, "run",
        "--task", "/task",
        "--workspace", "/app",
        "--runtime", "in-sandbox",
        "--skip-materialize",
        "--results-dir", "/run",
        "--skip-grade",
    ]
    if reset_workspace_flag:
        inner.append("--reset")
    if checks_override:
        inner.extend(["--checks", checks_override])
    inner.extend(["--command", malvin_command, *malvin_args])
    shell = _docker_pip_preamble() + " ".join(inner)
    task_dir_resolved = task_dir.resolve()
    return [
        "docker", "run", "--rm",
        *cursor_env_docker_args(),
        *_docker_common_mounts(workspace=workspace, run_root=run_root, deepswe_run_py=deepswe_run_py),
        "-v", f"{(task_dir_resolved / 'task.toml')}:/task/task.toml:ro",
        "-v", f"{(task_dir_resolved / 'instruction.md')}:/task/instruction.md:ro",
        "-v", f"{(task_dir_resolved / 'environment')}:/task/environment:ro",
        "-w", "/app",
        image,
        "bash", "-lc", shell,
    ]


def docker_local_grade_cmd(
    *,
    image: str,
    spec: TaskSpec,
    task_dir: Path,
    workspace: Path,
    run_root: Path,
    deepswe_run_py: Path,
    apply_solution: bool,
    reset_workspace_flag: bool,
) -> list[str]:
    """Docker command for the grade phase — /tests and /task/solution now available."""
    inner = [
        "python3", DEEPSWE_RUN_REMOTE, "run",
        "--task", "/task",
        "--workspace", "/app",
        "--runtime", "in-sandbox",
        "--skip-materialize",
        "--results-dir", "/run",
        "--grade-only",
    ]
    if apply_solution:
        inner.append("--apply-solution")
    if reset_workspace_flag:
        inner.append("--reset")
    shell = _docker_pip_preamble() + " ".join(inner)
    return [
        "docker", "run", "--rm",
        *_docker_common_mounts(workspace=workspace, run_root=run_root, deepswe_run_py=deepswe_run_py),
        "-v", f"{spec.tests_dir.resolve()}:/tests:ro",
        "-v", f"{task_dir.resolve()}:/task:ro",
        "-w", "/app",
        image,
        "bash", "-lc", shell,
    ]


def docker_local_eval_cmd(
    *,
    image: str,
    spec: TaskSpec,
    task_dir: Path,
    workspace: Path,
    run_root: Path,
    deepswe_run_py: Path,
    malvin_command: str,
    malvin_args: tuple[str, ...],
    grade_only: bool,
    skip_grade: bool,
    apply_solution: bool,
    reset_workspace_flag: bool,
    checks_override: str | None,
) -> list[str]:
    """Legacy single-container command. Used only by grade-only path."""
    inner = [
        "python3", DEEPSWE_RUN_REMOTE, "run",
        "--task", "/task",
        "--workspace", "/app",
        "--runtime", "in-sandbox",
        "--skip-materialize",
        "--results-dir", "/run",
    ]
    if grade_only:
        inner.append("--grade-only")
    if skip_grade:
        inner.append("--skip-grade")
    if apply_solution:
        inner.append("--apply-solution")
    if reset_workspace_flag:
        inner.append("--reset")
    if checks_override:
        inner.extend(["--checks", checks_override])
    if not grade_only:
        inner.extend(["--command", malvin_command, *malvin_args])
    shell = _docker_pip_preamble() + " ".join(inner)
    return [
        "docker", "run", "--rm",
        *cursor_env_docker_args(),
        "-v", f"{workspace.resolve()}:/app",
        "-v", f"{spec.tests_dir.resolve()}:/tests:ro",
        "-v", f"{task_dir.resolve()}:/task:ro",
        "-v", f"{deepswe_run_py.resolve()}:{DEEPSWE_RUN_REMOTE}:ro",
        "-v", f"{deepswe_run_py.resolve().parent / 'sandbox_prep.py'}:{SANDBOX_PREP_REMOTE}:ro",
        "-v", f"{deepswe_run_py.resolve().parent / 'toolchain_repos.py'}:{TOOLCHAIN_REPOS_REMOTE}:ro",
        "-v", f"{(run_root / 'verifier_logs').resolve()}:/logs",
        "-v", f"{run_root.resolve()}:/run",
        "-w", "/app",
        image,
        "bash", "-lc", shell,
    ]


def _read_docker_grade_result(run_root: Path, proc: subprocess.CompletedProcess[str]) -> dict[str, Any]:
    metadata_path = run_root / "metadata.json"
    if metadata_path.is_file():
        metadata = json.loads(metadata_path.read_text(encoding="utf-8"))
        return metadata.get("grade") or {}
    reward_path = run_root / "verifier_logs" / "verifier" / "reward.txt"
    reward: int | None = None
    if reward_path.is_file():
        text = reward_path.read_text(encoding="utf-8").strip()
        if text in {"0", "1"}:
            reward = int(text)
    return {
        "pass": reward == 1,
        "reward": reward,
        "verifier_exit_code": proc.returncode,
    }


def _run_local_docker_subprocess(
    cmd: list[str],
    *,
    timeout_sec: float,
) -> subprocess.CompletedProcess[str] | None:
    """Run a local Docker command with a host-side backstop timeout."""
    try:
        return subprocess.run(cmd, text=True, check=False, timeout=timeout_sec)
    except subprocess.TimeoutExpired:
        click.echo(
            f"local Docker backstop timed out after {timeout_sec:.0f}s",
            err=True,
        )
        return None


def run_local_eval_in_docker(
    spec: TaskSpec,
    task_dir: Path,
    workspace: Path,
    run_root: Path,
    *,
    malvin_command: str,
    malvin_args: tuple[str, ...],
    grade_only: bool,
    skip_grade: bool,
    apply_solution: bool,
    reset_workspace_flag: bool,
    checks_override: str | None,
    docker_image: str | None,
    dry_run: bool,
) -> dict[str, Any]:
    """Run agent and grade in separate local Docker containers.

    The agent container does not mount ``/tests`` or ``/task/solution``.
    After the agent finishes, a second container mounts the verifier
    files and runs ``--grade-only``.
    """
    base_image = resolve_docker_image(spec, docker_image, dry_run=dry_run)
    deepswe_run_py = Path(__file__).resolve()

    agent_result: dict[str, Any] | None = None
    grade_result: dict[str, Any]
    last_exit_code = 0

    if grade_only:
        click.echo("Running local Docker grade-only...")
        if deepswe_test_fast_grade_enabled():
            logs_dir = run_root / "verifier_logs"
            grade_result = _fast_grade_selftest_result(logs_dir)
            metadata = _build_run_metadata(
                spec, workspace, "local-docker", malvin_command, malvin_args,
                None, grade_result, None, grade_only=True, docker_image=base_image,
            )
            metadata["sandbox_prep"] = {"fast_selftest_stub": True}
            _write_host_run_artifacts(
                run_root, metadata, grade_result, logs_dir,
                dry_run=False, overwrite_artifacts=True,
            )
            return {"agent": None, "grade": grade_result, "runtime": "local-docker", "docker_exit_code": 0}
        cmd = docker_local_grade_cmd(
            image=base_image, spec=spec, task_dir=task_dir,
            workspace=workspace, run_root=run_root, deepswe_run_py=deepswe_run_py,
            apply_solution=apply_solution, reset_workspace_flag=reset_workspace_flag,
        )
        if dry_run:
            run_cmd(cmd, dry_run=True)
            return {"agent": None, "grade": {"pass": None, "reward": None, "dry_run": True}, "runtime": "local-docker"}
        proc = _run_local_docker_subprocess(
            cmd,
            timeout_sec=_docker_backstop_timeout_sec(spec.verifier_timeout_sec),
        )
        if proc is None:
            last_exit_code = TIMEOUT_EXIT_CODE
            grade_result = _grade_timeout_result(spec)
        else:
            last_exit_code = proc.returncode
            grade_result = _read_docker_grade_result(run_root, proc)
    else:
        malvin_repo = validate_toolchain_repos()
        eval_image = build_local_agent_image(
            spec, base_image, malvin_repo=malvin_repo, dry_run=dry_run,
        )
        agent_cmd = docker_local_agent_cmd(
            image=eval_image, spec=spec, task_dir=task_dir,
            workspace=workspace, run_root=run_root, deepswe_run_py=deepswe_run_py,
            malvin_command=malvin_command, malvin_args=malvin_args,
            reset_workspace_flag=reset_workspace_flag, checks_override=checks_override,
        )
        click.echo("Running local Docker agent (no /tests or /task/solution mounted)...")
        if dry_run:
            run_cmd(agent_cmd, dry_run=True)
            agent_result = {"dry_run": True}
        else:
            proc = _run_local_docker_subprocess(
                agent_cmd,
                timeout_sec=_docker_backstop_timeout_sec(spec.agent_timeout_sec),
            )
            if proc is None:
                last_exit_code = TIMEOUT_EXIT_CODE
                agent_result = _agent_timeout_result(spec)
            else:
                last_exit_code = proc.returncode
                metadata_path = run_root / "metadata.json"
                if metadata_path.is_file():
                    agent_result = json.loads(metadata_path.read_text(encoding="utf-8")).get("agent")
                else:
                    agent_result = {"exit_code": proc.returncode}

        if skip_grade:
            grade_result = {"pass": None, "reward": None, "skipped": True}
        else:
            grade_cmd = docker_local_grade_cmd(
                image=base_image, spec=spec, task_dir=task_dir,
                workspace=workspace, run_root=run_root, deepswe_run_py=deepswe_run_py,
                apply_solution=apply_solution, reset_workspace_flag=False,
            )
            click.echo("Running local Docker grade...")
            if dry_run:
                run_cmd(grade_cmd, dry_run=True)
                grade_result = {"pass": None, "reward": None, "dry_run": True}
            else:
                proc = _run_local_docker_subprocess(
                    grade_cmd,
                    timeout_sec=_docker_backstop_timeout_sec(spec.verifier_timeout_sec),
                )
                if proc is None:
                    last_exit_code = TIMEOUT_EXIT_CODE
                    grade_result = _grade_timeout_result(spec)
                else:
                    last_exit_code = proc.returncode
                    grade_result = _read_docker_grade_result(run_root, proc)

    return {
        "agent": agent_result,
        "grade": grade_result,
        "runtime": "local-docker",
        "docker_exit_code": last_exit_code,
    }


def grade_workspace_native(
    workspace: Path,
    test_sh: Path,
    logs_dir: Path,
    *,
    dry_run: bool,
    timeout_sec: float | None = None,
    configured_timeout_sec: float | None = None,
) -> dict[str, Any]:
    """Run Harbor ``test.sh`` in the current environment (no Docker wrapper)."""
    verifier_log = logs_dir / "verifier.log"
    cmd = ["bash", str(test_sh)]
    click.echo("Running Harbor verifier (in-sandbox)...")
    if dry_run:
        run_cmd(cmd, cwd=workspace, dry_run=True)
        return {"pass": None, "reward": None, "dry_run": True}
    if deepswe_test_fast_grade_enabled():
        return _fast_grade_selftest_result(logs_dir)
    logs_dir.mkdir(parents=True, exist_ok=True)
    (logs_dir / "verifier").mkdir(parents=True, exist_ok=True)
    (logs_dir / "artifacts").mkdir(parents=True, exist_ok=True)
    if timeout_sec is not None and timeout_sec <= 0:
        result = SubprocessResult(
            exit_code=TIMEOUT_EXIT_CODE,
            timed_out=True,
            elapsed_sec=0.0,
            output="",
        )
    elif timeout_sec is None:
        proc = subprocess.run(
            cmd,
            cwd=str(workspace),
            text=True,
            capture_output=True,
            check=False,
        )
        result = SubprocessResult(
            exit_code=int(proc.returncode or 0),
            timed_out=False,
            elapsed_sec=0.0,
            output=(proc.stdout or "") + (proc.stderr or ""),
        )
    else:
        result = _run_with_timeout(cmd, cwd=workspace, timeout_sec=timeout_sec, stream=False)
    verifier_log.write_text(result.output or "", encoding="utf-8")
    if result.output:
        sys.stdout.write(result.output)
    reward_path = logs_dir / "verifier" / "reward.txt"
    reward: int | None = None
    if not result.timed_out and reward_path.is_file():
        text = reward_path.read_text(encoding="utf-8").strip()
        if text in {"0", "1"}:
            reward = int(text)
    model_patch = logs_dir / "artifacts" / "model.patch"
    grade: dict[str, Any] = {
        "pass": reward == 1 if not result.timed_out else False,
        "reward": reward if not result.timed_out else 0,
        "verifier_exit_code": result.exit_code,
        "verifier_log": str(verifier_log),
        "model_patch": str(model_patch) if model_patch.is_file() else None,
    }
    if configured_timeout_sec is not None:
        grade["timeout_sec"] = configured_timeout_sec
    if result.timed_out:
        grade["timed_out"] = True
    return grade


def grade_workspace(
    spec: TaskSpec,
    workspace: Path,
    logs_dir: Path,
    *,
    image: str,
    dry_run: bool,
    timeout_sec: float | None = None,
    configured_timeout_sec: float | None = None,
) -> dict[str, Any]:
    logs_dir.mkdir(parents=True, exist_ok=True)
    (logs_dir / "verifier").mkdir(parents=True, exist_ok=True)
    (logs_dir / "artifacts").mkdir(parents=True, exist_ok=True)
    verifier_log = logs_dir / "verifier.log"
    cmd = [
        "docker",
        "run",
        "--rm",
        "-v",
        f"{workspace.resolve()}:/app",
        "-v",
        f"{spec.tests_dir.resolve()}:/tests:ro",
        "-v",
        f"{logs_dir.resolve()}:/logs",
        image,
        "bash",
        "/tests/test.sh",
    ]
    click.echo("Running Harbor verifier...")
    if dry_run:
        run_cmd(cmd, dry_run=True)
        return {"pass": None, "reward": None, "dry_run": True}
    if deepswe_test_fast_grade_enabled():
        return _fast_grade_selftest_result(logs_dir)
    if timeout_sec is not None and timeout_sec <= 0:
        result = SubprocessResult(
            exit_code=TIMEOUT_EXIT_CODE,
            timed_out=True,
            elapsed_sec=0.0,
            output="",
        )
    elif timeout_sec is None:
        proc = subprocess.run(cmd, text=True, capture_output=True, check=False)
        result = SubprocessResult(
            exit_code=int(proc.returncode or 0),
            timed_out=False,
            elapsed_sec=0.0,
            output=(proc.stdout or "") + (proc.stderr or ""),
        )
    else:
        result = _run_with_timeout(cmd, timeout_sec=timeout_sec, stream=False)
    verifier_log.write_text(result.output or "", encoding="utf-8")
    if result.output:
        sys.stdout.write(result.output)
    reward_path = logs_dir / "verifier" / "reward.txt"
    reward: int | None = None
    if not result.timed_out and reward_path.is_file():
        text = reward_path.read_text(encoding="utf-8").strip()
        if text in {"0", "1"}:
            reward = int(text)
    model_patch = logs_dir / "artifacts" / "model.patch"
    grade: dict[str, Any] = {
        "pass": reward == 1 if not result.timed_out else False,
        "reward": reward if not result.timed_out else 0,
        "verifier_exit_code": result.exit_code,
        "verifier_log": str(verifier_log),
        "model_patch": str(model_patch) if model_patch.is_file() else None,
    }
    if configured_timeout_sec is not None:
        grade["timeout_sec"] = configured_timeout_sec
    if result.timed_out:
        grade["timed_out"] = True
    return grade


def malvin_needs_task_plan(command: str) -> bool:
    """True when the agent phase reads task ``plan.md`` (``malvin code``)."""
    return command == "code"


def hello_probe_cmd(malvin_cmd: str, malvin_args: tuple[str, ...]) -> list[str]:
    """Argv for a one-turn Cursor connectivity probe with stdout tee."""
    return [malvin_cmd, "do", "Hello", *malvin_args]


def _relay_subprocess_stdout(cmd: list[str], *, cwd: Path) -> tuple[int, str]:
    """Run *cmd*, stream merged stdout/stderr to local stdout, return exit code and capture."""
    env = os.environ.copy()
    env.setdefault("MALVIN_FORCE_STDOUT_TEE", "1")
    proc = subprocess.Popen(
        cmd,
        cwd=str(cwd),
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
        bufsize=1,
        env=env,
    )
    chunks: list[str] = []
    if proc.stdout is not None:
        for line in proc.stdout:
            sys.stdout.write(line)
            sys.stdout.flush()
            chunks.append(line)
    proc.wait()
    return int(proc.returncode or 0), "".join(chunks)


def run_hello_probe_on_host(
    malvin_args: tuple[str, ...],
    *,
    dry_run: bool,
) -> None:
    """Run ``malvin do Hello`` on the host and relay agent stdout to local stdout."""
    cmd = hello_probe_cmd(MALVIN_CMD, malvin_args)
    click.echo(f"Running agent: {' '.join(cmd)}")
    if dry_run:
        run_cmd(cmd, cwd=Path.cwd(), dry_run=True)
        return
    exit_code, agent_stdout = _relay_subprocess_stdout(cmd, cwd=Path.cwd())
    click.echo("\n=== Hello probe ===")
    click.echo(f"malvin exit: {exit_code}")
    if agent_stdout.strip():
        click.echo("--- agent stdout ---")
        click.echo(agent_stdout.rstrip())
        click.echo("--- end agent stdout ---")
    if exit_code != 0:
        raise SystemExit(exit_code)


def run_malvin(
    workspace: Path,
    *,
    command: str,
    malvin_args: tuple[str, ...],
    dry_run: bool,
    timeout_sec: float | None = None,
    configured_timeout_sec: float | None = None,
) -> dict[str, Any]:
    if command == "do":
        if not malvin_args:
            raise click.ClickException("malvin do requires a prompt argument")
        cmd = [MALVIN_CMD, "do", *malvin_args]
    elif command == "hello":
        cmd = hello_probe_cmd(MALVIN_CMD, malvin_args)
    else:
        plan = workspace / "plan.md"
        if not dry_run and not plan.is_file():
            raise click.ClickException(f"Missing plan.md in workspace: {plan}")
        cmd = [MALVIN_CMD, command, plan.name, *malvin_args]
    click.echo(f"Running agent: {' '.join(cmd)}")
    t0 = time.monotonic()
    if dry_run:
        run_cmd(cmd, cwd=workspace, dry_run=True)
        return {"agent_seconds": 0.0, "exit_code": 0, "dry_run": True}
    if timeout_sec is not None and timeout_sec <= 0:
        elapsed = time.monotonic() - t0
        result: dict[str, Any] = {
            "agent_seconds": elapsed,
            "exit_code": TIMEOUT_EXIT_CODE,
            "timed_out": True,
        }
        if configured_timeout_sec is not None:
            result["timeout_sec"] = configured_timeout_sec
        return result
    if command == "hello":
        env = os.environ.copy()
        env.setdefault("MALVIN_FORCE_STDOUT_TEE", "1")
        if timeout_sec is None:
            exit_code, agent_stdout = _relay_subprocess_stdout(cmd, cwd=workspace)
            elapsed = time.monotonic() - t0
            return {
                "agent_seconds": elapsed,
                "exit_code": exit_code,
                "stdout": agent_stdout,
            }
        sub = _run_with_timeout(
            cmd,
            cwd=workspace,
            timeout_sec=timeout_sec,
            stream=True,
            env=env,
        )
        return {
            "agent_seconds": sub.elapsed_sec,
            "exit_code": sub.exit_code,
            "stdout": sub.output or "",
            "timed_out": sub.timed_out,
            "timeout_sec": configured_timeout_sec,
        }
    if timeout_sec is None:
        proc = subprocess.run(cmd, cwd=str(workspace), check=False)
        elapsed = time.monotonic() - t0
        return {"agent_seconds": elapsed, "exit_code": proc.returncode}
    sub = _run_with_timeout(
        cmd,
        cwd=workspace,
        timeout_sec=timeout_sec,
        inherit_stdio=True,
    )
    agent: dict[str, Any] = {
        "agent_seconds": sub.elapsed_sec,
        "exit_code": sub.exit_code,
        "timed_out": sub.timed_out,
    }
    if configured_timeout_sec is not None:
        agent["timeout_sec"] = configured_timeout_sec
    return agent


def find_latest_malvin_log(workspace: Path | None = None) -> Path | None:
    logs_root = (workspace or Path.cwd()) / ".malvin" / "logs"
    if not logs_root.is_dir():
        return None
    candidates = sorted(logs_root.iterdir(), key=lambda p: p.stat().st_mtime, reverse=True)
    return candidates[0] if candidates else None


def write_metadata(out_dir: Path, payload: dict[str, Any]) -> None:
    out_dir.mkdir(parents=True, exist_ok=True)
    (out_dir / "metadata.json").write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")


def run_modal_solve(
    *,
    task_dir: Path,
    malvin_command: str = "code",
    checks_override: str | None,
    skip_grade: bool,
    apply_solution: bool,
    reset_workspace_flag: bool,
    dry_run: bool,
    malvin_args: tuple[str, ...],
) -> None:
    """Dispatch task-name solves to Modal (lazy import keeps self-test Modal-free)."""
    try:
        from deepswe_modal import require_cursor_credentials_for_agent, run_modal_eval
    except ModuleNotFoundError as exc:
        raise click.ClickException(
            "Modal runtime requires the modal package (pip install modal). "
            "Use --local for local Docker instead."
        ) from exc

    require_cursor_credentials_for_agent(grade_only=False)
    run_modal_eval(
        task_dir=task_dir,
        malvin_command=malvin_command,
        checks_override=checks_override,
        grade_only=False,
        skip_grade=skip_grade,
        apply_solution=apply_solution,
        reset_flag=reset_workspace_flag,
        malvin_args=malvin_args,
        dry_run=dry_run,
    )


def _is_modal_spend_limit_error(exc: BaseException) -> bool:
    """True when Modal rejected work due to workspace billing / spend caps."""
    try:
        import modal.exception

        if isinstance(exc, modal.exception.ResourceExhaustedError):
            return True
        if isinstance(exc, modal.exception.RemoteError):
            message = str(exc).lower()
            return "spend limit" in message or "billing cycle" in message
    except ModuleNotFoundError:
        pass
    message = str(exc).lower()
    return "spend limit" in message or "billing cycle spend limit" in message


def _run_solve_local_docker_fallback(
    task_dir: Path,
    *,
    malvin_command: str = "code",
    checks_override: str | None,
    skip_grade: bool,
    apply_solution: bool,
    reset_workspace_flag: bool,
    docker_image: str | None,
    dry_run: bool,
    malvin_args: tuple[str, ...],
) -> None:
    """Run solve in local Docker when Modal billing blocks sandbox creation."""
    spec = parse_task_dir(task_dir)
    validate_verifier_paths(spec)
    results_root = default_deepswe_results_dir()
    run_root = results_root / spec.task_id / timestamp_dir()
    workspace = results_root / spec.task_id / "workspace"
    click.echo(f"Task: {spec.task_id}")
    click.echo("Runtime: local-docker (Modal spend-limit fallback)")
    click.echo(f"Workspace: {workspace.resolve()}")
    click.echo(f"Run artifacts: {run_root.resolve()}")
    _run_local_docker_task(
        spec,
        task_dir,
        workspace,
        run_root,
        malvin_command=malvin_command,
        malvin_args=malvin_args,
        grade_only=False,
        skip_grade=skip_grade,
        apply_solution=apply_solution,
        reset_workspace_flag=reset_workspace_flag,
        checks_override=checks_override,
        docker_image=docker_image,
        dry_run=dry_run,
    )


def _build_run_metadata(
    spec: TaskSpec,
    workspace: Path,
    runtime: str,
    malvin_command: str,
    malvin_args: tuple[str, ...],
    agent_result: dict[str, Any] | None,
    grade_result: dict[str, Any],
    malvin_log: Path | None,
    *,
    grade_only: bool,
    docker_image: str | None = None,
) -> dict[str, Any]:
    return {
        "task_id": spec.task_id,
        "task_dir": str(spec.task_dir),
        "workspace": str(workspace.resolve()),
        "runtime": runtime,
        "malvin_command": malvin_command if not grade_only else None,
        "malvin_args": list(malvin_args),
        "base_commit": spec.base_commit,
        "docker_image": docker_image if docker_image is not None else spec.docker_image,
        "agent": agent_result,
        "grade": grade_result,
        "malvin_log_dir": str(malvin_log.resolve()) if malvin_log else None,
        "timestamp_utc": timestamp_dir(),
    }


def _write_host_run_artifacts(
    run_root: Path,
    metadata: dict[str, Any],
    grade_result: dict[str, Any],
    logs_dir: Path,
    *,
    dry_run: bool,
    skip_metadata_if_exists: bool = False,
    overwrite_artifacts: bool = False,
) -> None:
    if dry_run:
        return
    host_metadata = run_root / "metadata.json"
    if not (skip_metadata_if_exists and host_metadata.is_file()):
        write_metadata(run_root, metadata)
    reward = grade_result.get("reward")
    reward_dst = run_root / "reward.txt"
    reward_src = logs_dir / "verifier" / "reward.txt"
    if reward is not None and reward_src.is_file():
        if overwrite_artifacts or not reward_dst.is_file():
            shutil.copyfile(reward_src, reward_dst)
    mp = grade_result.get("model_patch")
    mp_host = run_root / "model.patch"
    if mp and Path(mp).is_file():
        if overwrite_artifacts or not mp_host.is_file():
            shutil.copyfile(mp, mp_host)


def _print_evaluation_summary(
    grade_result: dict[str, Any],
    agent_result: dict[str, Any] | None,
    run_root: Path,
) -> None:
    click.echo("\n=== Evaluation ===")
    click.echo(f"reward: {grade_result.get('reward')}")
    click.echo(f"pass: {grade_result.get('pass')}")
    if agent_result:
        click.echo(f"malvin exit: {agent_result.get('exit_code')}")
        if agent_result.get("timed_out"):
            click.echo("agent timed_out: true")
        click.echo(f"agent_seconds: {agent_result.get('agent_seconds', 0):.1f}")
        agent_stdout = agent_result.get("stdout")
        if isinstance(agent_stdout, str) and agent_stdout.strip():
            click.echo("--- agent stdout ---")
            click.echo(agent_stdout.rstrip())
            click.echo("--- end agent stdout ---")
    if grade_result.get("timed_out"):
        click.echo("grade timed_out: true")
    click.echo(f"artifacts: {run_root.resolve()}")


def _exit_from_evaluation(
    grade_result: dict[str, Any],
    agent_result: dict[str, Any] | None,
) -> None:
    if grade_result.get("pass") is False:
        raise SystemExit(1)
    if agent_result and not agent_result.get("timed_out"):
        if agent_result.get("exit_code") not in (0, None):
            raise SystemExit(agent_result["exit_code"])


def _run_local_docker_task(
    spec: TaskSpec,
    task_dir: Path,
    workspace: Path,
    run_root: Path,
    *,
    malvin_command: str,
    malvin_args: tuple[str, ...],
    grade_only: bool,
    skip_grade: bool,
    apply_solution: bool,
    reset_workspace_flag: bool,
    checks_override: str | None,
    docker_image: str | None,
    dry_run: bool,
) -> None:
    local_result = run_local_eval_in_docker(
        spec,
        task_dir,
        workspace,
        run_root,
        malvin_command=malvin_command,
        malvin_args=malvin_args,
        grade_only=grade_only,
        skip_grade=skip_grade,
        apply_solution=apply_solution,
        reset_workspace_flag=reset_workspace_flag or apply_solution,
        checks_override=checks_override,
        docker_image=docker_image,
        dry_run=dry_run,
    )
    agent_result = local_result.get("agent")
    grade_result = local_result.get("grade") or {}
    malvin_log = find_latest_malvin_log(workspace)
    metadata = _build_run_metadata(
        spec,
        workspace,
        "local-docker",
        malvin_command,
        malvin_args,
        agent_result,
        grade_result,
        malvin_log,
        grade_only=grade_only,
    )
    logs_dir = run_root / "verifier_logs"
    _write_host_run_artifacts(
        run_root,
        metadata,
        grade_result,
        logs_dir,
        dry_run=dry_run,
        skip_metadata_if_exists=True,
    )
    _print_evaluation_summary(grade_result, agent_result, run_root)
    _exit_from_evaluation(grade_result, agent_result)


def run_task(
    *,
    local_task_name: str | None,
    task_dir: Path | None,
    workspace: Path | None,
    results_dir: Path | None,
    malvin_command: str,
    checks_override: str | None,
    runtime: str,
    skip_materialize: bool,
    grade_only: bool,
    skip_grade: bool,
    apply_solution: bool,
    reset_workspace_flag: bool,
    docker_image: str | None,
    dry_run: bool,
    malvin_args: tuple[str, ...],
    extra_args: tuple[str, ...] = (),
    use_local_docker: bool = False,
) -> None:
    """Run malvin on a DeepSWE task and grade with Harbor ``tests/test.sh``."""
    if extra_args:
        malvin_args = malvin_args + extra_args
    local_docker = False
    if local_task_name:
        if task_dir is not None:
            raise click.ClickException("Use either solve TASK_NAME or --task, not both")
        task_dir = resolve_local_task_dir(local_task_name)
        if use_local_docker:
            local_docker = True
        else:
            try:
                run_modal_solve(
                    task_dir=task_dir,
                    malvin_command=malvin_command,
                    checks_override=checks_override,
                    skip_grade=skip_grade,
                    apply_solution=apply_solution,
                    reset_workspace_flag=reset_workspace_flag,
                    dry_run=dry_run,
                    malvin_args=malvin_args,
                )
            except Exception as exc:
                if not _is_modal_spend_limit_error(exc):
                    raise
                click.echo(
                    "Modal workspace spend limit reached; falling back to local Docker.",
                    err=True,
                )
                _run_solve_local_docker_fallback(
                    task_dir,
                    malvin_command=malvin_command,
                    checks_override=checks_override,
                    skip_grade=skip_grade,
                    apply_solution=apply_solution,
                    reset_workspace_flag=reset_workspace_flag,
                    docker_image=docker_image,
                    dry_run=dry_run,
                    malvin_args=malvin_args,
                )
            return
    elif task_dir is None:
        raise click.ClickException("Provide solve TASK_NAME or run --task PATH")
    in_sandbox = runtime == "in-sandbox"
    spec = parse_task_dir(task_dir)
    results_root = results_dir or default_deepswe_results_dir()
    run_root = results_root if in_sandbox else results_root / spec.task_id / timestamp_dir()
    workspace = workspace or (results_root / spec.task_id / "workspace")
    logs_dir = (run_root / "verifier_logs") if not in_sandbox else IN_SANDBOX_LOGS_DIR
    click.echo(f"Task: {spec.task_id}")
    click.echo(f"Runtime: {'local-docker' if local_docker else runtime}")
    click.echo(f"Workspace: {workspace.resolve()}")
    click.echo(f"Run artifacts: {run_root.resolve()}")

    if not skip_materialize:
        materialize_workspace(spec, workspace, dry_run=dry_run)

    if apply_solution and spec.solution_patch is None:
        raise click.ClickException(f"No solution patch at {spec.task_dir / 'solution'}")

    if local_docker:
        _run_local_docker_task(
            spec,
            task_dir,
            workspace,
            run_root,
            malvin_command=malvin_command,
            malvin_args=malvin_args,
            grade_only=grade_only,
            skip_grade=skip_grade,
            apply_solution=apply_solution,
            reset_workspace_flag=reset_workspace_flag,
            checks_override=checks_override,
            docker_image=docker_image,
            dry_run=dry_run,
        )
        return

    if reset_workspace_flag or apply_solution:
        reset_workspace(spec, workspace, dry_run=dry_run)

    if apply_solution:
        click.echo(f"Applying reference solution: {spec.solution_patch}")
        apply_patch(workspace, spec.solution_patch, dry_run=dry_run)

    checks_text = ""
    if not grade_only and malvin_needs_task_plan(malvin_command):
        checks_text = checks_override or discover_deepswe_checks(workspace)

    prep_deadline: float | None = None
    if grade_only:
        prep_deadline = time.monotonic() + spec.verifier_timeout_sec
    elif not grade_only:
        prep_deadline = time.monotonic() + spec.agent_timeout_sec

    prep_result = prepare_task_sandbox(
        spec,
        workspace,
        checks=checks_text,
        dry_run=dry_run,
        deadline=prep_deadline,
    )

    agent_result: dict[str, Any] | None = None
    if grade_only and prep_result.timed_out:
        agent_result = None
    elif not grade_only:
        if prep_result.timed_out:
            agent_result = _agent_timeout_result(spec)
        else:
            agent_deadline = prep_deadline
            assert agent_deadline is not None
            agent_timed_out = False
            if malvin_needs_task_plan(malvin_command):
                if _remaining_sec(agent_deadline) <= 0:
                    agent_timed_out = True
                else:
                    if not write_plan_and_checks(
                        spec,
                        workspace,
                        command=malvin_command,
                        checks_override=checks_override,
                        dry_run=dry_run,
                        deadline=agent_deadline,
                    ):
                        agent_timed_out = True
            if not agent_timed_out:
                if _remaining_sec(agent_deadline) <= 0:
                    agent_timed_out = True
                else:
                    ensure_deepswe_malvin_config(spec, dry_run=dry_run)
                    if _remaining_sec(agent_deadline) <= 0:
                        agent_timed_out = True
            if not agent_timed_out:
                remaining = _remaining_sec(agent_deadline)
                if remaining <= 0:
                    agent_timed_out = True
                else:
                    agent_result = run_malvin(
                        workspace,
                        command=malvin_command,
                        malvin_args=malvin_args,
                        dry_run=dry_run,
                        timeout_sec=remaining,
                        configured_timeout_sec=spec.agent_timeout_sec,
                    )
                    if agent_result.get("timed_out"):
                        agent_timed_out = True
            if agent_timed_out and agent_result is None:
                agent_result = _agent_timeout_result(spec)

    grade_result: dict[str, Any]
    if skip_grade:
        grade_result = {"pass": None, "reward": None, "skipped": True}
    elif grade_only and prep_result.timed_out:
        grade_result = _grade_timeout_result(spec)
    else:
        if grade_only:
            assert prep_deadline is not None
            remaining = _remaining_sec(prep_deadline)
        else:
            verifier_deadline = time.monotonic() + spec.verifier_timeout_sec
            remaining = _remaining_sec(verifier_deadline)
        if in_sandbox:
            test_sh = IN_SANDBOX_TESTS_DIR / "test.sh"
            grade_result = grade_workspace_native(
                workspace,
                test_sh,
                logs_dir,
                dry_run=dry_run,
                timeout_sec=remaining,
                configured_timeout_sec=spec.verifier_timeout_sec,
            )
        else:
            validate_verifier_paths(spec)
            image = resolve_docker_image(spec, docker_image, dry_run=dry_run)
            grade_result = grade_workspace(
                spec,
                workspace,
                logs_dir,
                image=image,
                dry_run=dry_run,
                timeout_sec=remaining,
                configured_timeout_sec=spec.verifier_timeout_sec,
            )

    malvin_log = find_latest_malvin_log(workspace)
    metadata = _build_run_metadata(
        spec,
        workspace,
        runtime,
        malvin_command,
        malvin_args,
        agent_result,
        grade_result,
        malvin_log,
        grade_only=grade_only,
        docker_image=spec.docker_image if not in_sandbox else None,
    )
    metadata["sandbox_prep"] = prep_result.as_dict()
    _write_host_run_artifacts(run_root, metadata, grade_result, logs_dir, dry_run=dry_run, overwrite_artifacts=True)
    _print_evaluation_summary(grade_result, agent_result, run_root)
    _exit_from_evaluation(grade_result, agent_result)


def _task_kernel_options(f: Any) -> Any:
    """Click options for the path-based ``run`` subcommand."""
    f = click.option(
        "--task",
        "task_dir",
        type=click.Path(exists=True, file_okay=False, path_type=Path),
        default=None,
        help="Path to a DeepSWE task directory (contains task.toml).",
    )(f)
    f = click.option(
        "--workspace",
        type=click.Path(file_okay=False, path_type=Path),
        default=None,
        help="Git checkout for the task repo (default: <results-dir>/<task-id>/workspace).",
    )(f)
    f = click.option(
        "--results-dir",
        type=click.Path(file_okay=False, path_type=Path),
        default=None,
        show_default="~/.malvin_home/deepswe-results",
        help="Root directory for run artifacts (outside the malvin repo by default).",
    )(f)
    f = click.option(
        "--command",
        "malvin_command",
        type=click.Choice(["code", "do", "hello"]),
        default="code",
        show_default=True,
        help="malvin subcommand to run for the agent phase.",
    )(f)
    f = click.option(
        "--checks",
        "checks_override",
        default=None,
        help="Override .malvin/checks content (default: repo signals from pre-commit/Makefile/existing checks).",
    )(f)
    f = click.option(
        "--runtime",
        type=click.Choice(["host", "in-sandbox"]),
        default="host",
        show_default=True,
        help="host: malvin on host, grade via Docker; in-sandbox: agent+grade in current env.",
    )(f)
    f = click.option(
        "--skip-materialize",
        is_flag=True,
        help="Do not clone/checkout workspace (already provisioned, e.g. Modal mount).",
    )(f)
    f = click.option(
        "--skip-grade",
        is_flag=True,
        help="Skip Harbor verifier grading (agent phase only).",
    )(f)
    f = click.option(
        "--grade-only",
        is_flag=True,
        help="Skip agent; grade the current workspace tree.",
    )(f)
    f = click.option(
        "--apply-solution",
        is_flag=True,
        help="Apply task solution/solution.patch before agent or grade (harness sanity check).",
    )(f)
    f = click.option(
        "--reset",
        "reset_workspace_flag",
        is_flag=True,
        help="Hard reset workspace to base_commit before run.",
    )(f)
    f = click.option(
        "--docker-image",
        default=None,
        help="Override Harbor docker image tag.",
    )(f)
    f = click.option(
        "--dry-run",
        is_flag=True,
        help="Print commands without executing.",
    )(f)
    return f


def _local_solve_options(f: Any) -> Any:
    """Click options for the ``solve TASK_NAME`` subcommand."""
    f = click.option(
        "--local",
        "use_local_docker",
        is_flag=True,
        help="Run in a local Docker container instead of Modal (default: Modal).",
    )(f)
    f = click.option(
        "--checks",
        "checks_override",
        default=None,
        help="Override .malvin/checks content (default: repo signals from pre-commit/Makefile/existing checks).",
    )(f)
    f = click.option(
        "--skip-grade",
        is_flag=True,
        help="Skip Harbor verifier grading (agent phase only).",
    )(f)
    f = click.option(
        "--apply-solution",
        is_flag=True,
        help="Apply task solution/solution.patch before grade (harness sanity check).",
    )(f)
    f = click.option(
        "--reset",
        "reset_workspace_flag",
        is_flag=True,
        help="Hard reset workspace to base_commit.",
    )(f)
    f = click.option(
        "--docker-image",
        default=None,
        help="Override Harbor docker image tag.",
    )(f)
    f = click.option(
        "--dry-run",
        is_flag=True,
        help="Print commands without executing.",
    )(f)
    f = click.argument("malvin_args", nargs=-1, type=click.UNPROCESSED)(f)
    return f


def _hello_options(f: Any) -> Any:
    """Click options for the ``hello TASK_NAME`` connectivity probe (no grading)."""
    f = click.option(
        "--host",
        "run_on_host",
        is_flag=True,
        help="Run malvin do Hello on this machine (no Modal/Docker/task workspace).",
    )(f)
    f = click.option(
        "--local",
        "use_local_docker",
        is_flag=True,
        help="Run in a local Docker container instead of Modal (default: Modal).",
    )(f)
    f = click.option(
        "--docker-image",
        default=None,
        help="Override Harbor docker image tag.",
    )(f)
    f = click.option(
        "--dry-run",
        is_flag=True,
        help="Print commands without executing.",
    )(f)
    f = click.argument("malvin_args", nargs=-1, type=click.UNPROCESSED)(f)
    return f


class TaskAliasGroup(click.Group):
    """Route ``deepswe_run.py TASK_NAME`` to ``solve TASK_NAME``."""

    def resolve_command(self, ctx, args):
        if args:
            token = click.utils.make_str(args[0])
            if token not in self.commands and not token.startswith("-"):
                return "solve", self.get_command(ctx, "solve"), args
        return super().resolve_command(ctx, args)


@click.group(cls=TaskAliasGroup)
def cli() -> None:
    """Run malvin on a DeepSWE task and grade with Harbor ``tests/test.sh``."""


@cli.command(
    "run",
    context_settings={
        "ignore_unknown_options": True,
        "allow_extra_args": True,
    },
)
@_task_kernel_options
@click.pass_context
def run_task_cli(
    ctx: click.Context,
    task_dir: Path | None,
    workspace: Path | None,
    results_dir: Path | None,
    malvin_command: str,
    checks_override: str | None,
    runtime: str,
    skip_materialize: bool,
    grade_only: bool,
    skip_grade: bool,
    apply_solution: bool,
    reset_workspace_flag: bool,
    docker_image: str | None,
    dry_run: bool,
) -> None:
    """Run malvin on a task directory (path-based harness entry point)."""
    if task_dir is None:
        raise click.ClickException("run requires --task PATH")
    run_task(
        local_task_name=None,
        task_dir=task_dir,
        workspace=workspace,
        results_dir=results_dir,
        malvin_command=malvin_command,
        checks_override=checks_override,
        runtime=runtime,
        skip_materialize=skip_materialize,
        grade_only=grade_only,
        skip_grade=skip_grade,
        apply_solution=apply_solution,
        reset_workspace_flag=reset_workspace_flag,
        docker_image=docker_image,
        dry_run=dry_run,
        malvin_args=(),
        extra_args=tuple(ctx.args),
    )


@cli.command("tasks")
def tasks_cmd() -> None:
    """List all available DeepSWE tasks."""
    tasks_root = default_deepswe_tasks_root()
    if not tasks_root.is_dir():
        raise click.ClickException(
            f"DeepSWE tasks directory not found: {tasks_root} "
            f"(set DEEPSWE_TASKS or clone deep-swe next to malvin)"
        )
    task_entries = list_deepswe_tasks_with_language()
    if not task_entries:
        raise click.ClickException(f"No DeepSWE tasks found under {tasks_root}")
    for task_id, language in task_entries:
        click.echo(f"{task_id}\t{language}")


@cli.command("self-test")
def self_test_cmd() -> None:
    """Run unit tests and exit (no task run)."""
    run_self_tests()


@cli.command("solve")
@click.argument("task_name")
@_local_solve_options
@click.pass_context
def solve(
    ctx: click.Context,
    task_name: str,
    use_local_docker: bool,
    checks_override: str | None,
    skip_grade: bool,
    apply_solution: bool,
    reset_workspace_flag: bool,
    docker_image: str | None,
    dry_run: bool,
    malvin_args: tuple[str, ...],
) -> None:
    """Run malvin code and Harbor grade (Modal by default; --local for Docker)."""
    reset_workspace_flag = True
    run_task(
        local_task_name=task_name,
        task_dir=None,
        workspace=None,
        results_dir=None,
        malvin_command="code",
        checks_override=checks_override,
        runtime="host",
        skip_materialize=False,
        grade_only=False,
        skip_grade=skip_grade,
        apply_solution=apply_solution,
        reset_workspace_flag=reset_workspace_flag,
        docker_image=docker_image,
        dry_run=dry_run,
        malvin_args=malvin_args,
        extra_args=tuple(ctx.args),
        use_local_docker=use_local_docker,
    )


@cli.command("hello")
@click.argument("task_name", required=False)
@_hello_options
@click.pass_context
def hello(
    ctx: click.Context,
    task_name: str | None,
    run_on_host: bool,
    use_local_docker: bool,
    docker_image: str | None,
    dry_run: bool,
    malvin_args: tuple[str, ...],
) -> None:
    """Run malvin do Hello (Cursor connectivity probe). Default: full Modal sandbox smoke test (auth + CIDR allowlist, no Harbor grade). Use ``--host`` for a local probe."""
    if run_on_host:
        if use_local_docker:
            raise click.ClickException("Use either --host or --local, not both")
        run_hello_probe_on_host(malvin_args + tuple(ctx.args), dry_run=dry_run)
        return
    if not task_name:
        raise click.ClickException("TASK_NAME is required unless --host is set")
    run_task(
        local_task_name=task_name,
        task_dir=None,
        workspace=None,
        results_dir=None,
        malvin_command="hello",
        checks_override=None,
        runtime="host",
        skip_materialize=False,
        grade_only=False,
        skip_grade=True,
        apply_solution=False,
        reset_workspace_flag=True,
        docker_image=docker_image,
        dry_run=dry_run,
        malvin_args=malvin_args,
        extra_args=tuple(ctx.args),
        use_local_docker=use_local_docker,
    )


# Backward-compatible alias for tests and callers that import ``main``.
main = cli


def _test_malvin_repo_root() -> None:
    root = malvin_repo_root()
    assert (root / "Cargo.toml").is_file(), root
    assert (root / "ops" / "deepswe_run.py").is_file(), root


def _test_default_deepswe_tasks_root() -> None:
    root = default_deepswe_tasks_root()
    assert root.name == "tasks", root


def _test_resolve_local_task_dir() -> None:
    tasks_root = default_deepswe_tasks_root()
    if not tasks_root.is_dir():
        return
    sample = tasks_root / "bandit-interprocedural-taint-checks"
    if not sample.is_dir():
        return
    resolved = resolve_local_task_dir("bandit-interprocedural-taint-checks")
    assert resolved == sample.resolve(), (resolved, sample)


def _test_local_agent_image_tag() -> None:
    assert local_agent_image_tag("foo") == "deepswe-foo:agent"


def _test_docker_local_eval_cmd() -> None:
    tasks_root = default_deepswe_tasks_root()
    task = tasks_root / "bandit-interprocedural-taint-checks"
    if not task.is_dir():
        return
    spec = parse_task_dir(task)
    deepswe_run_py = Path(__file__).resolve()
    agent_cmd = docker_local_agent_cmd(
        image="deepswe-test:agent",
        spec=spec,
        task_dir=task,
        workspace=Path("/tmp/ws"),
        run_root=Path("/tmp/run"),
        deepswe_run_py=deepswe_run_py,
        malvin_command="code",
        malvin_args=(),
        reset_workspace_flag=False,
        checks_override=None,
    )
    agent_joined = " ".join(agent_cmd)
    assert "--runtime in-sandbox" in agent_joined
    assert "--skip-grade" in agent_joined
    assert DEEPSWE_RUN_REMOTE in agent_joined
    assert "--command code" in agent_joined
    assert "/tests:ro" not in agent_joined
    assert "task.toml" in agent_joined
    assert "instruction.md" in agent_joined

    grade_cmd = docker_local_grade_cmd(
        image="deepswe-test:base",
        spec=spec,
        task_dir=task,
        workspace=Path("/tmp/ws"),
        run_root=Path("/tmp/run"),
        deepswe_run_py=deepswe_run_py,
        apply_solution=False,
        reset_workspace_flag=False,
    )
    grade_joined = " ".join(grade_cmd)
    assert "--grade-only" in grade_joined
    assert "/tests:ro" in grade_joined
    assert "/task:ro" in grade_joined


def _test_solve_dry_run() -> None:
    from click.testing import CliRunner

    tasks_root = default_deepswe_tasks_root()
    if not (tasks_root / "bandit-interprocedural-taint-checks").is_dir():
        return
    runner = CliRunner()
    result = runner.invoke(
        cli,
        [
            "solve",
            "--local",
            "bandit-interprocedural-taint-checks",
            "--dry-run",
        ],
    )
    assert result.exit_code == 0, result.output
    assert "docker run" in result.output
    assert "Runtime: local-docker" in result.output
    assert "--runtime in-sandbox" in result.output


def _patch_modal_cursor_credentials() -> Any:
    """Self-test helper: agent Modal paths require host Cursor credentials."""
    return patch("deepswe_modal.cursor_credentials_available", return_value=True)


def _test_solve_modal_missing_credentials() -> None:
    """Agent Modal solve fails fast when host lacks Cursor credentials."""
    from click.testing import CliRunner

    tasks_root = default_deepswe_tasks_root()
    if not (tasks_root / "bandit-interprocedural-taint-checks").is_dir():
        return
    keys = ["CURSOR_AGENT_API_KEY", "CURSOR_API_KEY", "AGENT_API_KEY", "MODAL_CURSOR_SECRET_NAME"]
    saved = {key: os.environ.pop(key, None) for key in keys}
    try:
        runner = CliRunner()
        result = runner.invoke(
            cli,
            ["solve", "bandit-interprocedural-taint-checks", "--dry-run"],
        )
        assert result.exit_code != 0, result.output
        assert "Cursor API key required" in result.output
        assert "Dry run: would materialize workspace" not in result.output
    finally:
        for key, value in saved.items():
            if value is None:
                os.environ.pop(key, None)
            else:
                os.environ[key] = value


def _test_solve_modal_dry_run() -> None:
    from click.testing import CliRunner

    tasks_root = default_deepswe_tasks_root()
    if not (tasks_root / "bandit-interprocedural-taint-checks").is_dir():
        return
    runner = CliRunner()
    with _patch_modal_cursor_credentials():
        result = runner.invoke(
            cli,
            [
                "solve",
                "bandit-interprocedural-taint-checks",
                "--skip-grade",
                "--dry-run",
            ],
        )
    assert result.exit_code == 0, result.output
    assert "Runtime: modal" in result.output
    assert "docker run" not in result.output
    assert "Dry run: would materialize workspace" in result.output
    assert "Dry run: malvin agent in Modal sandbox" in result.output


def _test_solve_modal_full_dry_run() -> None:
    """Default solve runs malvin and Harbor grade in one Modal sandbox."""
    from click.testing import CliRunner

    tasks_root = default_deepswe_tasks_root()
    if not (tasks_root / "bandit-interprocedural-taint-checks").is_dir():
        return
    runner = CliRunner()
    with _patch_modal_cursor_credentials():
        result = runner.invoke(
            cli,
            ["solve", "bandit-interprocedural-taint-checks", "--dry-run"],
        )
    assert result.exit_code == 0, result.output
    assert "Runtime: modal" in result.output
    assert "Dry run: malvin agent in Modal sandbox (Cursor API allowlist)" in result.output
    assert "Dry run: Harbor grade in same Modal sandbox (in-sandbox runtime)" in result.output
    assert "Running agent on host" not in result.output


def _test_solve_resets_workspace_for_agent_runs() -> None:
    """Agent solves always reset workspace."""
    from click.testing import CliRunner

    tasks_root = default_deepswe_tasks_root()
    if not (tasks_root / "bandit-interprocedural-taint-checks").is_dir():
        return
    runner = CliRunner()
    captured: dict[str, bool] = {}

    def fake_modal_eval(**kwargs: Any) -> None:
        captured["reset_flag"] = kwargs.get("reset_flag", False)
        captured["grade_only"] = kwargs.get("grade_only", False)

    with _patch_modal_cursor_credentials(), patch("deepswe_modal.run_modal_eval", fake_modal_eval):
        result = runner.invoke(
            cli,
            ["solve", "bandit-interprocedural-taint-checks", "--dry-run"],
        )
    assert result.exit_code == 0, result.output
    assert captured.get("reset_flag") is True, captured
    assert captured.get("grade_only") is False, captured


def _test_solve_local_dry_run_passes_reset() -> None:
    from click.testing import CliRunner

    tasks_root = default_deepswe_tasks_root()
    if not (tasks_root / "bandit-interprocedural-taint-checks").is_dir():
        return
    runner = CliRunner()
    result = runner.invoke(
        cli,
        ["solve", "--local", "bandit-interprocedural-taint-checks", "--dry-run"],
    )
    assert result.exit_code == 0, result.output
    assert "--reset" in result.output


def _test_solve_command_in_help() -> None:
    from click.testing import CliRunner

    runner = CliRunner()
    result = runner.invoke(cli, ["--help"])
    assert result.exit_code == 0, result.output
    for name in ("solve", "hello", "tasks", "run", "self-test"):
        assert name in result.output, name
    assert "--task" not in result.output.split("Commands:")[0]


def _test_task_name_shorthand_routes_to_solve() -> None:
    """``deepswe_run.py TASK_NAME`` is equivalent to ``solve TASK_NAME``."""
    from click.testing import CliRunner

    tasks_root = default_deepswe_tasks_root()
    if not (tasks_root / "bandit-interprocedural-taint-checks").is_dir():
        return
    runner = CliRunner()
    with _patch_modal_cursor_credentials():
        result = runner.invoke(
            cli,
            ["bandit-interprocedural-taint-checks", "--skip-grade", "--dry-run"],
        )
    assert result.exit_code == 0, result.output
    assert "Runtime: modal" in result.output
    assert "bandit-interprocedural-taint-checks" in result.output


def _test_bare_invocation_shows_usage() -> None:
    from click.testing import CliRunner

    runner = CliRunner()
    result = runner.invoke(cli, [])
    assert result.exit_code != 0, result.output
    assert "Missing command" in result.output or "Usage:" in result.output


def _test_list_deepswe_tasks() -> None:
    tasks_root = default_deepswe_tasks_root()
    if not tasks_root.is_dir():
        return
    task_ids = list_deepswe_tasks()
    assert task_ids, tasks_root
    assert task_ids == sorted(task_ids)
    sample = tasks_root / "bandit-interprocedural-taint-checks"
    if sample.is_dir():
        assert "bandit-interprocedural-taint-checks" in task_ids


def _test_read_task_language() -> None:
    tasks_root = default_deepswe_tasks_root()
    task = tasks_root / "bandit-interprocedural-taint-checks"
    if not task.is_dir():
        return
    assert read_task_language(task) == "python", task


def _test_list_deepswe_tasks_with_language() -> None:
    tasks_root = default_deepswe_tasks_root()
    if not tasks_root.is_dir():
        return
    entries = list_deepswe_tasks_with_language()
    assert entries, tasks_root
    assert entries == sorted(entries, key=lambda pair: pair[0])
    task_ids = [task_id for task_id, _language in entries]
    assert task_ids == list_deepswe_tasks()
    sample = tasks_root / "bandit-interprocedural-taint-checks"
    if sample.is_dir():
        by_id = dict(entries)
        assert by_id["bandit-interprocedural-taint-checks"] == "python"


def _test_discover_deepswe_checks_minimal() -> None:
    with tempfile.TemporaryDirectory() as tmp:
        root = Path(tmp)
        lines = discover_deepswe_check_lines(root)
        assert lines == [], lines


def _test_discover_deepswe_checks_python_repo() -> None:
    with tempfile.TemporaryDirectory() as tmp:
        root = Path(tmp)
        (root / "pkg").mkdir()
        (root / "pkg" / "mod.py").write_text("x = 1\n", encoding="utf-8")
        (root / "tests").mkdir()
        (root / "tests" / "test_mod.py").write_text(
            "def test_x():\n    assert True\n", encoding="utf-8"
        )
        text = discover_deepswe_checks(root)
        assert text == "\n"
        assert "pytest" not in text


def _test_discover_deepswe_checks_stestr_repo() -> None:
    with tempfile.TemporaryDirectory() as tmp:
        root = Path(tmp)
        (root / "tests").mkdir()
        (root / "tests" / "test_mod.py").write_text(
            "def test_x():\n    assert True\n", encoding="utf-8"
        )
        (root / "test-requirements.txt").write_text("stestr>=2.5.0\n", encoding="utf-8")
        text = discover_deepswe_checks(root)
        assert text == "\n"


def _test_discover_deepswe_checks_stestr_drops_stale_pytest() -> None:
    with tempfile.TemporaryDirectory() as tmp:
        root = Path(tmp)
        malvin_dir = root / ".malvin"
        malvin_dir.mkdir()
        (malvin_dir / "checks").write_text("pytest -sv tests\n", encoding="utf-8")
        (root / "test-requirements.txt").write_text("stestr>=2.5.0\n", encoding="utf-8")
        text = discover_deepswe_checks(root)
        assert text == "pytest -sv tests\n"


def _test_discover_deepswe_checks_precommit() -> None:
    with tempfile.TemporaryDirectory() as tmp:
        root = Path(tmp)
        (root / ".pre-commit-config.yaml").write_text(
            "repos:\n  - repo: local\n    hooks:\n      - id: ruff\n"
            "        entry: ruff check .\n",
            encoding="utf-8",
        )
        lines = discover_deepswe_check_lines(root)
        assert lines == ["ruff check ."]


def _test_discover_deepswe_checks_existing_malvin_checks() -> None:
    with tempfile.TemporaryDirectory() as tmp:
        root = Path(tmp)
        malvin_dir = root / ".malvin"
        malvin_dir.mkdir()
        (malvin_dir / "checks").write_text(
            "mypy .\nruff check .\n", encoding="utf-8"
        )
        lines = discover_deepswe_check_lines(root)
        assert lines == ["mypy .", "ruff check ."]


def _test_write_plan_and_checks_discovers() -> None:
    with tempfile.TemporaryDirectory() as tmp:
        workspace = Path(tmp)
        instruction = workspace / "instruction.md"
        instruction.write_text("fix it\n", encoding="utf-8")
        (workspace / "mod.py").write_text("pass\n", encoding="utf-8")
        (workspace / "tests").mkdir()
        (workspace / "tests" / "test_mod.py").write_text(
            "def test_mod():\n    assert True\n", encoding="utf-8"
        )
        spec = TaskSpec(
            task_dir=workspace,
            task_id="fake",
            base_commit="HEAD",
            docker_image="fake:local",
            dockerfile=workspace / "Dockerfile",
            instruction=instruction,
            tests_dir=workspace / "tests",
            test_sh=workspace / "tests" / "test.sh",
            solution_patch=None,
            repository_url=None,
            agent_timeout_sec=3600.0,
            verifier_timeout_sec=1800.0,
            environment_memory_mb=4096,
        )
        write_plan_and_checks(
            spec,
            workspace,
            command="code",
            checks_override=None,
            dry_run=False,
        )
        checks = (workspace / ".malvin" / "checks").read_text(encoding="utf-8")
        assert checks == "\n"
        plan_text = (workspace / "plan.md").read_text(encoding="utf-8")
        assert plan_text == "fix it\n"


def _test_parse_task_dir_does_not_require_test_sh() -> None:
    with tempfile.TemporaryDirectory() as tmp:
        task_dir = Path(tmp)
        toml_text = (
            '[metadata]\ntask_id = "no-tests"\nbase_commit_hash = "abc123"\n'
            '[environment]\ndocker_image = "ubuntu:22.04"\n'
        )
        (task_dir / "task.toml").write_text(toml_text, encoding="utf-8")
        (task_dir / "instruction.md").write_text("do something\n", encoding="utf-8")
        spec = parse_task_dir(task_dir)
        assert spec.task_id == "no-tests"
        assert spec.test_sh == task_dir / "tests" / "test.sh"
        assert not spec.test_sh.is_file()


def _test_validate_verifier_paths_fails_without_test_sh() -> None:
    with tempfile.TemporaryDirectory() as tmp:
        task_dir = Path(tmp)
        toml_text = (
            '[metadata]\ntask_id = "no-tests"\nbase_commit_hash = "abc123"\n'
            '[environment]\ndocker_image = "ubuntu:22.04"\n'
        )
        (task_dir / "task.toml").write_text(toml_text, encoding="utf-8")
        (task_dir / "instruction.md").write_text("do something\n", encoding="utf-8")
        spec = parse_task_dir(task_dir)
        try:
            validate_verifier_paths(spec)
            assert False, "Expected ClickException"
        except click.ClickException:
            pass


def _test_scan_pytest_monkeypatch_hooks() -> None:
    with tempfile.TemporaryDirectory() as tmp:
        root = Path(tmp)
        tests = root / "tests"
        tests.mkdir()
        (tests / "test_api.py").write_text(
            "def test_x(monkeypatch):\n"
            '    monkeypatch.setattr(Foo, "bar", 1)\n'
            '    monkeypatch.setattr(mod.Baz, "qux", 2)\n',
            encoding="utf-8",
        )
        hooks = scan_pytest_monkeypatch_hooks(root)
        assert hooks == [("Foo", "bar"), ("mod.Baz", "qux")]


def _test_scan_class_level_attributes() -> None:
    with tempfile.TemporaryDirectory() as tmp:
        root = Path(tmp)
        pkg = root / "pkg"
        pkg.mkdir()
        (pkg / "mod.py").write_text(
            "class Foo:\n    bar = 1\n    _hidden = 2\n\nclass Baz:\n    qux: int = 3\n",
            encoding="utf-8",
        )
        attrs = scan_class_level_attributes(root)
        assert ("pkg.mod.Foo", "bar") in attrs
        assert ("pkg.mod.Foo", "_hidden") in attrs
        assert ("pkg.mod.Baz", "qux") in attrs


def _test_patch_surface_targets_prefers_config_style_classes() -> None:
    with tempfile.TemporaryDirectory() as tmp:
        root = Path(tmp)
        pkg = root / "igel"
        pkg.mkdir()
        (pkg / "igel.py").write_text(
            "class Igel:\n"
            "    results_path = 'x'\n"
            "    default_model_path = 'y'\n"
            "    description_file = 'z'\n",
            encoding="utf-8",
        )
        targets = patch_surface_targets(root)
        assert ("igel.igel.Igel", "results_path") in targets


def _test_render_patch_surface_probe_roundtrip() -> None:
    with tempfile.TemporaryDirectory() as tmp:
        root = Path(tmp)
        pkg = root / "pkg"
        pkg.mkdir()
        (pkg / "__init__.py").write_text("", encoding="utf-8")
        (pkg / "mod.py").write_text(
            "class Foo:\n    bar = 1\n",
            encoding="utf-8",
        )
        probe = root / "probe.py"
        probe.write_text(
            render_patch_surface_probe([("pkg.mod.Foo", "bar")]),
            encoding="utf-8",
        )
        proc = subprocess.run(
            [sys.executable, str(probe)],
            cwd=root,
            env={**os.environ, "PYTHONPATH": str(root)},
            capture_output=True,
            text=True,
            check=False,
        )
        assert proc.returncode == 0, proc.stderr
        assert "patch surface ok" in proc.stdout


def _test_write_plan_and_checks_includes_patch_surface_probe() -> None:
    with tempfile.TemporaryDirectory() as tmp:
        workspace = Path(tmp)
        instruction = workspace / "instruction.md"
        instruction.write_text("fix it\n", encoding="utf-8")
        pkg = workspace / "pkg"
        pkg.mkdir()
        (pkg / "mod.py").write_text("class Foo:\n    a = 1\n    b = 2\n", encoding="utf-8")
        spec = TaskSpec(
            task_dir=workspace,
            task_id="fake",
            base_commit="HEAD",
            docker_image="fake:local",
            dockerfile=workspace / "Dockerfile",
            instruction=instruction,
            tests_dir=workspace / "tests",
            test_sh=workspace / "tests" / "test.sh",
            solution_patch=None,
            repository_url=None,
            agent_timeout_sec=3600.0,
            verifier_timeout_sec=1800.0,
            environment_memory_mb=4096,
        )
        write_plan_and_checks(
            spec,
            workspace,
            command="code",
            checks_override=None,
            dry_run=False,
        )
        checks = (workspace / ".malvin" / "checks").read_text(encoding="utf-8")
        assert _PATCH_SURFACE_PROBE_COMMAND in checks
        probe = workspace / ".malvin" / "patch_surface_probe.py"
        assert probe.is_file(), probe
        proc = subprocess.run(
            [sys.executable, str(probe)],
            cwd=workspace,
            env={**os.environ, "PYTHONPATH": str(workspace)},
            capture_output=True,
            text=True,
            check=False,
        )
        assert proc.returncode == 0, proc.stderr


def _test_is_modal_spend_limit_error() -> None:
    assert _is_modal_spend_limit_error(RuntimeError("Workspace billing cycle spend limit reached"))
    try:
        import modal.exception

        assert _is_modal_spend_limit_error(
            modal.exception.ResourceExhaustedError("Workspace has exceeded its spend limit")
        )
        assert _is_modal_spend_limit_error(
            modal.exception.RemoteError("billing cycle spend limit reached, cancelling task")
        )
    except ModuleNotFoundError:
        pass
    assert not _is_modal_spend_limit_error(RuntimeError("connection reset"))


def _test_solve_modal_spend_limit_falls_back_to_local_dry_run() -> None:
    from click.testing import CliRunner

    tasks_root = default_deepswe_tasks_root()
    task = tasks_root / "bandit-interprocedural-taint-checks"
    if not task.is_dir():
        return
    try:
        import modal.exception
    except ModuleNotFoundError:
        return
    runner = CliRunner()
    with (
        _patch_modal_cursor_credentials(),
        patch(
            "deepswe_modal.run_modal_eval",
            side_effect=modal.exception.RemoteError("billing cycle spend limit reached"),
        ),
    ):
        result = runner.invoke(
            cli,
            ["solve", "bandit-interprocedural-taint-checks", "--dry-run"],
        )
    assert result.exit_code == 0, result.output
    assert "Modal workspace spend limit reached" in result.output
    assert "local-docker (Modal spend-limit fallback)" in result.output


def _test_tasks_command() -> None:
    from click.testing import CliRunner

    tasks_root = default_deepswe_tasks_root()
    if not tasks_root.is_dir():
        return
    runner = CliRunner()
    result = runner.invoke(cli, ["tasks"])
    assert result.exit_code == 0, result.output
    lines = [line for line in result.output.splitlines() if line.strip()]
    task_ids = [line.split("\t", 1)[0] for line in lines]
    assert task_ids == sorted(task_ids)
    assert "bandit-interprocedural-taint-checks" in task_ids
    bandit_line = next(
        line for line in lines if line.startswith("bandit-interprocedural-taint-checks\t")
    )
    assert bandit_line.endswith("\tpython"), bandit_line


def _test_ephemeral_cache_find_expr() -> None:
    expr = ephemeral_cache_find_expr()
    assert "__pycache__" in expr
    assert ".pytest_cache" in expr


_GIT_TEST_IDENTITY = {
    "GIT_AUTHOR_NAME": "malvin-test",
    "GIT_AUTHOR_EMAIL": "malvin-test@example.com",
    "GIT_COMMITTER_NAME": "malvin-test",
    "GIT_COMMITTER_EMAIL": "malvin-test@example.com",
}


def _test_reset_workspace_removes_user_pycache() -> None:
    with tempfile.TemporaryDirectory() as tmp:
        workspace = Path(tmp)
        run_cmd(["git", "init", "-q"], cwd=workspace, env=_GIT_TEST_IDENTITY)
        run_cmd(
            ["git", "commit", "--allow-empty", "-m", "init", "-q"],
            cwd=workspace,
            env=_GIT_TEST_IDENTITY,
        )
        cache = workspace / "pkg" / "__pycache__"
        cache.mkdir(parents=True)
        (cache / "mod.cpython-312.pyc").write_bytes(b"\x00")
        spec = TaskSpec(
            task_dir=workspace,
            task_id="fake",
            base_commit="HEAD",
            docker_image="fake:local",
            dockerfile=workspace / "Dockerfile",
            instruction=workspace / "instruction.md",
            tests_dir=workspace / "tests",
            test_sh=workspace / "tests" / "test.sh",
            solution_patch=None,
            repository_url=None,
            agent_timeout_sec=3600.0,
            verifier_timeout_sec=1800.0,
            environment_memory_mb=4096,
        )
        # Host-owned caches are removed by git clean; Docker purge is out of scope here.
        with patch("deepswe_run.purge_root_owned_ephemeral_caches", return_value=False):
            reset_workspace(spec, workspace, dry_run=False)
        assert not cache.exists(), cache


def _test_purge_root_owned_ephemeral_caches_docker_cmd() -> None:
    with tempfile.TemporaryDirectory() as tmp:
        workspace = Path(tmp)
        captured: dict[str, list[str]] = {}

        def fake_run(cmd: list[str], **kwargs: Any) -> subprocess.CompletedProcess[str]:
            captured["cmd"] = cmd
            return subprocess.CompletedProcess(cmd, 0)

        with patch("deepswe_run.docker_daemon_available", return_value=True):
            with patch("deepswe_run.subprocess.run", side_effect=fake_run):
                assert purge_root_owned_ephemeral_caches(workspace)
        cmd = captured["cmd"]
        assert cmd[0:2] == ["docker", "run"]
        assert list(cmd[2 : 2 + len(DOCKER_RUN_FAST_ARGS)]) == list(DOCKER_RUN_FAST_ARGS)
        vol = 2 + len(DOCKER_RUN_FAST_ARGS)
        assert cmd[vol : vol + 2] == ["-v", f"{workspace.resolve()}:/app"]
        assert cmd[vol + 2] == DOCKER_EPHEMERAL_PURGE_IMAGE
        shell = cmd[vol + 5]
        find_expr = ephemeral_cache_find_expr()
        assert find_expr in shell
        assert "__pycache__" in shell
        assert ".stestr" in shell
        assert "acp_spawn" in shell


def _test_purge_root_owned_sandbox_artifacts_docker() -> None:
    if skip_docker_selftests() or not docker_daemon_available():
        return
    scratch_parent = Path.home() / ".malvin_home" / "deepswe-self-test"
    scratch_parent.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(dir=scratch_parent) as tmp:
        workspace = Path(tmp)
        image = DOCKER_EPHEMERAL_PURGE_IMAGE
        proc = subprocess.run(
            [
                "docker",
                "run",
                *DOCKER_RUN_FAST_ARGS,
                "-v",
                f"{workspace.resolve()}:/app",
                image,
                "sh",
                "-c",
                "mkdir -p /app/.stestr /app/.malvin/acp_spawn /app/.kiss "
                "&& touch /app/.stestr/x /app/.malvin/acp_spawn/pid1.lock "
                "/app/.kiss/rslip.json",
            ],
            capture_output=True,
        )
        if proc.returncode != 0:
            return
        stestr = workspace / ".stestr"
        assert stestr.is_dir()
        try:
            next(stestr.iterdir()).unlink()
            host_blocked = False
        except OSError:
            host_blocked = True
        assert host_blocked or os.geteuid() == 0
        assert purge_root_owned_ephemeral_caches(workspace)
        assert not stestr.exists(), stestr
        assert not (workspace / ".malvin" / "acp_spawn").exists()
        assert not (workspace / ".kiss").exists()


def _test_scan_class_level_attributes_skips_non_utf8() -> None:
    with tempfile.TemporaryDirectory() as tmp:
        workspace = Path(tmp)
        latin1 = workspace / "examples" / "trojansource_latin1.py"
        latin1.parent.mkdir(parents=True)
        latin1.write_bytes(
            b"#!/usr/bin/env python3\n# -*- coding: latin-1 -*-\n"
            b"access_level = \"user\"\n" + bytes([0xE0, 0x00, 0x00])
        )
        assert scan_class_level_attributes(workspace) == []


def _test_purge_root_owned_ephemeral_caches_docker() -> None:
    """Docker-marked pytest entry: validate purge docker argv (fast); real containers in ops selftest."""
    if skip_docker_selftests() or not docker_daemon_available():
        return
    _test_purge_root_owned_ephemeral_caches_docker_cmd()


def _test_run_malvin_uses_plan_name_not_at_notation() -> None:
    with tempfile.TemporaryDirectory() as tmp:
        workspace = Path(tmp)
        (workspace / "plan.md").write_text("task\n", encoding="utf-8")
        captured: dict[str, list[str]] = {}

        def fake_run(cmd: list[str], **kwargs: Any) -> subprocess.CompletedProcess[str]:
            captured["cmd"] = cmd
            return subprocess.CompletedProcess(cmd, 0)

        with patch("subprocess.run", fake_run):
            run_malvin(workspace, command="code", malvin_args=(), dry_run=False)
        assert captured["cmd"][2] == "plan.md"
        assert "@" not in captured["cmd"][2]


def _test_run_malvin_do_uses_prompt_not_plan() -> None:
    with tempfile.TemporaryDirectory() as tmp:
        workspace = Path(tmp)
        captured: dict[str, list[str]] = {}

        def fake_run(cmd: list[str], **kwargs: Any) -> subprocess.CompletedProcess[str]:
            captured["cmd"] = cmd
            return subprocess.CompletedProcess(cmd, 0)

        with patch("subprocess.run", fake_run):
            run_malvin(workspace, command="do", malvin_args=("Hello",), dry_run=False)
        assert captured["cmd"] == [MALVIN_CMD, "do", "Hello"]


def _test_resolve_malvin_cmd_prefers_repo_target() -> None:
    root = malvin_repo_root()
    debug = root / "target" / "debug" / "malvin"
    if not debug.is_file():
        return
    with patch.dict(os.environ, {}, clear=False):
        os.environ.pop("MALVIN", None)
        assert resolve_malvin_cmd() == str(debug)


def _test_relay_subprocess_stdout_sets_force_tee_env() -> None:
    with tempfile.TemporaryDirectory() as tmp:
        captured: dict[str, Any] = {}

        def fake_popen(*args: Any, **kwargs: Any) -> Any:
            captured["env"] = kwargs.get("env")
            proc = MagicMock()
            proc.stdout = iter([])
            proc.wait.return_value = 0
            proc.returncode = 0
            return proc

        with patch(f"{__name__}.subprocess.Popen", fake_popen):
            _relay_subprocess_stdout(["echo", "hi"], cwd=Path(tmp))
        assert captured["env"] is not None
        assert captured["env"].get("MALVIN_FORCE_STDOUT_TEE") == "1"


def _test_run_command_accepts_hello() -> None:
    with tempfile.TemporaryDirectory() as tmp:
        run_cmd = cli.get_command(click.Context(cli), "run")
        ctx = run_cmd.make_context(  # type: ignore[union-attr]
            "run",
            ["--task", tmp, "--command", "hello", "--dry-run"],
        )
        assert ctx.params["malvin_command"] == "hello"


def _test_hello_host_relays_stdout() -> None:
    from click.testing import CliRunner

    runner = CliRunner()

    def fake_relay(cmd: list[str], *, cwd: Path) -> tuple[int, str]:
        assert "do" in cmd and "Hello" in cmd
        return 0, "agent said hi\n"

    with patch(f"{__name__}._relay_subprocess_stdout", fake_relay):
        result = runner.invoke(cli, ["hello", "--host"])
    assert result.exit_code == 0, result.output
    assert "agent said hi" in result.output
    assert "--- agent stdout ---" in result.output


def _test_hello_host_rejects_local() -> None:
    from click.testing import CliRunner

    runner = CliRunner()
    result = runner.invoke(cli, ["hello", "--host", "--local"])
    assert result.exit_code != 0
    assert "not both" in result.output.lower()


def _test_hello_modal_dry_run() -> None:
    """``hello TASK`` runs malvin do Hello on Modal without Harbor grading."""
    from click.testing import CliRunner

    tasks_root = default_deepswe_tasks_root()
    if not (tasks_root / "bandit-interprocedural-taint-checks").is_dir():
        return
    runner = CliRunner()
    captured: dict[str, Any] = {}

    def fake_modal_eval(**kwargs: Any) -> None:
        captured.update(kwargs)

    with _patch_modal_cursor_credentials(), patch("deepswe_modal.run_modal_eval", fake_modal_eval):
        result = runner.invoke(
            cli,
            ["hello", "bandit-interprocedural-taint-checks", "--dry-run"],
        )
    assert result.exit_code == 0, result.output
    assert captured.get("malvin_command") == "hello", captured
    assert captured.get("malvin_args") == (), captured
    assert captured.get("reset_flag") is True, captured
    assert captured.get("skip_grade") is True, captured
    assert captured.get("grade_only") is False, captured
    assert "Harbor grade" not in result.output, result.output


def _test_hello_command_in_help() -> None:
    from click.testing import CliRunner

    runner = CliRunner()
    result = runner.invoke(cli, ["--help"])
    assert result.exit_code == 0, result.output
    assert "hello" in result.output


def _test_local_grade_only_apply_solution() -> None:
    """Integration: grade-only apply-solution path (Harbor stubbed when fast-grade env set)."""
    if skip_docker_selftests():
        return
    tasks_root = default_deepswe_tasks_root()
    task = tasks_root / "bandit-interprocedural-taint-checks"
    if not task.is_dir():
        return
    from click.testing import CliRunner
    from unittest.mock import patch

    prev = os.environ.get("DEEPSWE_TEST_FAST_GRADE")
    os.environ["DEEPSWE_TEST_FAST_GRADE"] = "1"
    try:
        runner = CliRunner()
        with patch("deepswe_run.reset_workspace"), patch("deepswe_run.apply_patch"):
            result = runner.invoke(
                cli,
                [
                    "run",
                    "--task",
                    str(task),
                    "--grade-only",
                    "--apply-solution",
                ],
            )
        assert result.exit_code == 0, result.output
        assert "reward: 1" in result.output
        assert "pass: True" in result.output
    finally:
        if prev is None:
            os.environ.pop("DEEPSWE_TEST_FAST_GRADE", None)
        else:
            os.environ["DEEPSWE_TEST_FAST_GRADE"] = prev


def _test_malvin_mem_limit_gb_from_task_memory() -> None:
    assert malvin_mem_limit_gb(4096) == 4
    assert malvin_mem_limit_gb(8192) == 8
    assert malvin_mem_limit_gb(5000) == 5


def _test_ensure_deepswe_malvin_config_seeds_home_config() -> None:
    import tempfile
    from unittest.mock import patch

    with tempfile.TemporaryDirectory() as tmp:
        home = Path(tmp) / "home"
        home.mkdir()
        spec = TaskSpec(
            task_dir=Path("/task"),
            task_id="t",
            base_commit="abc",
            docker_image="img",
            dockerfile=Path("/task/environment/Dockerfile"),
            instruction=Path("/task/instruction.md"),
            tests_dir=Path("/task/tests"),
            test_sh=Path("/task/tests/test.sh"),
            solution_patch=None,
            repository_url=None,
            agent_timeout_sec=3600.0,
            verifier_timeout_sec=1800.0,
            environment_memory_mb=8192,
        )
        with patch.object(Path, "home", return_value=home):
            ensure_deepswe_malvin_config(spec, dry_run=False)
        cfg = home / ".malvin_home" / "config.toml"
        assert cfg.is_file(), cfg
        assert "mem_limit_gb = 8" in cfg.read_text(encoding="utf-8")


def _test_ensure_deepswe_malvin_config_skips_default_memory() -> None:
    import tempfile
    from unittest.mock import patch

    with tempfile.TemporaryDirectory() as tmp:
        home = Path(tmp) / "home"
        home.mkdir()
        spec = TaskSpec(
            task_dir=Path("/task"),
            task_id="t",
            base_commit="abc",
            docker_image="img",
            dockerfile=Path("/task/environment/Dockerfile"),
            instruction=Path("/task/instruction.md"),
            tests_dir=Path("/task/tests"),
            test_sh=Path("/task/tests/test.sh"),
            solution_patch=None,
            repository_url=None,
            agent_timeout_sec=3600.0,
            verifier_timeout_sec=1800.0,
            environment_memory_mb=4096,
        )
        with patch.object(Path, "home", return_value=home):
            ensure_deepswe_malvin_config(spec, dry_run=False)
        assert not (home / ".malvin_home" / "config.toml").exists()


def _test_prepare_task_sandbox_dry_run() -> None:
    with tempfile.TemporaryDirectory() as tmp:
        workspace = Path(tmp)
        dockerfile = workspace / "Dockerfile"
        dockerfile.write_text(
            "RUN git clone https://example.com/repo .\n"
            "RUN pip install -e .\n",
            encoding="utf-8",
        )
        spec = TaskSpec(
            task_dir=workspace,
            task_id="fake",
            base_commit="HEAD",
            docker_image="fake:local",
            dockerfile=dockerfile,
            instruction=workspace / "instruction.md",
            tests_dir=workspace / "tests",
            test_sh=workspace / "tests" / "test.sh",
            solution_patch=None,
            repository_url=None,
            agent_timeout_sec=3600.0,
            verifier_timeout_sec=1800.0,
            environment_memory_mb=4096,
        )
        result = prepare_task_sandbox(
            spec,
            workspace,
            checks="true\n",
            dry_run=True,
        )
        assert result.sync_commands == (), result
        assert result.ok is True


def _minimal_timeout_task_tree(
    tmp: Path,
    *,
    agent_timeout_sec: float = 2.5,
    verifier_timeout_sec: float = 1.0,
) -> tuple[Path, Path]:
    task_dir = tmp / "task"
    task_dir.mkdir()
    workspace = tmp / "workspace"
    workspace.mkdir()
    (task_dir / "task.toml").write_text(
        """
[metadata]
task_id = "timeout-test"
base_commit_hash = "abc"

[environment]
docker_image = "fake:local"

[agent]
timeout_sec = {agent}

[verifier]
timeout_sec = {verifier}
""".format(agent=agent_timeout_sec, verifier=verifier_timeout_sec),
        encoding="utf-8",
    )
    (task_dir / "instruction.md").write_text("instruction\n", encoding="utf-8")
    tests = task_dir / "tests"
    tests.mkdir()
    (tests / "test.sh").write_text("#!/bin/bash\ntrue\n", encoding="utf-8")
    (workspace / "plan.md").write_text("# plan\n", encoding="utf-8")
    return task_dir, workspace


def _test_remaining_sec_floors_at_zero() -> None:
    assert _remaining_sec(time.monotonic() - 1.0) == 0.0
    assert _remaining_sec(time.monotonic() + 5.0) >= 4.9


def _test_run_with_timeout_kills_slow_command() -> None:
    t0 = time.monotonic()
    result = _run_with_timeout(["sleep", "999"], timeout_sec=1.0, stream=False)
    elapsed = time.monotonic() - t0
    assert result.timed_out is True
    assert result.exit_code == TIMEOUT_EXIT_CODE
    assert elapsed < 2.5


def _test_exit_after_agent_timeout_grade_pass() -> None:
    _exit_from_evaluation(
        {"pass": True},
        {"timed_out": True, "exit_code": TIMEOUT_EXIT_CODE},
    )


def _test_exit_after_agent_timeout_grade_fail() -> None:
    try:
        _exit_from_evaluation(
            {"pass": False},
            {"timed_out": True, "exit_code": TIMEOUT_EXIT_CODE},
        )
    except SystemExit as exc:
        assert exc.code == 1
    else:
        raise AssertionError("expected SystemExit(1)")


def _test_agent_timeout_skip_grade_exits_zero() -> None:
    _exit_from_evaluation(
        {"skipped": True, "pass": None},
        {"timed_out": True, "exit_code": TIMEOUT_EXIT_CODE},
    )


def _test_verifier_timeout_forces_fail() -> None:
    with tempfile.TemporaryDirectory() as tmp:
        workspace = Path(tmp)
        logs_dir = workspace / "logs"
        test_sh = workspace / "test.sh"
        test_sh.write_text("#!/bin/bash\nsleep 999\n", encoding="utf-8")
        result = grade_workspace_native(
            workspace,
            test_sh,
            logs_dir,
            dry_run=False,
            timeout_sec=0.0,
            configured_timeout_sec=30.0,
        )
    assert result["timed_out"] is True
    assert result["pass"] is False
    assert result["reward"] == 0


def _test_run_task_agent_phase_includes_prep() -> None:
    from sandbox_prep import SandboxPrepResult

    captured: list[dict[str, Any]] = []

    def slow_prep(*_args: Any, **_kwargs: Any) -> SandboxPrepResult:
        time.sleep(2.0)
        return SandboxPrepResult((), (), (), True, False)

    def slow_malvin(*_args: Any, **kwargs: Any) -> dict[str, Any]:
        remaining = kwargs.get("timeout_sec")
        if remaining is not None and remaining < 2.0:
            return {
                "exit_code": TIMEOUT_EXIT_CODE,
                "timed_out": True,
                "agent_seconds": remaining,
                "timeout_sec": kwargs.get("configured_timeout_sec"),
            }
        time.sleep(2.0)
        return {
            "exit_code": 0,
            "agent_seconds": 2.0,
            "timeout_sec": kwargs.get("configured_timeout_sec"),
        }

    def capture_artifacts(
        _run_root: Path,
        metadata: dict[str, Any],
        *_args: Any,
        **_kwargs: Any,
    ) -> None:
        captured.append(metadata)

    with tempfile.TemporaryDirectory() as tmp:
        task_dir, workspace = _minimal_timeout_task_tree(Path(tmp), agent_timeout_sec=2.5)
        run_root = Path(tmp) / "run"
        run_root.mkdir()
        t0 = time.monotonic()
        mod = sys.modules[__name__]
        with (
            patch.object(mod, "prepare_task_sandbox", slow_prep),
            patch.object(mod, "run_malvin", slow_malvin),
            patch.object(mod, "write_plan_and_checks"),
            patch.object(mod, "ensure_deepswe_malvin_config"),
            patch.object(mod, "_write_host_run_artifacts", capture_artifacts),
            patch.object(mod, "_print_evaluation_summary"),
            patch.object(mod, "_exit_from_evaluation"),
        ):
            run_task(
                local_task_name=None,
                task_dir=task_dir,
                workspace=workspace,
                results_dir=run_root,
                malvin_command="code",
                checks_override="true",
                runtime="host",
                skip_materialize=True,
                grade_only=False,
                skip_grade=True,
                apply_solution=False,
                reset_workspace_flag=False,
                docker_image=None,
                dry_run=False,
                malvin_args=(),
            )
        elapsed = time.monotonic() - t0
        assert elapsed < 3.5, elapsed
        assert captured, "expected metadata write"
        agent = captured[0].get("agent") or {}
        assert agent.get("timed_out") is True


def _test_combined_path_agent_timeout_still_grades() -> None:
    from sandbox_prep import SandboxPrepResult

    grade_calls: list[float | None] = []

    def fast_prep(*_args: Any, **_kwargs: Any) -> SandboxPrepResult:
        return SandboxPrepResult((), (), (), True, False)

    def timeout_malvin(*_args: Any, **_kwargs: Any) -> dict[str, Any]:
        return {
            "exit_code": TIMEOUT_EXIT_CODE,
            "timed_out": True,
            "agent_seconds": 0.0,
            "timeout_sec": 2.5,
        }

    def capture_grade(*_args: Any, **kwargs: Any) -> dict[str, Any]:
        grade_calls.append(kwargs.get("timeout_sec"))
        return {"pass": True, "reward": 1, "verifier_exit_code": 0}

    with tempfile.TemporaryDirectory() as tmp:
        task_dir, workspace = _minimal_timeout_task_tree(Path(tmp))
        run_root = Path(tmp) / "run"
        run_root.mkdir()
        mod = sys.modules[__name__]
        with (
            patch.object(mod, "prepare_task_sandbox", fast_prep),
            patch.object(mod, "run_malvin", timeout_malvin),
            patch.object(mod, "write_plan_and_checks"),
            patch.object(mod, "ensure_deepswe_malvin_config"),
            patch.object(mod, "grade_workspace", capture_grade),
            patch.object(mod, "_write_host_run_artifacts"),
            patch.object(mod, "_print_evaluation_summary"),
            patch.object(mod, "_exit_from_evaluation"),
        ):
            run_task(
                local_task_name=None,
                task_dir=task_dir,
                workspace=workspace,
                results_dir=run_root,
                malvin_command="code",
                checks_override="true",
                runtime="host",
                skip_materialize=True,
                grade_only=False,
                skip_grade=False,
                apply_solution=False,
                reset_workspace_flag=False,
                docker_image="fake:local",
                dry_run=False,
                malvin_args=(),
            )
    assert len(grade_calls) == 1


def _test_finalize_modal_eval_skips_agent_exit_on_timed_out() -> None:
    from deepswe_modal import finalize_modal_eval

    with tempfile.TemporaryDirectory() as tmp:
        run_root = Path(tmp)
        spec = TaskSpec(
            task_dir=Path(tmp),
            task_id="t",
            base_commit="HEAD",
            docker_image="fake:local",
            dockerfile=Path(tmp) / "Dockerfile",
            instruction=Path(tmp) / "instruction.md",
            tests_dir=Path(tmp) / "tests",
            test_sh=Path(tmp) / "tests" / "test.sh",
            solution_patch=None,
            repository_url=None,
            agent_timeout_sec=3600.0,
            verifier_timeout_sec=1800.0,
            environment_memory_mb=4096,
        )
        finalize_modal_eval(
            run_root=run_root,
            spec=spec,
            workspace=Path(tmp),
            malvin_command="code",
            malvin_args=(),
            grade_only=False,
            agent_result={"timed_out": True, "exit_code": TIMEOUT_EXIT_CODE},
            grade_result={"pass": True, "reward": 1},
        )


def _test_run_local_eval_in_docker_passes_backstop_timeout() -> None:
    captured: list[float | None] = []

    def fake_run(cmd: list[str], **kwargs: Any) -> MagicMock:
        captured.append(kwargs.get("timeout"))
        proc = MagicMock()
        proc.returncode = 0
        return proc

    with tempfile.TemporaryDirectory() as tmp:
        task_dir, workspace = _minimal_timeout_task_tree(
            Path(tmp),
            agent_timeout_sec=5400.0,
            verifier_timeout_sec=1800.0,
        )
        spec = parse_task_dir(task_dir)
        run_root = Path(tmp) / "run"
        run_root.mkdir()
        mod = sys.modules[__name__]
        with (
            patch.object(mod, "validate_toolchain_repos", return_value=Path("/fake/malvin")),
            patch.object(mod, "build_local_agent_image", return_value="deepswe-test:agent"),
            patch("subprocess.run", side_effect=fake_run),
        ):
            run_local_eval_in_docker(
                spec,
                task_dir,
                workspace,
                run_root,
                malvin_command="code",
                malvin_args=(),
                grade_only=False,
                skip_grade=False,
                apply_solution=False,
                reset_workspace_flag=False,
                checks_override=None,
                docker_image="fake:local",
                dry_run=False,
            )
    assert captured == [6300.0, 2700.0], captured


def run_self_tests() -> None:
    _test_malvin_repo_root()
    _test_default_deepswe_tasks_root()
    _test_resolve_local_task_dir()
    _test_local_agent_image_tag()
    _test_docker_local_eval_cmd()
    _test_solve_dry_run()
    _test_solve_modal_dry_run()
    _test_solve_modal_missing_credentials()
    _test_solve_modal_full_dry_run()
    _test_solve_resets_workspace_for_agent_runs()
    _test_solve_local_dry_run_passes_reset()
    _test_solve_command_in_help()
    _test_task_name_shorthand_routes_to_solve()
    _test_bare_invocation_shows_usage()
    _test_list_deepswe_tasks()
    _test_read_task_language()
    _test_list_deepswe_tasks_with_language()
    _test_discover_deepswe_checks_minimal()
    _test_discover_deepswe_checks_python_repo()
    _test_discover_deepswe_checks_stestr_repo()
    _test_discover_deepswe_checks_stestr_drops_stale_pytest()
    _test_discover_deepswe_checks_precommit()
    _test_discover_deepswe_checks_existing_malvin_checks()
    _test_write_plan_and_checks_discovers()
    _test_parse_task_dir_does_not_require_test_sh()
    _test_validate_verifier_paths_fails_without_test_sh()
    _test_scan_pytest_monkeypatch_hooks()
    _test_scan_class_level_attributes()
    _test_patch_surface_targets_prefers_config_style_classes()
    _test_render_patch_surface_probe_roundtrip()
    _test_write_plan_and_checks_includes_patch_surface_probe()
    _test_malvin_mem_limit_gb_from_task_memory()
    _test_ensure_deepswe_malvin_config_seeds_home_config()
    _test_ensure_deepswe_malvin_config_skips_default_memory()
    _test_prepare_task_sandbox_dry_run()
    _test_remaining_sec_floors_at_zero()
    _test_run_with_timeout_kills_slow_command()
    _test_exit_after_agent_timeout_grade_pass()
    _test_exit_after_agent_timeout_grade_fail()
    _test_agent_timeout_skip_grade_exits_zero()
    _test_verifier_timeout_forces_fail()
    _test_run_task_agent_phase_includes_prep()
    _test_combined_path_agent_timeout_still_grades()
    _test_run_local_eval_in_docker_passes_backstop_timeout()
    _test_finalize_modal_eval_skips_agent_exit_on_timed_out()
    _test_tasks_command()
    _test_is_modal_spend_limit_error()
    _test_solve_modal_spend_limit_falls_back_to_local_dry_run()
    _test_ephemeral_cache_find_expr()
    _test_purge_root_owned_ephemeral_caches_docker_cmd()
    _test_reset_workspace_removes_user_pycache()
    _test_purge_root_owned_sandbox_artifacts_docker()
    _test_purge_root_owned_ephemeral_caches_docker()
    _test_scan_class_level_attributes_skips_non_utf8()
    _test_run_malvin_uses_plan_name_not_at_notation()
    _test_run_malvin_do_uses_prompt_not_plan()
    _test_resolve_malvin_cmd_prefers_repo_target()
    _test_relay_subprocess_stdout_sets_force_tee_env()
    _test_run_command_accepts_hello()
    _test_hello_host_relays_stdout()
    _test_hello_host_rejects_local()
    _test_hello_modal_dry_run()
    _test_hello_command_in_help()
    _test_local_grade_only_apply_solution()
    click.echo("deepswe_run self-tests passed")






if __name__ == "__main__":
    cli()
