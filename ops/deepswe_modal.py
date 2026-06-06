#!/usr/bin/env python3
"""Run DeepSWE Harbor verifier (and optionally malvin agent) on Modal.

No local Docker required for grading: builds a Modal Image from the task
Harbor Dockerfile (or pulls the registry image) and execs ``tests/test.sh`` in
a sandbox.

Agent sandboxes restrict outbound network to Cursor API endpoints. Grade-only
sandboxes block all network access. malvin and kiss are built from local source
trees (``MALVIN_REPO`` / ``KISS_REPO``), not crates.io.

Examples::

    # Grade reference solution (sanity check, expect reward=1):
    modal run ops/deepswe_modal.py --grade-only --apply-solution \\
        --task ../deep-swe/tasks/bandit-interprocedural-taint-checks

    # Agent on Modal + grade (full eval smoke):
    modal run ops/deepswe_modal.py \\
        --task ../deep-swe/tasks/bandit-interprocedural-taint-checks \\
        --command code -- --max-loops 1 --background

Local unit tests (no Modal credentials)::

    python ops/deepswe_modal.py --self-test
"""

from __future__ import annotations

import io
import json
import os
import socket
import sys
import tempfile
import threading
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, TextIO
from unittest.mock import MagicMock, patch

import click
from click.testing import CliRunner
import modal
from modal.stream_type import StreamType

sys.path.insert(0, str(Path(__file__).resolve().parent))
from deepswe_run import (
    DEFAULT_CHECKS_CODE,
    DEFAULT_CHECKS_DO,
    apply_patch,
    materialize_workspace,
    parse_task_dir,
    reset_workspace,
    timestamp_dir,
    write_plan_and_checks,
)

APP_NAME = "deepswe-modal"
LOGS_REMOTE = "/logs"
APP_REMOTE = "/app"
TESTS_REMOTE = "/tests"
MALVIN_TOOLCHAIN_REMOTE = "/opt/toolchain/malvin"
KISS_TOOLCHAIN_REMOTE = "/opt/toolchain/kiss"
TOOLCHAIN_PATH = (
    "/root/.cargo/bin:/root/.local/bin:/usr/local/sbin:/usr/local/bin"
    ":/usr/sbin:/usr/bin:/sbin:/bin"
)

CURSOR_API_HOSTS = (
    "api2.cursor.sh",
    "api2geo.cursor.sh",
    "api2direct.cursor.sh",
    "api.cursor.com",
    "api3.cursor.sh",
    "repo42.cursor.sh",
)

app = modal.App(APP_NAME)


def malvin_repo_root() -> Path:
    """Return the malvin repository root (parent of ``ops/``)."""
    return Path(__file__).resolve().parent.parent


def kiss_repo_root() -> Path:
    """Return the kiss-ai source tree (``KISS_REPO`` or sibling ``kiss`` repo)."""
    override = os.environ.get("KISS_REPO")
    if override:
        return Path(override).resolve()
    return malvin_repo_root().parent / "kiss"


def validate_toolchain_repos() -> tuple[Path, Path]:
    """Ensure local malvin and kiss trees exist before building the agent image."""
    malvin_repo = malvin_repo_root()
    kiss_repo = kiss_repo_root()
    if not (malvin_repo / "Cargo.toml").is_file():
        raise click.ClickException(f"malvin repo not found: {malvin_repo}")
    if not (kiss_repo / "Cargo.toml").is_file():
        raise click.ClickException(
            f"kiss repo not found: {kiss_repo} (set KISS_REPO to override)"
        )
    return malvin_repo, kiss_repo


def malvin_upload_ignore() -> list[str]:
    """Exclude heavy or ephemeral paths when uploading malvin source to Modal."""
    return [
        "target/",
        "experiments/",
        ".malvin/logs",
        ".git",
        ".kissignore",
        "__pycache__/",
        "results/",
    ]


def kiss_upload_ignore() -> list[str]:
    """Exclude build artifacts when uploading kiss source to Modal."""
    return ["target/", ".git", "__pycache__/"]


def resolve_cursor_api_cidrs() -> list[str]:
    """Resolve Cursor API hostnames to CIDR strings for sandbox egress allowlists."""
    cidrs: set[str] = set()
    for host in CURSOR_API_HOSTS:
        try:
            infos = socket.getaddrinfo(host, 443, type=socket.SOCK_STREAM)
        except socket.gaierror:
            continue
        for info in infos:
            ip = info[4][0]
            if ":" in ip:
                cidrs.add(f"{ip}/128")
            else:
                cidrs.add(f"{ip}/32")
    if not cidrs:
        raise click.ClickException(
            "Could not resolve Cursor API hosts for sandbox network allowlist"
        )
    return sorted(cidrs)


def sandbox_network_kwargs(*, cursor_api_only: bool, block_all: bool) -> dict[str, Any]:
    """Return Modal ``Sandbox.create`` kwargs for the requested network posture."""
    if block_all:
        return {"block_network": True}
    if cursor_api_only:
        return {"cidr_allowlist": resolve_cursor_api_cidrs()}
    return {}


def relay_stream(reader: Any, sink: TextIO) -> None:
    for chunk in reader:
        sink.write(chunk)
        sink.flush()


def stream_process_output(proc: Any, out: TextIO, err: TextIO) -> None:
    threads = [
        threading.Thread(target=relay_stream, args=(proc.stdout, out), daemon=True),
        threading.Thread(target=relay_stream, args=(proc.stderr, err), daemon=True),
    ]
    for thread in threads:
        thread.start()
    for thread in threads:
        thread.join()


def harbor_image(spec: Any, *, dockerfile: Path) -> modal.Image:
    """Modal image with Harbor task dependencies; workspace/tests mounted at runtime."""
    if dockerfile.is_file():
        click.echo(f"Building Modal image from {dockerfile} (may take several minutes)...")
        return modal.Image.from_dockerfile(str(dockerfile))
    click.echo(f"Pulling Harbor image {spec.docker_image}...")
    return modal.Image.from_registry(spec.docker_image)


def mount_task_tree(image: modal.Image, workspace: Path, tests_dir: Path) -> modal.Image:
    return (
        image.add_local_dir(str(workspace.resolve()), remote_path=APP_REMOTE)
        .add_local_dir(str(tests_dir.resolve()), remote_path=TESTS_REMOTE)
    )


def mount_local_toolchain(
    image: modal.Image,
    *,
    malvin_repo: Path,
    kiss_repo: Path,
) -> modal.Image:
    """Layer local malvin/kiss source and build binaries inside the Modal image."""
    return (
        image.add_local_dir(
            str(malvin_repo.resolve()),
            remote_path=MALVIN_TOOLCHAIN_REMOTE,
            ignore=malvin_upload_ignore(),
        )
        .add_local_dir(
            str(kiss_repo.resolve()),
            remote_path=KISS_TOOLCHAIN_REMOTE,
            ignore=kiss_upload_ignore(),
        )
        .run_commands(
            f"bash -lc 'cargo install --path {KISS_TOOLCHAIN_REMOTE} --locked'",
            f"bash -lc 'cargo install --path {MALVIN_TOOLCHAIN_REMOTE} --locked'",
            "curl -fsSL https://cursor.com/install | bash",
            "/root/.local/bin/agent --version || true",
        )
        .env({"PATH": TOOLCHAIN_PATH})
    )


def read_remote_file(sandbox: modal.Sandbox, path: str) -> str | None:
    try:
        with sandbox.open(path, "r") as handle:
            return handle.read()
    except OSError:
        return None


def grade_in_sandbox(
    image: modal.Image,
    *,
    timeout: int = 3600,
) -> dict[str, Any]:
    sandbox: modal.Sandbox | None = None
    try:
        sandbox = modal.Sandbox.create(
            app=app,
            image=image,
            workdir=APP_REMOTE,
            timeout=timeout,
            **sandbox_network_kwargs(cursor_api_only=False, block_all=True),
        )
        proc = sandbox.exec(
            "bash",
            "-lc",
            f"mkdir -p {LOGS_REMOTE}/verifier {LOGS_REMOTE}/artifacts && bash {TESTS_REMOTE}/test.sh",
            stdout=StreamType.PIPE,
            stderr=StreamType.PIPE,
            workdir=APP_REMOTE,
            text=True,
            bufsize=1,
        )
        stream_process_output(proc, sys.stdout, sys.stderr)
        proc.wait()
        reward_text = read_remote_file(sandbox, f"{LOGS_REMOTE}/verifier/reward.txt")
        reward: int | None = None
        if reward_text is not None:
            stripped = reward_text.strip()
            if stripped in {"0", "1"}:
                reward = int(stripped)
        model_patch = read_remote_file(sandbox, f"{LOGS_REMOTE}/artifacts/model.patch")
        return {
            "pass": reward == 1,
            "reward": reward,
            "verifier_exit_code": int(proc.returncode or 0),
            "model_patch_chars": len(model_patch) if model_patch else 0,
        }
    finally:
        if sandbox is not None:
            sandbox.terminate()


def harbor_agent_image(
    spec: Any,
    workspace: Path,
    tests_dir: Path,
    *,
    dockerfile: Path,
    malvin_repo: Path,
    kiss_repo: Path,
) -> modal.Image:
    """Harbor task image plus locally built malvin/kiss and cursor-agent."""
    base = harbor_image(spec, dockerfile=dockerfile)
    augmented = base.run_commands(
        "apt-get update -qq && apt-get install -y -qq curl build-essential pkg-config libssl-dev",
        "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y",
    )
    augmented = mount_local_toolchain(
        augmented,
        malvin_repo=malvin_repo,
        kiss_repo=kiss_repo,
    )
    return mount_task_tree(augmented, workspace, tests_dir)


def run_agent_and_grade_in_sandbox(
    image: modal.Image,
    *,
    command: str,
    malvin_argv: list[str],
    cursor_secrets: list[modal.Secret],
    timeout: int = 7200,
) -> tuple[dict[str, Any], dict[str, Any]]:
    """Run malvin then Harbor verifier in one sandbox so edits persist for grading."""
    sandbox: modal.Sandbox | None = None
    try:
        sandbox = modal.Sandbox.create(
            app=app,
            image=image,
            workdir=APP_REMOTE,
            secrets=cursor_secrets,
            timeout=timeout,
            **sandbox_network_kwargs(cursor_api_only=True, block_all=False),
        )
        agent_proc = sandbox.exec(
            "malvin",
            command,
            "@plan.md",
            *malvin_argv,
            stdout=StreamType.PIPE,
            stderr=StreamType.PIPE,
            workdir=APP_REMOTE,
            text=True,
            bufsize=1,
        )
        click.echo("Running malvin agent on Modal...")
        stream_process_output(agent_proc, sys.stdout, sys.stderr)
        agent_proc.wait()
        agent_result = {"exit_code": int(agent_proc.returncode or 0)}

        grade_proc = sandbox.exec(
            "bash",
            "-lc",
            f"mkdir -p {LOGS_REMOTE}/verifier {LOGS_REMOTE}/artifacts && bash {TESTS_REMOTE}/test.sh",
            stdout=StreamType.PIPE,
            stderr=StreamType.PIPE,
            workdir=APP_REMOTE,
            text=True,
            bufsize=1,
        )
        click.echo("Running Harbor verifier on Modal...")
        stream_process_output(grade_proc, sys.stdout, sys.stderr)
        grade_proc.wait()
        reward_text = read_remote_file(sandbox, f"{LOGS_REMOTE}/verifier/reward.txt")
        reward: int | None = None
        if reward_text is not None:
            stripped = reward_text.strip()
            if stripped in {"0", "1"}:
                reward = int(stripped)
        model_patch = read_remote_file(sandbox, f"{LOGS_REMOTE}/artifacts/model.patch")
        grade_result = {
            "pass": reward == 1,
            "reward": reward,
            "verifier_exit_code": int(grade_proc.returncode or 0),
            "model_patch_chars": len(model_patch) if model_patch else 0,
        }
        return agent_result, grade_result
    finally:
        if sandbox is not None:
            sandbox.terminate()


def run_malvin_in_sandbox(
    image: modal.Image,
    *,
    command: str,
    malvin_argv: list[str],
    cursor_secrets: list[modal.Secret],
    timeout: int = 7200,
) -> dict[str, Any]:
    sandbox: modal.Sandbox | None = None
    try:
        sandbox = modal.Sandbox.create(
            app=app,
            image=image,
            workdir=APP_REMOTE,
            secrets=cursor_secrets,
            timeout=timeout,
            **sandbox_network_kwargs(cursor_api_only=True, block_all=False),
        )
        proc = sandbox.exec(
            "malvin",
            command,
            "@plan.md",
            *malvin_argv,
            stdout=StreamType.PIPE,
            stderr=StreamType.PIPE,
            workdir=APP_REMOTE,
            text=True,
            bufsize=1,
        )
        stream_process_output(proc, sys.stdout, sys.stderr)
        proc.wait()
        return {"exit_code": int(proc.returncode or 0)}
    finally:
        if sandbox is not None:
            sandbox.terminate()


def cursor_secrets() -> list[modal.Secret]:
    keys = ["CURSOR_AGENT_API_KEY", "CURSOR_API_KEY", "AGENT_API_KEY"]
    present = [k for k in keys if os.environ.get(k)]
    if not present:
        return []
    return [modal.Secret.from_local_environ(present)]


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
    "--self-test",
    is_flag=True,
    help="Run local unit tests without Modal credentials.",
)
@click.option(
    "--task",
    "task_dir",
    type=click.Path(exists=True, file_okay=False, path_type=Path),
    default=None,
)
@click.option(
    "--workspace",
    type=click.Path(file_okay=False, path_type=Path),
    default=None,
)
@click.option(
    "--results-dir",
    type=click.Path(file_okay=False, path_type=Path),
    default=Path("results/deepswe"),
    show_default=True,
)
@click.option(
    "--command",
    "malvin_command",
    type=click.Choice(["code", "do"]),
    default="code",
    show_default=True,
)
@click.option(
    "--grade-only",
    is_flag=True,
    help="Skip agent; grade current workspace on Modal.",
)
@click.option(
    "--apply-solution",
    is_flag=True,
    help="Apply reference solution.patch before agent or grade.",
)
@click.option(
    "--reset",
    "reset_flag",
    is_flag=True,
    help="Hard reset workspace to base_commit before run.",
)
@click.argument("malvin_args", nargs=-1, type=click.UNPROCESSED)
@click.pass_context
def main(
    ctx: click.Context,
    self_test: bool,
    task_dir: Path | None,
    workspace: Path | None,
    results_dir: Path,
    malvin_command: str,
    grade_only: bool,
    apply_solution: bool,
    reset_flag: bool,
    malvin_args: tuple[str, ...],
) -> None:
    """DeepSWE evaluation on Modal (agent optional, Harbor grade in sandbox)."""
    if self_test:
        run_unit_tests()
        raise SystemExit(0)
    if task_dir is None:
        raise click.ClickException("--task is required unless --self-test is set")
    extra = tuple(ctx.args)
    if extra:
        malvin_args = malvin_args + extra
    spec = parse_task_dir(task_dir)
    workspace = workspace or (results_dir / spec.task_id / "workspace")
    run_root = results_dir / spec.task_id / f"modal_{timestamp_dir()}"

    click.echo(f"Task: {spec.task_id}")
    click.echo(f"Workspace: {workspace.resolve()}")
    click.echo(f"Artifacts: {run_root.resolve()}")

    materialize_workspace(spec, workspace, dry_run=False)
    if reset_flag or apply_solution:
        reset_workspace(spec, workspace, dry_run=False)
    if apply_solution:
        if spec.solution_patch is None:
            raise click.ClickException(f"No solution at {spec.task_dir / 'solution'}")
        apply_patch(workspace, spec.solution_patch, dry_run=False)

    agent_result: dict[str, Any] | None = None
    grade_result: dict[str, Any]
    if not grade_only:
        malvin_repo, kiss_repo = validate_toolchain_repos()
        click.echo(f"malvin source: {malvin_repo.resolve()}")
        click.echo(f"kiss source: {kiss_repo.resolve()}")
        write_plan_and_checks(
            spec,
            workspace,
            command=malvin_command,
            checks_override=DEFAULT_CHECKS_CODE if malvin_command == "code" else DEFAULT_CHECKS_DO,
            dry_run=False,
        )
        combined = harbor_agent_image(
            spec,
            workspace,
            spec.tests_dir,
            dockerfile=spec.dockerfile,
            malvin_repo=malvin_repo,
            kiss_repo=kiss_repo,
        )
        agent_result, grade_result = run_agent_and_grade_in_sandbox(
            combined,
            command=malvin_command,
            malvin_argv=list(malvin_args),
            cursor_secrets=cursor_secrets(),
        )
    else:
        grade_img = mount_task_tree(
            harbor_image(spec, dockerfile=spec.dockerfile),
            workspace,
            spec.tests_dir,
        )
        click.echo("Running Harbor verifier on Modal...")
        grade_result = grade_in_sandbox(grade_img)

    metadata = {
        "task_id": spec.task_id,
        "runtime": "modal",
        "workspace": str(workspace.resolve()),
        "malvin_command": malvin_command if not grade_only else None,
        "malvin_args": list(malvin_args),
        "agent": agent_result,
        "grade": grade_result,
        "timestamp_utc": datetime.now(timezone.utc).isoformat(),
    }
    write_metadata(run_root, metadata)
    if grade_result.get("reward") is not None:
        (run_root / "reward.txt").write_text(f"{grade_result['reward']}\n", encoding="utf-8")

    click.echo("\n=== Modal evaluation ===")
    click.echo(f"reward: {grade_result.get('reward')}")
    click.echo(f"pass: {grade_result.get('pass')}")
    if agent_result:
        click.echo(f"malvin exit: {agent_result.get('exit_code')}")
    click.echo(f"artifacts: {run_root.resolve()}")

    if grade_result.get("pass") is False:
        raise SystemExit(1)
    if agent_result and agent_result.get("exit_code") not in (0, None):
        raise SystemExit(agent_result["exit_code"])


@app.local_entrypoint()
def entrypoint(*arglist: str) -> None:
    """``modal run ops/deepswe_modal.py -- [OPTIONS] [-- MALVIN_ARGS]``."""
    main.main(args=list(arglist), prog_name="modal run ops/deepswe_modal.py", standalone_mode=True)


def _test_repo_roots() -> None:
    malvin_repo = malvin_repo_root()
    assert malvin_repo.name == "malvin"
    assert (malvin_repo / "ops" / "deepswe_modal.py").is_file()
    kiss_repo = kiss_repo_root()
    assert kiss_repo.name == "kiss"


def _test_cursor_cidrs() -> None:
    cidrs = resolve_cursor_api_cidrs()
    assert cidrs
    for cidr in cidrs:
        assert "/" in cidr


def _test_network_kwargs() -> None:
    blocked = sandbox_network_kwargs(cursor_api_only=False, block_all=True)
    assert blocked == {"block_network": True}
    with patch(f"{__name__}.resolve_cursor_api_cidrs", return_value=["1.2.3.4/32"]):
        allowed = sandbox_network_kwargs(cursor_api_only=True, block_all=False)
    assert allowed == {"cidr_allowlist": ["1.2.3.4/32"]}
    assert sandbox_network_kwargs(cursor_api_only=False, block_all=False) == {}


def _test_stream_helpers() -> None:
    sink = io.StringIO()
    relay_stream(iter(["alpha", "beta"]), sink)
    assert sink.getvalue() == "alphabeta"
    out = io.StringIO()
    err = io.StringIO()
    proc = MagicMock(stdout=iter(["out"]), stderr=iter(["err"]))
    stream_process_output(proc, out, err)
    assert out.getvalue() == "out"
    assert err.getvalue() == "err"


def _test_cursor_secrets() -> None:
    keys = ["CURSOR_AGENT_API_KEY", "CURSOR_API_KEY", "AGENT_API_KEY"]
    saved = {key: os.environ.pop(key, None) for key in keys}
    try:
        assert cursor_secrets() == []
        os.environ["CURSOR_API_KEY"] = "test-key"
        assert len(cursor_secrets()) == 1
    finally:
        for key, value in saved.items():
            if value is None:
                os.environ.pop(key, None)
            else:
                os.environ[key] = value


def _test_grade_in_sandbox_network() -> None:
    fake_proc = MagicMock(stdout=iter([]), stderr=iter([]), returncode=0)
    fake_proc.wait.return_value = None
    fake_sandbox = MagicMock()
    fake_sandbox.exec.return_value = fake_proc
    fake_sandbox.open.return_value.__enter__.return_value.read.return_value = "1\n"
    image = MagicMock()
    with patch.object(modal.Sandbox, "create", return_value=fake_sandbox) as mock_create:
        result = grade_in_sandbox(image)
    mock_create.assert_called_once()
    assert mock_create.call_args.kwargs["block_network"] is True
    assert result["reward"] == 1
    fake_sandbox.terminate.assert_called_once()


def _test_agent_sandbox_network() -> None:
    fake_proc = MagicMock(stdout=iter([]), stderr=iter([]), returncode=0)
    fake_proc.wait.return_value = None
    fake_sandbox = MagicMock()
    fake_sandbox.exec.return_value = fake_proc
    fake_sandbox.open.return_value.__enter__.return_value.read.return_value = "0\n"
    image = MagicMock()
    with patch.object(modal.Sandbox, "create", return_value=fake_sandbox) as mock_create:
        with patch(f"{__name__}.resolve_cursor_api_cidrs", return_value=["9.9.9.9/32"]):
            agent_result, grade_result = run_agent_and_grade_in_sandbox(
                image,
                command="code",
                malvin_argv=[],
                cursor_secrets=[],
            )
    assert mock_create.call_args.kwargs["cidr_allowlist"] == ["9.9.9.9/32"]
    assert "block_network" not in mock_create.call_args.kwargs
    assert agent_result["exit_code"] == 0
    assert grade_result["reward"] == 0
    fake_sandbox.terminate.assert_called_once()


def _test_run_malvin_sandbox_network() -> None:
    fake_proc = MagicMock(stdout=iter([]), stderr=iter([]), returncode=0)
    fake_proc.wait.return_value = None
    fake_sandbox = MagicMock()
    fake_sandbox.exec.return_value = fake_proc
    image = MagicMock()
    with patch.object(modal.Sandbox, "create", return_value=fake_sandbox) as mock_create:
        with patch(f"{__name__}.resolve_cursor_api_cidrs", return_value=["8.8.8.8/32"]):
            result = run_malvin_in_sandbox(
                image,
                command="do",
                malvin_argv=["--max-loops", "1"],
                cursor_secrets=[],
            )
    assert mock_create.call_args.kwargs["cidr_allowlist"] == ["8.8.8.8/32"]
    assert "block_network" not in mock_create.call_args.kwargs
    assert result["exit_code"] == 0
    fake_sandbox.terminate.assert_called_once()


def _test_validate_toolchain_repos() -> None:
    validate_toolchain_repos()
    with tempfile.TemporaryDirectory() as tmp:
        missing = Path(tmp) / "empty"
        missing.mkdir()
        with patch.dict(os.environ, {"KISS_REPO": str(missing)}):
            try:
                validate_toolchain_repos()
            except click.ClickException as exc:
                assert "kiss repo not found" in str(exc)
            else:
                raise AssertionError("expected ClickException for missing kiss repo")


def _test_read_remote_file() -> None:
    sandbox = MagicMock()
    sandbox.open.return_value.__enter__.return_value.read.return_value = "payload"
    assert read_remote_file(sandbox, "/tmp/x") == "payload"
    sandbox.open.side_effect = OSError("missing")
    assert read_remote_file(sandbox, "/tmp/missing") is None


def _test_write_metadata() -> None:
    with tempfile.TemporaryDirectory() as tmp:
        out_dir = Path(tmp) / "run"
        payload = {"task_id": "demo", "reward": 1}
        write_metadata(out_dir, payload)
        loaded = json.loads((out_dir / "metadata.json").read_text(encoding="utf-8"))
        assert loaded == payload


def _test_resolve_cursor_api_cidrs_mocked() -> None:
    def fake_getaddrinfo(host: str, port: int, *args: Any, **kwargs: Any) -> list[tuple]:
        if host == CURSOR_API_HOSTS[0]:
            return [(None, None, None, None, ("1.2.3.4", port))]
        if host == CURSOR_API_HOSTS[1]:
            return [
                (None, None, None, None, ("1.2.3.4", port)),
                (None, None, None, None, ("2001:db8::1", port)),
            ]
        raise socket.gaierror("no such host")

    with patch(f"{__name__}.socket.getaddrinfo", side_effect=fake_getaddrinfo):
        cidrs = resolve_cursor_api_cidrs()
    assert cidrs == ["1.2.3.4/32", "2001:db8::1/128"]

    def all_fail(*args: Any, **kwargs: Any) -> list[tuple]:
        raise socket.gaierror("offline")

    with patch(f"{__name__}.socket.getaddrinfo", side_effect=all_fail):
        try:
            resolve_cursor_api_cidrs()
        except click.ClickException as exc:
            assert "Could not resolve Cursor API hosts" in str(exc)
        else:
            raise AssertionError("expected ClickException when DNS fails")


def _test_upload_ignore_patterns() -> None:
    malvin_ignores = malvin_upload_ignore()
    kiss_ignores = kiss_upload_ignore()
    assert "target/" in malvin_ignores
    assert "target/" in kiss_ignores
    assert "Cargo.toml" not in malvin_ignores
    assert "src/" not in malvin_ignores


class _RecordingImage:
    def __init__(self) -> None:
        self.calls: list[tuple[str, tuple[Any, ...], dict[str, Any]]] = []

    def add_local_dir(self, *args: Any, **kwargs: Any) -> _RecordingImage:
        self.calls.append(("add_local_dir", args, kwargs))
        return self

    def run_commands(self, *args: Any, **kwargs: Any) -> _RecordingImage:
        self.calls.append(("run_commands", args, kwargs))
        return self

    def env(self, *args: Any, **kwargs: Any) -> _RecordingImage:
        self.calls.append(("env", args, kwargs))
        return self


def _test_mount_local_toolchain_recipe() -> None:
    malvin_repo, kiss_repo = validate_toolchain_repos()
    recorder = _RecordingImage()
    mount_local_toolchain(recorder, malvin_repo=malvin_repo, kiss_repo=kiss_repo)
    uploads = [call for call in recorder.calls if call[0] == "add_local_dir"]
    assert len(uploads) == 2
    assert uploads[0][2]["remote_path"] == MALVIN_TOOLCHAIN_REMOTE
    assert uploads[0][2]["ignore"] == malvin_upload_ignore()
    assert uploads[1][2]["remote_path"] == KISS_TOOLCHAIN_REMOTE
    commands = next(call[1] for call in recorder.calls if call[0] == "run_commands")
    assert len(commands) == 4
    assert KISS_TOOLCHAIN_REMOTE in commands[0]
    assert MALVIN_TOOLCHAIN_REMOTE in commands[1]
    assert "cursor.com/install" in commands[2]
    env_calls = [call for call in recorder.calls if call[0] == "env"]
    assert env_calls[-1][1][0] == {"PATH": TOOLCHAIN_PATH}


def _test_self_test_flag() -> None:
    runner = CliRunner()
    with patch(f"{__name__}.run_unit_tests") as mock_tests:
        result = runner.invoke(main, ["--self-test"])
    assert result.exit_code == 0, result.output
    mock_tests.assert_called_once()


def run_unit_tests() -> None:
    """Local tests for deepswe_modal helpers (no Modal network)."""
    _test_repo_roots()
    _test_cursor_cidrs()
    _test_network_kwargs()
    _test_stream_helpers()
    _test_cursor_secrets()
    _test_validate_toolchain_repos()
    _test_read_remote_file()
    _test_write_metadata()
    _test_resolve_cursor_api_cidrs_mocked()
    _test_upload_ignore_patterns()
    _test_mount_local_toolchain_recipe()
    _test_grade_in_sandbox_network()
    _test_agent_sandbox_network()
    _test_run_malvin_sandbox_network()
    _test_self_test_flag()


if __name__ == "__main__":
    main()
