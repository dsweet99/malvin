#!/usr/bin/env python3
"""Run malvin against a DeepSWE Harbor task and grade with the official verifier.

Phase-0/1 harness from ``deepswe.md``. Agent runs on the host (Cursor API needs
egress); grading runs in the Harbor Docker image (air-gapped verifier).

Examples::

    python ops/deepswe_run.py --task ../deep-swe/tasks/bandit-interprocedural-taint-checks
    python ops/deepswe_run.py --task ../deep-swe/tasks/bandit-interprocedural-taint-checks --grade-only
    python ops/deepswe_run.py --task ../deep-swe/tasks/bandit-interprocedural-taint-checks --apply-solution --grade-only
    python ops/deepswe_run.py --task ../deep-swe/tasks/bandit-interprocedural-taint-checks --command code --no-tenacious --max-loops 1
"""

from __future__ import annotations

import json
import os
import shutil
import subprocess
import sys
import time
from dataclasses import dataclass
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

import click

try:
    import tomllib
except ModuleNotFoundError:  # pragma: no cover - py310
    import tomli as tomllib  # type: ignore[no-redef]


DEFAULT_CHECKS_CODE = "true\n"
DEFAULT_CHECKS_DO = "true\n"
MALVIN_CMD = os.environ.get("MALVIN", "malvin")


def default_deepswe_results_dir() -> Path:
    """Eval artifact root outside the malvin repo so quality gates are not polluted."""
    return Path.home() / ".malvin" / "deepswe-results"


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
        checks = DEFAULT_CHECKS_CODE if command == "code" else DEFAULT_CHECKS_DO
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


def grade_workspace(
    spec: TaskSpec,
    workspace: Path,
    logs_dir: Path,
    *,
    image: str,
    dry_run: bool,
) -> dict[str, Any]:
    logs_dir.mkdir(parents=True, exist_ok=True)
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
    if not plan.is_file():
        raise click.ClickException(f"Missing plan.md in workspace: {plan}")
    cmd = [MALVIN_CMD, command, f"@{plan.name}", *malvin_args]
    click.echo(f"Running agent: {' '.join(cmd)}")
    t0 = time.monotonic()
    if dry_run:
        run_cmd(cmd, cwd=workspace, dry_run=True)
        return {"agent_seconds": 0.0, "exit_code": 0, "dry_run": True}
    proc = subprocess.run(cmd, cwd=str(workspace), check=False)
    elapsed = time.monotonic() - t0
    return {"agent_seconds": elapsed, "exit_code": proc.returncode}


def find_latest_malvin_log() -> Path | None:
    logs_root = Path(".malvin/logs")
    if not logs_root.is_dir():
        return None
    candidates = sorted(logs_root.iterdir(), key=lambda p: p.stat().st_mtime, reverse=True)
    return candidates[0] if candidates else None


def write_metadata(out_dir: Path, payload: dict[str, Any]) -> None:
    out_dir.mkdir(parents=True, exist_ok=True)
    (out_dir / "metadata.json").write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")


@click.command(
    context_settings={
        "ignore_unknown_options": True,
        "allow_extra_args": True,
    },
)
@click.option(
    "--task",
    "task_dir",
    type=click.Path(exists=True, file_okay=False, path_type=Path),
    required=True,
    help="Path to a DeepSWE task directory (contains task.toml).",
)
@click.option(
    "--workspace",
    type=click.Path(file_okay=False, path_type=Path),
    default=None,
    help="Git checkout for the task repo (default: <results-dir>/<task-id>/workspace).",
)
@click.option(
    "--results-dir",
    type=click.Path(file_okay=False, path_type=Path),
    default=None,
    show_default="~/.malvin/deepswe-results",
    help="Root directory for run artifacts (outside the malvin repo by default).",
)
@click.option(
    "--command",
    "malvin_command",
    type=click.Choice(["code", "do"]),
    default="code",
    show_default=True,
    help="malvin subcommand to run for the agent phase.",
)
@click.option(
    "--checks",
    "checks_override",
    default=None,
    help="Override .malvin/checks content (default: pytest unit tests for code, true for do).",
)
@click.option(
    "--skip-grade",
    is_flag=True,
    help="Skip Harbor verifier grading (agent phase only).",
)
@click.option(
    "--grade-only",
    is_flag=True,
    help="Skip agent; grade the current workspace tree.",
)
@click.option(
    "--apply-solution",
    is_flag=True,
    help="Apply task solution/solution.patch before agent or grade (harness sanity check).",
)
@click.option(
    "--reset",
    "reset_workspace_flag",
    is_flag=True,
    help="Hard reset workspace to base_commit before run.",
)
@click.option(
    "--docker-image",
    default=None,
    help="Override Harbor docker image tag.",
)
@click.option(
    "--dry-run",
    is_flag=True,
    help="Print commands without executing.",
)
@click.argument("malvin_args", nargs=-1, type=click.UNPROCESSED)
@click.pass_context
def main(
    ctx: click.Context,
    task_dir: Path,
    workspace: Path | None,
    results_dir: Path | None,
    malvin_command: str,
    checks_override: str | None,
    grade_only: bool,
    skip_grade: bool,
    apply_solution: bool,
    reset_workspace_flag: bool,
    docker_image: str | None,
    dry_run: bool,
    malvin_args: tuple[str, ...],
) -> None:
    """Run malvin on a DeepSWE task and grade with Harbor ``tests/test.sh``."""
    extra = tuple(ctx.args)
    if extra:
        malvin_args = malvin_args + extra
    spec = parse_task_dir(task_dir)
    results_root = results_dir or default_deepswe_results_dir()
    run_root = results_root / spec.task_id / timestamp_dir()
    workspace = workspace or (results_root / spec.task_id / "workspace")
    logs_dir = run_root / "verifier_logs"
    click.echo(f"Task: {spec.task_id}")
    click.echo(f"Workspace: {workspace.resolve()}")
    click.echo(f"Run artifacts: {run_root.resolve()}")

    materialize_workspace(spec, workspace, dry_run=dry_run)
    if reset_workspace_flag or apply_solution:
        reset_workspace(spec, workspace, dry_run=dry_run)

    if apply_solution:
        if spec.solution_patch is None:
            raise click.ClickException(f"No solution patch at {spec.task_dir / 'solution'}")
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

    image = resolve_docker_image(spec, docker_image, dry_run=dry_run)
    grade_result: dict[str, Any]
    if skip_grade:
        grade_result = {"pass": None, "reward": None, "skipped": True}
    else:
        grade_result = grade_workspace(spec, workspace, logs_dir, image=image, dry_run=dry_run)

    malvin_log = find_latest_malvin_log()
    metadata = {
        "task_id": spec.task_id,
        "task_dir": str(spec.task_dir),
        "workspace": str(workspace.resolve()),
        "malvin_command": malvin_command,
        "malvin_args": list(malvin_args),
        "base_commit": spec.base_commit,
        "docker_image": image,
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


if __name__ == "__main__":
    main()
