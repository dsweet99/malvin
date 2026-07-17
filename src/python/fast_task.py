#!/usr/bin/env python3
"""Run malvin on a ``fast_tasks/<ID>`` workspace in local Docker; grade on the host.

The agent container mounts only a staged copy of ``workspace/`` at ``/app``.
``grade.py``, ``goldens/``, and other grader material stay on the host and are
never bind-mounted or baked into the agent image.

Usage::

    python ops/fast_task.py solve FT-01
    python ops/fast_task.py solve FT-01 --dry-run
    python ops/fast_task.py solve FT-01 --cursor
    python ops/fast_task.py tasks
    python ops/fast_task.py self-test

Results default to ``~/.malvin_home/fast_task_results``. Prefer a path under
``$HOME`` for ``--results-dir``: Snap Docker often cannot bind-mount host ``/tmp``.
"""

from __future__ import annotations

import json
import os
import shutil
import subprocess
import sys
import tempfile
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
MALVIN_BIN_REMOTE = "/root/.cargo/bin/malvin"
TOOLCHAIN_PATH = (
    "/root/.cargo/bin:/root/.local/bin:/usr/local/sbin:/usr/local/bin"
    ":/usr/sbin:/usr/bin:/sbin:/bin"
)
CURSOR_ENV_KEYS = ("CURSOR_AGENT_API_KEY", "CURSOR_API_KEY", "AGENT_API_KEY")
LEAK_NAME_MARKERS = ("grade.py", "goldens", "golden", "solution")
# Shell form required so stdin redirect works under `docker run … -w /app`.
CURSOR_AGENT_SHELL = "cursor-agent --force -p < plan.md"


def ft_default_results_dir() -> Path:
    """Return ``~/.malvin_home/fast_task_results`` (override with ``FAST_TASK_RESULTS``)."""
    override = os.environ.get("FAST_TASK_RESULTS")
    if override:
        return Path(override).expanduser().resolve()
    return (Path.home() / ".malvin_home" / "fast_task_results").resolve()


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
    which = shutil.which("malvin")
    if which:
        path = Path(which)
        if path.is_file():
            return path.resolve()
    cargo = Path.home() / ".cargo" / "bin" / "malvin"
    if cargo.is_file():
        return cargo.resolve()
    return None


def ft_dockerfile_for_agent(base_image: str = DEFAULT_BASE_IMAGE) -> str:
    """Dockerfile text for the reusable fast-task agent image (no grade material).

    Malvin is not baked into the image; ``ft_docker_agent_cmd`` bind-mounts the
    host binary at run time so evals always use the current build.
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
    """``docker run -e`` args for Cursor API keys present on the host."""
    args: list[str] = []
    for key in CURSOR_ENV_KEYS:
        value = os.environ.get(key)
        if value:
            args.extend(["-e", f"{key}={value}"])
    return args


def ft_redact_cmd_tokens(cmd: list[str]) -> list[str]:
    """Return *cmd* with Cursor API key values replaced by ``***``."""
    out: list[str] = []
    for token in cmd:
        redacted = token
        for key in CURSOR_ENV_KEYS:
            prefix = f"{key}="
            if redacted.startswith(prefix) and len(redacted) > len(prefix):
                redacted = prefix + "***"
                break
        out.append(redacted)
    return out


def ft_redact_cmd_for_display(cmd: list[str]) -> str:
    """Join *cmd* for logs, redacting Cursor API key values."""
    return " ".join(ft_redact_cmd_tokens(cmd))


def ft_docker_agent_cmd(
    *,
    image: str,
    workspace: Path,
    malvin_binary: Path | None = None,
    malvin_args: tuple[str, ...] = (),
    use_cursor: bool = False,
) -> list[str]:
    """Agent-phase ``docker run`` argv: workspace-only mount at ``/app``."""
    ws = workspace.resolve()
    volume_mounts: list[str] = ["-v", f"{ws}:/app"]
    if not use_cursor:
        if malvin_binary is None:
            raise click.ClickException(
                "malvin_binary is required when not using --cursor"
            )
        host_malvin = malvin_binary.resolve()
        if not host_malvin.is_file():
            raise click.ClickException(f"Host malvin binary not found: {host_malvin}")
        volume_mounts = [
            "-v",
            f"{host_malvin}:{MALVIN_BIN_REMOTE}:ro",
            *volume_mounts,
        ]
    cmd = [
        "docker",
        "run",
        "--rm",
        *ft_cursor_env_args(),
        *volume_mounts,
        "-e",
        f"PATH={TOOLCHAIN_PATH}",
        "-e",
        "MALVIN_FORCE_STDOUT_TEE=1",
        # Host-owned bind mount often looks "dubious" to container git-as-root.
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
    if use_cursor:
        # Skip malvin (and thus router init): shell stdin from plan.md.
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


def ft_relay_subprocess_stdout(cmd: list[str]) -> tuple[int, str]:
    """Run *cmd*, stream merged stdout/stderr live, return (exit_code, capture)."""
    env = os.environ.copy()
    env.setdefault("MALVIN_FORCE_STDOUT_TEE", "1")
    proc = subprocess.Popen(
        cmd,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
        bufsize=1,
        env=env,
    )
    chunks: list[str] = []
    assert proc.stdout is not None
    for line in proc.stdout:
        sys.stdout.write(line)
        sys.stdout.flush()
        chunks.append(line)
    proc.wait()
    return int(proc.returncode or 0), "".join(chunks)


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
    """Print DeepSWE-style evaluation block including ``reward:``."""
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
    _ = grade_result  # reward/pass are observational; not harness failure
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
    use_cursor: bool = False,
) -> dict[str, Any]:
    """Stage workspace, run agent in Docker, grade on host; return result dict."""
    task_dir = ft_resolve_task_dir(task_id)
    root = (results_dir or ft_default_results_dir()).resolve()
    run_root = root / task_id / ft_timestamp_dir()
    run_root.mkdir(parents=True, exist_ok=True)
    workspace = ft_stage_workspace(task_dir, run_root)
    image = ft_ensure_agent_image(
        image=docker_image or DEFAULT_IMAGE,
        base_image=base_image,
        dry_run=dry_run,
    )
    host_malvin: Path | None = None
    if not use_cursor:
        host_malvin = ft_resolve_malvin_binary()
        if host_malvin is None:
            raise click.ClickException(
                "No host malvin binary found (PATH or ~/.cargo/bin/malvin); "
                "build malvin on the host or use --cursor"
            )
    cmd = ft_docker_agent_cmd(
        image=image,
        workspace=workspace,
        malvin_binary=host_malvin,
        malvin_args=malvin_args,
        use_cursor=use_cursor,
    )
    click.echo(f"Staged workspace: {workspace}")
    click.echo(f"Agent command: {ft_redact_cmd_for_display(cmd)}")

    t0 = time.monotonic()
    if dry_run:
        click.echo("Dry run: skipping docker run")
        agent_result: dict[str, Any] = {
            "exit_code": 0,
            "agent_seconds": 0.0,
            "dry_run": True,
            "stdout": "",
        }
    else:
        if not ft_docker_available():
            raise click.ClickException("Docker daemon is not available")
        ft_preflight_workspace_mount(image=image, workspace=workspace)
        agent_label = "cursor-agent" if use_cursor else "malvin"
        click.echo(f"Running {agent_label} in local Docker (workspace-only mount)...")
        code, captured = ft_relay_subprocess_stdout(cmd)
        agent_result = {
            "exit_code": code,
            "agent_seconds": time.monotonic() - t0,
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
    use_cursor: bool = False,
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
        use_cursor=use_cursor,
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
    _ft_test_assert_agent_cmd_rejects_task_root()
    _ft_test_grade_on_host_starter_reward_zero()
    _ft_test_solve_help_and_dry_run()
    _ft_test_relay_streams_before_wait()
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
        assert len(mounts) == 2
        assert any(m.endswith(":/app") and str(ws.resolve()) in m for m in mounts)
        assert any(
            m == f"{host_malvin.resolve()}:{MALVIN_BIN_REMOTE}:ro" for m in mounts
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
            image=DEFAULT_IMAGE, workspace=ws, use_cursor=True, malvin_args=("--verbose",)
        )
        joined = " ".join(cmd)
        assert "sh" in cmd
        assert "-c" in cmd
        assert CURSOR_AGENT_SHELL in cmd
        assert "cursor-agent --force -p < plan.md" in joined
        assert "malvin" not in cmd
        assert "--verbose" not in cmd
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
    assert "--cursor" in help_result.output
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
        meta_paths = list(Path(tmp).glob("FT-01/*/metadata.json"))
        assert meta_paths, result.output
        meta = json.loads(meta_paths[0].read_text(encoding="utf-8"))
        joined_cmd = " ".join(meta["docker_cmd"])
        assert "malvin" in meta["docker_cmd"]
        assert "plan.md" in meta["docker_cmd"]
        assert "cursor-agent" not in joined_cmd
        assert "grade.py" not in joined_cmd
        assert "goldens" not in joined_cmd

        cursor_tmp = Path(tmp) / "cursor"
        cursor_tmp.mkdir()
        cursor_result = runner.invoke(
            cli,
            [
                "solve",
                "FT-01",
                "--cursor",
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
        code, captured = ft_relay_subprocess_stdout(cmd)
    finally:
        sys.stdout.write = _FT_RELAY_SPY_ORIG  # type: ignore[method-assign]
    assert code == 0
    assert "stream-line-1" in captured
    assert any("stream-line-1" in chunk for chunk in _FT_RELAY_SPY_SEEN)


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
    # Reward 0 must not fail the harness (user: don't require reward value).
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
        "PATH=/bin",
        "img",
    ]
    shown = ft_redact_cmd_for_display(cmd)
    assert "super-secret" not in shown
    assert "CURSOR_API_KEY=***" in shown
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

