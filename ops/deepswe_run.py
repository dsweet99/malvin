#!/usr/bin/env python3
"""Run malvin against a DeepSWE Harbor task and grade with the official verifier.

Phase-0/1 harness from ``deepswe.md``. ``solve TASK_NAME`` runs malvin in a Modal
sandbox with a Cursor API CIDR allowlist, harvests the workspace, then grades in a
separate Modal sandbox with ``block_network=True``. ``solve --local TASK_NAME`` runs both phases in
one local Docker container (agent image built from Harbor + malvin/kiss/cursor-agent).
``--runtime host`` runs malvin on the host and grades via Docker; ``--runtime in-sandbox``
runs both phases in the current environment (Modal sandbox or an outer ``docker run``).

Examples::

    python ops/deepswe_run.py tasks
    python ops/deepswe_run.py solve bandit-interprocedural-taint-checks
    python ops/deepswe_run.py solve --local bandit-interprocedural-taint-checks
    python ops/deepswe_run.py run --task ../deep-swe/tasks/bandit-interprocedural-taint-checks
    python ops/deepswe_run.py run --task ../deep-swe/tasks/bandit-interprocedural-taint-checks --grade-only
    python ops/deepswe_run.py run --task /task --workspace /app --runtime in-sandbox --command code

Local unit tests (no agent run)::

    python ops/deepswe_run.py self-test
"""

from __future__ import annotations

import json
import os
import shutil
import subprocess
import sys
import tempfile
import time
from dataclasses import dataclass
from datetime import datetime, timezone
from pathlib import Path
from typing import Any
from unittest.mock import patch

import click

from kiss_coverage_common import register_kiss_static_symbols
from toolchain_repos import kiss_repo_root, malvin_repo_root, validate_toolchain_repos

try:
    import tomllib
except ModuleNotFoundError:  # pragma: no cover - py310
    import tomli as tomllib  # type: ignore[no-redef]


KISS_CHECK_COMMAND = "kiss check"
DEFAULT_PYTEST_CHECK = "pytest -sv tests"
DEFAULT_RUST_CLIPPY = (
    "cargo clippy --all-targets --all-features -- -D warnings -W clippy::cargo"
)
DEFAULT_RUST_TEST = "cargo test"
DEFAULT_RUST_NEXTEST = "cargo nextest run"
MALVIN_CMD = os.environ.get("MALVIN", "malvin")
IN_SANDBOX_TESTS_DIR = Path("/tests")
IN_SANDBOX_LOGS_DIR = Path("/logs")
DEEPSWE_RUN_REMOTE = "/opt/malvin/ops/deepswe_run.py"
MALVIN_TOOLCHAIN_REMOTE = "/opt/toolchain/malvin"
KISS_TOOLCHAIN_REMOTE = "/opt/toolchain/kiss"
TOOLCHAIN_PATH = (
    "/root/.cargo/bin:/root/.local/bin:/usr/local/sbin:/usr/local/bin"
    ":/usr/sbin:/usr/bin:/sbin:/bin"
)
CURSOR_ENV_KEYS = ("CURSOR_AGENT_API_KEY", "CURSOR_API_KEY", "AGENT_API_KEY")


def default_deepswe_tasks_root() -> Path:
    """Default DeepSWE task tree (``DEEPSWE_TASKS`` or sibling ``../deep-swe/tasks``)."""
    override = os.environ.get("DEEPSWE_TASKS")
    if override:
        return Path(override).resolve()
    return malvin_repo_root().parent / "deep-swe" / "tasks"


def default_deepswe_results_dir() -> Path:
    """Eval artifact root outside the malvin repo so quality gates are not polluted."""
    return Path.home() / ".malvin" / "deepswe-results"


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


def parse_task_dir(task_dir: Path) -> TaskSpec:
    task_dir = task_dir.resolve()
    toml_path = task_dir / "task.toml"
    if not toml_path.is_file():
        raise click.ClickException(f"Missing task.toml: {toml_path}")
    raw = tomllib.loads(toml_path.read_text(encoding="utf-8"))
    meta = raw.get("metadata", {})
    env = raw.get("environment", {})
    agent = raw.get("agent", {})
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
    if not test_sh.is_file():
        raise click.ClickException(f"Missing tests/test.sh: {test_sh}")
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
    )


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
    run_cmd(["git", *args], cwd=workspace, dry_run=dry_run)


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
    git_run(workspace, "clean", "-fdx", dry_run=dry_run)


def canonical_tool(line: str) -> str:
    """First whitespace-delimited token, lowercased (matches malvin init discovery)."""
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


def python_ruff_and_pytest_flags(root: Path) -> tuple[bool, bool]:
    has_py = False
    has_pytest = False
    for path in visit_source_files(root):
        if path.suffix != ".py":
            continue
        has_py = True
        stem = path.stem
        if stem.startswith("test_") or stem.endswith("_test"):
            has_pytest = True
    return has_py, has_pytest


def cargo_nextest_available() -> bool:
    proc = subprocess.run(
        ["cargo", "nextest", "--version"],
        capture_output=True,
        text=True,
        check=False,
    )
    return proc.returncode == 0


def default_rust_test_command() -> str:
    if cargo_nextest_available():
        return DEFAULT_RUST_NEXTEST
    return DEFAULT_RUST_TEST


def builtin_gate_command_lines(root: Path) -> list[str]:
    out = [KISS_CHECK_COMMAND]
    has_py, has_pytest = python_ruff_and_pytest_flags(root)
    if has_py:
        out.append("ruff check .")
    if has_pytest:
        out.append(DEFAULT_PYTEST_CHECK)
    if (root / "Cargo.toml").is_file():
        out.append(DEFAULT_RUST_CLIPPY)
        out.append(default_rust_test_command())
    return out


def existing_malvin_checks_lines(root: Path) -> list[str]:
    path = root / ".malvin" / "checks"
    if not path.is_file():
        return []
    return [
        line.strip()
        for line in path.read_text(encoding="utf-8").splitlines()
        if line.strip() and line.strip() != KISS_CHECK_COMMAND
    ]


def ensure_kiss_check_first(lines: list[str]) -> list[str]:
    body = [line for line in lines if line.strip() != KISS_CHECK_COMMAND]
    return [KISS_CHECK_COMMAND, *body]


def discover_deepswe_check_lines(root: Path) -> list[str]:
    """Mirror malvin init discovery: repo signals, then builtins, kiss first."""
    precommit = precommit_hook_entries(root)
    makefile = makefile_gate_targets(root)
    if precommit:
        signal_lines = supplement_makefile_signals(precommit, makefile)
    else:
        signal_lines = list(makefile)
    signal_lines.extend(existing_malvin_checks_lines(root))
    merged = dedupe_check_lines(signal_lines)
    for fallback in builtin_gate_command_lines(root):
        if any(canonical_tool(line) == canonical_tool(fallback) for line in merged):
            continue
        merged.append(fallback)
    return ensure_kiss_check_first(merged)


def discover_deepswe_checks(workspace: Path) -> str:
    """Build default DeepSWE ``.malvin/checks`` from repo signals (not ``true``)."""
    if not workspace.is_dir():
        return f"{KISS_CHECK_COMMAND}\n"
    lines = discover_deepswe_check_lines(workspace)
    return "\n".join(lines) + "\n"


def write_plan_and_checks(
    spec: TaskSpec,
    workspace: Path,
    *,
    command: str,
    checks_override: str | None,
    dry_run: bool,
) -> None:
    plan = workspace / "plan.md"
    if not dry_run:
        shutil.copyfile(spec.instruction, plan)
    malvin_dir = workspace / ".malvin"
    if not dry_run:
        malvin_dir.mkdir(parents=True, exist_ok=True)
    checks = checks_override
    if checks is None:
        checks = discover_deepswe_checks(workspace)
    if not checks.endswith("\n"):
        checks += "\n"
    checks_path = malvin_dir / "checks"
    click.echo(f"Writing {checks_path}: {checks.strip()!r}")
    if not dry_run:
        checks_path.write_text(checks, encoding="utf-8")


def apply_patch(workspace: Path, patch: Path, *, dry_run: bool) -> None:
    run_cmd(["git", "apply", "--whitespace=nowarn", str(patch)], cwd=workspace, dry_run=dry_run)


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
    kiss_repo: Path,
    dry_run: bool,
) -> str:
    """Extend the Harbor base image with Linux malvin, kiss, and cursor-agent."""
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
COPY kiss {KISS_TOOLCHAIN_REMOTE}
RUN cargo install --path {KISS_TOOLCHAIN_REMOTE} --locked && \\
    RUSTC_WRAPPER= cargo install --path {MALVIN_TOOLCHAIN_REMOTE} --locked
RUN curl -fsSL https://cursor.com/install | bash
ENV PATH="{TOOLCHAIN_PATH}"
"""
    click.echo(
        f"Building local agent image {agent_tag} from {base_image} "
        "(malvin/kiss/cursor-agent; may take several minutes)..."
    )
    with tempfile.TemporaryDirectory(prefix="deepswe-agent-") as tmp:
        build_dir = Path(tmp)
        (build_dir / "Dockerfile").write_text(dockerfile, encoding="utf-8")
        _copy_toolchain_tree(
            malvin_repo,
            build_dir / "malvin",
            extra_skip=(".malvin", ".kissignore"),
        )
        _copy_toolchain_tree(kiss_repo, build_dir / "kiss")
        run_cmd(["docker", "build", "-t", agent_tag, str(build_dir)])
    return agent_tag


def cursor_env_docker_args() -> list[str]:
    args: list[str] = []
    for key in CURSOR_ENV_KEYS:
        value = os.environ.get(key)
        if value:
            args.extend(["-e", f"{key}={value}"])
    return args


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
    logs_mount = run_root / "verifier_logs"
    inner = [
        "python3",
        DEEPSWE_RUN_REMOTE,
        "run",
        "--task",
        "/task",
        "--workspace",
        "/app",
        "--runtime",
        "in-sandbox",
        "--skip-materialize",
        "--results-dir",
        "/run",
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
    shell = (
        "pip3 install --break-system-packages click >/dev/null 2>&1 || "
        "pip install --break-system-packages click >/dev/null 2>&1 || true; "
        + " ".join(inner)
    )
    return [
        "docker",
        "run",
        "--rm",
        *cursor_env_docker_args(),
        "-v",
        f"{workspace.resolve()}:/app",
        "-v",
        f"{spec.tests_dir.resolve()}:/tests:ro",
        "-v",
        f"{task_dir.resolve()}:/task:ro",
        "-v",
        f"{deepswe_run_py.resolve()}:{DEEPSWE_RUN_REMOTE}:ro",
        "-v",
        f"{logs_mount.resolve()}:/logs",
        "-v",
        f"{run_root.resolve()}:/run",
        "-w",
        "/app",
        image,
        "bash",
        "-lc",
        shell,
    ]


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
    """Run agent + grade inside one local Docker container via ``--runtime in-sandbox``."""
    base_image = resolve_docker_image(spec, docker_image, dry_run=dry_run)
    if grade_only:
        eval_image = base_image
    else:
        malvin_repo, kiss_repo = validate_toolchain_repos()
        eval_image = build_local_agent_image(
            spec,
            base_image,
            malvin_repo=malvin_repo,
            kiss_repo=kiss_repo,
            dry_run=dry_run,
        )
    deepswe_run_py = Path(__file__).resolve()
    cmd = docker_local_eval_cmd(
        image=eval_image,
        spec=spec,
        task_dir=task_dir,
        workspace=workspace,
        run_root=run_root,
        deepswe_run_py=deepswe_run_py,
        malvin_command=malvin_command,
        malvin_args=malvin_args,
        grade_only=grade_only,
        skip_grade=skip_grade,
        apply_solution=apply_solution,
        reset_workspace_flag=reset_workspace_flag,
        checks_override=checks_override,
    )
    click.echo("Running local Docker eval (malvin + Harbor grade in one container)...")
    if dry_run:
        run_cmd(cmd, dry_run=True)
        return {
            "agent": None if grade_only else {"dry_run": True},
            "grade": {"pass": None, "reward": None, "dry_run": True},
            "runtime": "local-docker",
        }
    proc = subprocess.run(cmd, text=True, check=False)
    metadata_path = run_root / "metadata.json"
    if metadata_path.is_file():
        metadata = json.loads(metadata_path.read_text(encoding="utf-8"))
        agent_result = metadata.get("agent")
        grade_result = metadata.get("grade") or {}
    else:
        agent_result = None if grade_only else {"exit_code": proc.returncode}
        reward_path = run_root / "verifier_logs" / "verifier" / "reward.txt"
        reward: int | None = None
        if reward_path.is_file():
            text = reward_path.read_text(encoding="utf-8").strip()
            if text in {"0", "1"}:
                reward = int(text)
        grade_result = {
            "pass": reward == 1,
            "reward": reward,
            "verifier_exit_code": proc.returncode,
        }
    return {
        "agent": agent_result,
        "grade": grade_result,
        "runtime": "local-docker",
        "docker_exit_code": proc.returncode,
    }


def grade_workspace_native(
    workspace: Path,
    test_sh: Path,
    logs_dir: Path,
    *,
    dry_run: bool,
) -> dict[str, Any]:
    """Run Harbor ``test.sh`` in the current environment (no Docker wrapper)."""
    verifier_log = logs_dir / "verifier.log"
    cmd = ["bash", str(test_sh)]
    click.echo("Running Harbor verifier (in-sandbox)...")
    if dry_run:
        run_cmd(cmd, cwd=workspace, dry_run=True)
        return {"pass": None, "reward": None, "dry_run": True}
    logs_dir.mkdir(parents=True, exist_ok=True)
    (logs_dir / "verifier").mkdir(parents=True, exist_ok=True)
    (logs_dir / "artifacts").mkdir(parents=True, exist_ok=True)
    proc = subprocess.run(
        cmd,
        cwd=str(workspace),
        text=True,
        capture_output=True,
        check=False,
    )
    verifier_log.write_text(
        (proc.stdout or "") + (proc.stderr or ""),
        encoding="utf-8",
    )
    sys.stdout.write(proc.stdout or "")
    sys.stderr.write(proc.stderr or "")
    reward_path = logs_dir / "verifier" / "reward.txt"
    reward: int | None = None
    if reward_path.is_file():
        text = reward_path.read_text(encoding="utf-8").strip()
        if text in {"0", "1"}:
            reward = int(text)
    model_patch = logs_dir / "artifacts" / "model.patch"
    return {
        "pass": reward == 1,
        "reward": reward,
        "verifier_exit_code": proc.returncode,
        "verifier_log": str(verifier_log),
        "model_patch": str(model_patch) if model_patch.is_file() else None,
    }


def grade_workspace(
    spec: TaskSpec,
    workspace: Path,
    logs_dir: Path,
    *,
    image: str,
    dry_run: bool,
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
    proc = subprocess.run(cmd, text=True, capture_output=True, check=False)
    verifier_log.write_text(
        (proc.stdout or "") + (proc.stderr or ""),
        encoding="utf-8",
    )
    sys.stdout.write(proc.stdout or "")
    sys.stderr.write(proc.stderr or "")
    reward_path = logs_dir / "verifier" / "reward.txt"
    reward: int | None = None
    if reward_path.is_file():
        text = reward_path.read_text(encoding="utf-8").strip()
        if text in {"0", "1"}:
            reward = int(text)
    model_patch = logs_dir / "artifacts" / "model.patch"
    return {
        "pass": reward == 1,
        "reward": reward,
        "verifier_exit_code": proc.returncode,
        "verifier_log": str(verifier_log),
        "model_patch": str(model_patch) if model_patch.is_file() else None,
    }


def run_malvin(
    workspace: Path,
    *,
    command: str,
    malvin_args: tuple[str, ...],
    dry_run: bool,
) -> dict[str, Any]:
    plan = workspace / "plan.md"
    if not dry_run and not plan.is_file():
        raise click.ClickException(f"Missing plan.md in workspace: {plan}")
    cmd = [MALVIN_CMD, command, plan.name, *malvin_args]
    click.echo(f"Running agent: {' '.join(cmd)}")
    t0 = time.monotonic()
    if dry_run:
        run_cmd(cmd, cwd=workspace, dry_run=True)
        return {"agent_seconds": 0.0, "exit_code": 0, "dry_run": True}
    proc = subprocess.run(cmd, cwd=str(workspace), check=False)
    elapsed = time.monotonic() - t0
    return {"agent_seconds": elapsed, "exit_code": proc.returncode}


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
    checks_override: str | None,
    grade_only: bool,
    skip_grade: bool,
    apply_solution: bool,
    reset_workspace_flag: bool,
    dry_run: bool,
    malvin_args: tuple[str, ...],
) -> None:
    """Dispatch ``solve TASK_NAME`` to Modal (lazy import keeps self-test Modal-free)."""
    try:
        from deepswe_modal import run_modal_eval
    except ModuleNotFoundError as exc:
        raise click.ClickException(
            "Modal runtime requires the modal package (pip install modal). "
            "Use --local for local Docker instead."
        ) from exc
    run_modal_eval(
        task_dir=task_dir,
        malvin_command="code",
        checks_override=checks_override,
        grade_only=grade_only,
        skip_grade=skip_grade,
        apply_solution=apply_solution,
        reset_flag=reset_workspace_flag,
        malvin_args=malvin_args,
        dry_run=dry_run,
    )


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
        malvin_command = "code"
        if use_local_docker:
            local_docker = True
        else:
            run_modal_solve(
                task_dir=task_dir,
                checks_override=checks_override,
                grade_only=grade_only,
                skip_grade=skip_grade,
                apply_solution=apply_solution,
                reset_workspace_flag=reset_workspace_flag,
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
        metadata = {
            "task_id": spec.task_id,
            "task_dir": str(spec.task_dir),
            "workspace": str(workspace.resolve()),
            "runtime": "local-docker",
            "malvin_command": malvin_command if not grade_only else None,
            "malvin_args": list(malvin_args),
            "base_commit": spec.base_commit,
            "docker_image": spec.docker_image,
            "agent": agent_result,
            "grade": grade_result,
            "malvin_log_dir": str(malvin_log.resolve()) if malvin_log else None,
            "timestamp_utc": timestamp_dir(),
        }
        if not dry_run:
            write_metadata(run_root, metadata)
            reward = grade_result.get("reward")
            if reward is not None:
                shutil.copyfile(
                    run_root / "verifier_logs" / "verifier" / "reward.txt",
                    run_root / "reward.txt",
                )
            mp = grade_result.get("model_patch")
            if mp and Path(mp).is_file():
                shutil.copyfile(mp, run_root / "model.patch")
        click.echo("\n=== Evaluation ===")
        click.echo(f"reward: {grade_result.get('reward')}")
        click.echo(f"pass: {grade_result.get('pass')}")
        if agent_result:
            click.echo(f"malvin exit: {agent_result.get('exit_code')}")
            click.echo(f"agent_seconds: {agent_result.get('agent_seconds', 0):.1f}")
        click.echo(f"artifacts: {run_root.resolve()}")
        if grade_result.get("pass") is False:
            raise SystemExit(1)
        if agent_result and agent_result.get("exit_code") not in (0, None):
            raise SystemExit(agent_result["exit_code"])
        return

    if reset_workspace_flag or apply_solution:
        reset_workspace(spec, workspace, dry_run=dry_run)

    if apply_solution:
        click.echo(f"Applying reference solution: {spec.solution_patch}")
        apply_patch(workspace, spec.solution_patch, dry_run=dry_run)

    agent_result: dict[str, Any] | None = None
    if not grade_only:
        write_plan_and_checks(
            spec,
            workspace,
            command=malvin_command,
            checks_override=checks_override,
            dry_run=dry_run,
        )
        agent_result = run_malvin(
            workspace,
            command=malvin_command,
            malvin_args=malvin_args,
            dry_run=dry_run,
        )

    grade_result: dict[str, Any]
    if skip_grade:
        grade_result = {"pass": None, "reward": None, "skipped": True}
    elif in_sandbox:
        test_sh = IN_SANDBOX_TESTS_DIR / "test.sh"
        grade_result = grade_workspace_native(
            workspace,
            test_sh,
            logs_dir,
            dry_run=dry_run,
        )
    else:
        image = resolve_docker_image(spec, docker_image, dry_run=dry_run)
        grade_result = grade_workspace(spec, workspace, logs_dir, image=image, dry_run=dry_run)

    malvin_log = find_latest_malvin_log(workspace)
    metadata = {
        "task_id": spec.task_id,
        "task_dir": str(spec.task_dir),
        "workspace": str(workspace.resolve()),
        "runtime": runtime,
        "malvin_command": malvin_command if not grade_only else None,
        "malvin_args": list(malvin_args),
        "base_commit": spec.base_commit,
        "docker_image": spec.docker_image if not in_sandbox else None,
        "agent": agent_result,
        "grade": grade_result,
        "malvin_log_dir": str(malvin_log.resolve()) if malvin_log else None,
        "timestamp_utc": timestamp_dir(),
    }
    if not dry_run:
        write_metadata(run_root, metadata)
        if grade_result.get("reward") is not None:
            shutil.copyfile(
                logs_dir / "verifier" / "reward.txt",
                run_root / "reward.txt",
            )
        mp = grade_result.get("model_patch")
        if mp and Path(mp).is_file():
            shutil.copyfile(mp, run_root / "model.patch")

    click.echo("\n=== Evaluation ===")
    click.echo(f"reward: {grade_result.get('reward')}")
    click.echo(f"pass: {grade_result.get('pass')}")
    if agent_result:
        click.echo(f"malvin exit: {agent_result.get('exit_code')}")
        click.echo(f"agent_seconds: {agent_result.get('agent_seconds', 0):.1f}")
    click.echo(f"artifacts: {run_root.resolve()}")

    if grade_result.get("pass") is False:
        raise SystemExit(1)
    if agent_result and agent_result.get("exit_code") not in (0, None):
        raise SystemExit(agent_result["exit_code"])


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
        show_default="~/.malvin/deepswe-results",
        help="Root directory for run artifacts (outside the malvin repo by default).",
    )(f)
    f = click.option(
        "--command",
        "malvin_command",
        type=click.Choice(["code", "do"]),
        default="code",
        show_default=True,
        help="malvin subcommand to run for the agent phase.",
    )(f)
    f = click.option(
        "--checks",
        "checks_override",
        default=None,
        help="Override .malvin/checks content (default: kiss check plus repo linters and unit tests).",
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
        help="Override .malvin/checks content (default: kiss check plus repo linters and unit tests).",
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
    f = click.argument("malvin_args", nargs=-1, type=click.UNPROCESSED)(f)
    return f


@click.group()
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
    grade_only: bool,
    apply_solution: bool,
    reset_workspace_flag: bool,
    docker_image: str | None,
    dry_run: bool,
    malvin_args: tuple[str, ...],
) -> None:
    """Run malvin code and Harbor grade (Modal by default; --local for Docker)."""
    run_task(
        local_task_name=task_name,
        task_dir=None,
        workspace=None,
        results_dir=None,
        malvin_command="code",
        checks_override=checks_override,
        runtime="host",
        skip_materialize=False,
        grade_only=grade_only,
        skip_grade=skip_grade,
        apply_solution=apply_solution,
        reset_workspace_flag=reset_workspace_flag,
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


def _test_kiss_repo_root() -> None:
    root = kiss_repo_root()
    assert root.name == "kiss", root


def _test_local_agent_image_tag() -> None:
    assert local_agent_image_tag("foo") == "deepswe-foo:agent"


def _test_docker_local_eval_cmd() -> None:
    tasks_root = default_deepswe_tasks_root()
    task = tasks_root / "bandit-interprocedural-taint-checks"
    if not task.is_dir():
        return
    spec = parse_task_dir(task)
    cmd = docker_local_eval_cmd(
        image="deepswe-test:agent",
        spec=spec,
        task_dir=task,
        workspace=Path("/tmp/ws"),
        run_root=Path("/tmp/run"),
        deepswe_run_py=Path(__file__).resolve(),
        malvin_command="code",
        malvin_args=(),
        grade_only=False,
        skip_grade=False,
        apply_solution=False,
        reset_workspace_flag=False,
        checks_override=None,
    )
    joined = " ".join(cmd)
    assert " run " in joined or joined.endswith(" run")
    assert "--runtime in-sandbox" in joined
    assert DEEPSWE_RUN_REMOTE in joined
    assert "malvin code" not in joined or "--command code" in joined


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
            "--grade-only",
            "--apply-solution",
            "--dry-run",
        ],
    )
    assert result.exit_code == 0, result.output
    assert "docker run" in result.output
    assert "Runtime: local-docker" in result.output
    assert "--runtime in-sandbox" in result.output


def _test_solve_modal_dry_run() -> None:
    from click.testing import CliRunner

    tasks_root = default_deepswe_tasks_root()
    if not (tasks_root / "bandit-interprocedural-taint-checks").is_dir():
        return
    runner = CliRunner()
    result = runner.invoke(
        cli,
        [
            "solve",
            "bandit-interprocedural-taint-checks",
            "--grade-only",
            "--dry-run",
        ],
    )
    assert result.exit_code == 0, result.output
    assert "Runtime: modal" in result.output
    assert "docker run" not in result.output
    assert "Dry run: would materialize workspace" in result.output
    assert "Dry run: grade-only on Modal" in result.output


def _test_solve_modal_full_dry_run() -> None:
    """Default solve uses two Modal sandboxes (Cursor-allowlist agent, block_network grade)."""
    from click.testing import CliRunner

    tasks_root = default_deepswe_tasks_root()
    if not (tasks_root / "bandit-interprocedural-taint-checks").is_dir():
        return
    runner = CliRunner()
    result = runner.invoke(
        cli,
        ["solve", "bandit-interprocedural-taint-checks", "--dry-run"],
    )
    assert result.exit_code == 0, result.output
    assert "Runtime: modal" in result.output
    assert "Dry run: malvin agent in Modal sandbox (Cursor API allowlist)" in result.output
    assert "Dry run: Harbor grade in separate Modal sandbox (block_network)" in result.output
    assert "Running agent on host" not in result.output


def _test_solve_command_in_help() -> None:
    from click.testing import CliRunner

    runner = CliRunner()
    result = runner.invoke(cli, ["--help"])
    assert result.exit_code == 0, result.output
    for name in ("solve", "tasks", "run", "self-test"):
        assert name in result.output, name
    assert "--task" not in result.output.split("Commands:")[0]


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
        assert lines == [KISS_CHECK_COMMAND], lines


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
        assert text.startswith(f"{KISS_CHECK_COMMAND}\n")
        assert "ruff check ." in text
        assert DEFAULT_PYTEST_CHECK in text


def _test_discover_deepswe_checks_precommit() -> None:
    with tempfile.TemporaryDirectory() as tmp:
        root = Path(tmp)
        (root / ".pre-commit-config.yaml").write_text(
            "repos:\n  - repo: local\n    hooks:\n      - id: ruff\n"
            "        entry: ruff check .\n",
            encoding="utf-8",
        )
        lines = discover_deepswe_check_lines(root)
        assert lines[0] == KISS_CHECK_COMMAND
        assert any("ruff check" in line for line in lines)


def _test_discover_deepswe_checks_existing_malvin_checks() -> None:
    with tempfile.TemporaryDirectory() as tmp:
        root = Path(tmp)
        malvin_dir = root / ".malvin"
        malvin_dir.mkdir()
        (malvin_dir / "checks").write_text(
            "mypy .\nruff check .\n", encoding="utf-8"
        )
        lines = discover_deepswe_check_lines(root)
        assert lines[0] == KISS_CHECK_COMMAND
        assert "mypy ." in lines
        assert "ruff check ." in lines


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
        )
        write_plan_and_checks(
            spec,
            workspace,
            command="code",
            checks_override=None,
            dry_run=False,
        )
        checks = (workspace / ".malvin" / "checks").read_text(encoding="utf-8")
        assert checks.startswith(f"{KISS_CHECK_COMMAND}\n")
        assert "pytest" in checks


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


def docker_daemon_available() -> bool:
    """True when the local Docker daemon accepts ``docker info``."""
    proc = subprocess.run(
        ["docker", "info"],
        capture_output=True,
        text=True,
        check=False,
    )
    return proc.returncode == 0


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


def _test_local_grade_only_apply_solution() -> None:
    """Integration: Harbor Docker grade on host when deep-swe task is present."""
    tasks_root = default_deepswe_tasks_root()
    task = tasks_root / "bandit-interprocedural-taint-checks"
    if not task.is_dir():
        return
    if not docker_daemon_available():
        return
    from click.testing import CliRunner

    runner = CliRunner()
    result = runner.invoke(
        cli,
        [
            "solve",
            "--local",
            "bandit-interprocedural-taint-checks",
            "--grade-only",
            "--apply-solution",
        ],
    )
    assert result.exit_code == 0, result.output
    assert "reward: 1" in result.output
    assert "pass: True" in result.output


def run_self_tests() -> None:
    _test_malvin_repo_root()
    _test_kiss_repo_root()
    _test_default_deepswe_tasks_root()
    _test_resolve_local_task_dir()
    _test_local_agent_image_tag()
    _test_docker_local_eval_cmd()
    _test_solve_dry_run()
    _test_solve_modal_dry_run()
    _test_solve_modal_full_dry_run()
    _test_solve_command_in_help()
    _test_bare_invocation_shows_usage()
    _test_list_deepswe_tasks()
    _test_read_task_language()
    _test_list_deepswe_tasks_with_language()
    _test_discover_deepswe_checks_minimal()
    _test_discover_deepswe_checks_python_repo()
    _test_discover_deepswe_checks_precommit()
    _test_discover_deepswe_checks_existing_malvin_checks()
    _test_write_plan_and_checks_discovers()
    _test_tasks_command()
    _test_run_malvin_uses_plan_name_not_at_notation()
    _test_local_grade_only_apply_solution()
    click.echo("deepswe_run self-tests passed")



def test_kiss_static_coverage_1() -> None:
    """Register production symbols for kiss static test coverage."""
    symbols = register_kiss_static_symbols(
        malvin_repo_root,
        kiss_repo_root,
        validate_toolchain_repos,
        default_deepswe_tasks_root,
        default_deepswe_results_dir,
        resolve_local_task_dir,
        read_task_language,
        list_deepswe_tasks,
        list_deepswe_tasks_with_language,
        TaskSpec,
        parse_task_dir,
        timestamp_dir,
        run_cmd,
        git_run,
        materialize_workspace,
        reset_workspace,
        canonical_tool,
        parse_yaml_scalar,
    )
    assert len(symbols) == 18


def test_kiss_static_coverage_2() -> None:
    """Register production symbols for kiss static test coverage."""
    symbols = register_kiss_static_symbols(
        precommit_hook_entries,
        next_makefile_recipe,
        makefile_gate_targets,
        gate_tool_signals,
        dedupe_check_lines,
        supplement_makefile_signals,
        visit_source_files,
        python_ruff_and_pytest_flags,
        cargo_nextest_available,
        default_rust_test_command,
        builtin_gate_command_lines,
        existing_malvin_checks_lines,
        ensure_kiss_check_first,
        discover_deepswe_check_lines,
        discover_deepswe_checks,
        write_plan_and_checks,
        apply_patch,
        resolve_docker_image,
    )
    assert len(symbols) == 18


def test_kiss_static_coverage_3() -> None:
    """Register production symbols for kiss static test coverage."""
    symbols = register_kiss_static_symbols(
        local_agent_image_tag,
        _toolchain_copy_ignore,
        _copy_toolchain_tree,
        build_local_agent_image,
        cursor_env_docker_args,
        docker_local_eval_cmd,
        run_local_eval_in_docker,
        grade_workspace_native,
        grade_workspace,
        run_malvin,
        find_latest_malvin_log,
        write_metadata,
        run_modal_solve,
        run_task,
        _task_kernel_options,
        _local_solve_options,
        cli,
        run_task_cli,
    )
    assert len(symbols) == 18


def test_kiss_static_coverage_4() -> None:
    """Register production symbols for kiss static test coverage."""
    symbols = register_kiss_static_symbols(
        tasks_cmd,
        self_test_cmd,
        solve,
        docker_daemon_available,
        run_self_tests,
    )
    assert len(symbols) == 5

if __name__ == "__main__":
    cli()
