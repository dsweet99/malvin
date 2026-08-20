#!/usr/bin/env python

"""Run malvin on Modal, forwarding host CLI arguments to the remote malvin process.

Modal sandboxes do not inherit interactive ``agent login`` sessions from the host.
Export ``CURSOR_AGENT_API_KEY``, ``CURSOR_API_KEY``, or ``AGENT_API_KEY`` in the
shell that launches this command (or set ``MODAL_CURSOR_SECRET_NAME`` for a Modal-stored
secret).

Runtime dependency: install Modal with ``pip install modal`` or ``uv pip install modal``.

Gating smoke test (requires Modal credentials and network)::

    modal run ops/malvin_modal.py -- --version

Local unit tests (no Modal credentials)::

    python ops/malvin_modal.py --self-test
"""

from __future__ import annotations

import io
import os
import subprocess
import sys
import threading
from pathlib import Path
from types import SimpleNamespace
from typing import Any, TextIO
from unittest.mock import MagicMock, patch

import click
from click.testing import CliRunner
import modal
from modal.stream_type import StreamType

sys.path.insert(0, str(Path(__file__).resolve().parent))
from modal_sandbox_app import lookup_sandbox_app, test_sandbox_app_lookup
from modal_sandbox_lifecycle import release_modal_sandbox

APP_NAME = "malvin-modal"
WORKSPACE = "/workspace"
CURSOR_ENV_KEYS = ["CURSOR_AGENT_API_KEY", "CURSOR_API_KEY", "AGENT_API_KEY"]
MODAL_CURSOR_SECRET_NAME_ENV = "MODAL_CURSOR_SECRET_NAME"
_CURSOR_CREDENTIALS_ERROR = (
    "Cursor API key required for Modal agent runs. "
    "Export CURSOR_AGENT_API_KEY, CURSOR_API_KEY, or AGENT_API_KEY "
    "in the shell that launches this command. "
    "Interactive `agent login` is not available inside Modal sandboxes."
)

SANDBOX_CPU = 2.0
SANDBOX_MEMORY_MIB = 4096

app = modal.App(APP_NAME)

_PATH = (
    "/root/.cargo/bin:/root/.local/bin:/usr/local/sbin:/usr/local/bin"
    ":/usr/sbin:/usr/bin:/sbin:/bin"
)

_BASE_IMAGE = (
    modal.Image.debian_slim(python_version="3.11")
    .apt_install(
        "curl",
        "bash",
        "ca-certificates",
        "build-essential",
        "pkg-config",
        "libssl-dev",
    )
    .run_commands(
        "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y",
        "bash -lc 'cargo install malvin --locked'",
        "curl -fsSL https://cursor.com/install | bash",
        "/root/.local/bin/agent --version || true",
    )
    .env({"PATH": _PATH})
)

def build_ignore_patterns() -> list[str]:
    """Patterns for ``add_local_dir`` upload excludes."""
    return [
        "target/",
        "experiments/",
        ".malvin/logs",
        ".git",
        ".kissignore",
        "__pycache__/",
    ]

def parse_malvin_argv(argv: list[str]) -> list[str]:
    """Return malvin args; text after the first ``--`` when present."""
    if "--" in argv:
        return list(argv[argv.index("--") + 1 :])
    return list(argv)

def relay_stream(reader: Any, sink: TextIO) -> None:
    """Copy text chunks from *reader* to *sink* in order."""
    for chunk in reader:
        sink.write(chunk)
        sink.flush()

def workspace_image() -> modal.Image:
    """Image with the caller cwd mounted at ``/workspace``."""
    return _BASE_IMAGE.add_local_dir(
        os.getcwd(),
        remote_path=WORKSPACE,
        ignore=build_ignore_patterns(),
    )

def present_cursor_keys() -> list[str]:
    """Return Cursor env var names that are set locally."""
    return [key for key in CURSOR_ENV_KEYS if os.environ.get(key)]

def cursor_credentials_available() -> bool:
    """Return True when local env keys or a named Modal secret can supply credentials."""
    if present_cursor_keys():
        return True
    return bool(os.environ.get(MODAL_CURSOR_SECRET_NAME_ENV))

def require_cursor_credentials_for_agent() -> None:
    """Fail fast on the host when Modal agent runs lack Cursor credentials."""
    if not cursor_credentials_available():
        raise click.ClickException(_CURSOR_CREDENTIALS_ERROR)

def cursor_secrets() -> list[modal.Secret]:
    """Inject Cursor API keys present in the local environment."""
    present = present_cursor_keys()
    if present:
        return [modal.Secret.from_local_environ(present)]
    secret_name = os.environ.get(MODAL_CURSOR_SECRET_NAME_ENV)
    if secret_name:
        return [modal.Secret.from_name(secret_name)]
    return []

def finish_process(proc: Any) -> int:
    """Wait for *proc* and return its exit code."""
    proc.wait()
    return int(proc.returncode or 0)

def stream_process_output(proc: Any, out: TextIO, err: TextIO) -> None:
    """Relay sandbox stdout/stderr to local streams concurrently."""
    threads = [
        threading.Thread(target=relay_stream, args=(proc.stdout, out), daemon=True),
        threading.Thread(target=relay_stream, args=(proc.stderr, err), daemon=True),
    ]
    for thread in threads:
        thread.start()
    for thread in threads:
        thread.join()

def run_local_malvin_usage() -> str:
    """Return bare ``malvin`` usage text from a local subprocess when available."""
    try:
        result = subprocess.run(
            ["malvin"],
            capture_output=True,
            text=True,
            check=False,
        )
    except FileNotFoundError:
        return (
            "malvin not found on PATH.\n"
            "Install malvin locally or pass malvin arguments after `--`.\n"
        )
    if result.stdout.strip():
        return result.stdout
    if result.stderr.strip():
        return result.stderr
    return "malvin produced no usage output.\n"

def render_empty_argv_help(ctx: click.Context) -> str:
    """Compose malvin usage followed by wrapper usage for empty forwarded argv."""
    malvin_text = run_local_malvin_usage().rstrip()
    wrapper_text = ctx.get_help().rstrip()
    return f"{malvin_text}\n\n{wrapper_text}\n"

def print_empty_argv_help(ctx: click.Context) -> None:
    """Print composite help for empty forwarded argv."""
    sys.stdout.write(render_empty_argv_help(ctx))
    sys.stdout.flush()

def sandbox_app() -> modal.App:
    """Return an initialized Modal app for sandbox creation."""
    return lookup_sandbox_app(app, APP_NAME)

def run_malvin_remote(malvin_argv: list[str]) -> int:
    """Create sandbox, exec malvin, stream I/O, terminate sandbox."""
    require_cursor_credentials_for_agent()
    image = workspace_image()
    secrets = cursor_secrets()
    sandbox: modal.Sandbox | None = None
    try:
        sandbox = modal.Sandbox.create(
            app=sandbox_app(),
            image=image,
            workdir=WORKSPACE,
            secrets=secrets,
            timeout=3600,
            cpu=SANDBOX_CPU,
            memory=SANDBOX_MEMORY_MIB,
        )
        proc = sandbox.exec(
            "malvin",
            *malvin_argv,
            stdout=StreamType.PIPE,
            stderr=StreamType.PIPE,
            workdir=WORKSPACE,
            text=True,
            bufsize=1,
        )
        stream_process_output(proc, sys.stdout, sys.stderr)
        return finish_process(proc)
    finally:
        release_modal_sandbox(sandbox)

def dispatch_cli(ctx: click.Context, self_test: bool) -> None:
    """Click command body for the Modal malvin wrapper."""
    if self_test:
        run_unit_tests()
        raise SystemExit(0)
    if not ctx.args:
        print_empty_argv_help(ctx)
        raise SystemExit(0)
    code = run_malvin_remote(list(ctx.args))
    raise SystemExit(code)

def _test_static_helpers() -> None:
    assert parse_malvin_argv(["--", "--version"]) == ["--version"]
    assert parse_malvin_argv(["--help"]) == ["--help"]
    ignore = build_ignore_patterns()
    for needle in ("target/", "experiments/", ".malvin/logs", ".git", "__pycache__/"):
        assert needle in ignore, f"UT-IGNORE: missing {needle}"
    sink = io.StringIO()
    relay_stream(iter(["alpha", "beta"]), sink)
    assert sink.getvalue() == "alphabeta"
    stub = SimpleNamespace(returncode=42, wait=lambda: None)
    assert finish_process(stub) == 42

def _test_cursor_and_stream() -> None:
    saved = {key: os.environ.pop(key, None) for key in CURSOR_ENV_KEYS}
    saved_secret = os.environ.pop(MODAL_CURSOR_SECRET_NAME_ENV, None)
    try:
        assert present_cursor_keys() == []
        assert cursor_secrets() == []
        assert not cursor_credentials_available()
        os.environ["CURSOR_API_KEY"] = "test-key"
        assert present_cursor_keys() == ["CURSOR_API_KEY"]
        assert cursor_credentials_available()
        assert len(cursor_secrets()) == 1
        os.environ.pop("CURSOR_API_KEY", None)
        os.environ[MODAL_CURSOR_SECRET_NAME_ENV] = "modal-cursor-secret"
        assert cursor_credentials_available()
        with patch(f"{__name__}.modal.Secret.from_name", return_value=MagicMock()) as mock_name:
            assert len(cursor_secrets()) == 1
            mock_name.assert_called_once_with("modal-cursor-secret")
    finally:
        for key, value in saved.items():
            os.environ.pop(key, None) if value is None else os.environ.__setitem__(key, value)
        if saved_secret is None:
            os.environ.pop(MODAL_CURSOR_SECRET_NAME_ENV, None)
        else:
            os.environ[MODAL_CURSOR_SECRET_NAME_ENV] = saved_secret
    out = io.StringIO()
    err = io.StringIO()
    proc = SimpleNamespace(stdout=iter(["out"]), stderr=iter(["err"]))
    stream_process_output(proc, out, err)
    assert out.getvalue() == "out"
    assert err.getvalue() == "err"

def _test_modal_remote() -> None:
    fake_proc = SimpleNamespace(
        stdout=iter(["remote-out"]),
        stderr=iter(["remote-err"]),
        returncode=7,
        wait=lambda: None,
    )
    fake_sandbox = MagicMock()
    fake_sandbox.exec.return_value = fake_proc
    saved = {key: os.environ.pop(key, None) for key in CURSOR_ENV_KEYS}
    saved_secret = os.environ.pop(MODAL_CURSOR_SECRET_NAME_ENV, None)
    try:
        os.environ["CURSOR_API_KEY"] = "test-key"
        with patch.object(modal.App, "lookup", return_value=app):
            with patch.object(modal.Sandbox, "create", return_value=fake_sandbox):
                code = run_malvin_remote(["--version"])
        assert code == 7
        fake_sandbox.detach.assert_called_once()
    finally:
        for key, value in saved.items():
            os.environ.pop(key, None) if value is None else os.environ.__setitem__(key, value)
        if saved_secret is None:
            os.environ.pop(MODAL_CURSOR_SECRET_NAME_ENV, None)
        else:
            os.environ[MODAL_CURSOR_SECRET_NAME_ENV] = saved_secret

def _test_modal_remote_missing_credentials() -> None:
    saved = {key: os.environ.pop(key, None) for key in CURSOR_ENV_KEYS}
    saved_secret = os.environ.pop(MODAL_CURSOR_SECRET_NAME_ENV, None)
    try:
        try:
            run_malvin_remote(["--version"])
        except click.ClickException as exc:
            assert "Cursor API key required" in str(exc)
        else:
            raise AssertionError("expected ClickException")
    finally:
        for key, value in saved.items():
            os.environ.pop(key, None) if value is None else os.environ.__setitem__(key, value)
        if saved_secret is None:
            os.environ.pop(MODAL_CURSOR_SECRET_NAME_ENV, None)
        else:
            os.environ[MODAL_CURSOR_SECRET_NAME_ENV] = saved_secret

def run_unit_tests() -> None:
    """UT-ARGV, UT-IGNORE, UT-RELAY, UT-EXIT, UT-MODAL, UT-CLICK — no Modal network."""
    _test_static_helpers()
    _test_cursor_and_stream()
    _test_sandbox_app()
    _test_modal_remote()
    _test_modal_remote_missing_credentials()
    _test_render_empty_argv_help()
    _test_empty_argv_help()
    _test_click_cli()

def _test_sandbox_app() -> None:
    test_sandbox_app_lookup(__name__, app, APP_NAME, sandbox_app)

def _test_render_empty_argv_help() -> None:
    fake_malvin = "Usage: malvin [COMMAND|REQUEST]...\n"
    fake_wrapper = "Usage: python ops/malvin_modal.py [OPTIONS]\n"
    ctx = MagicMock()
    ctx.get_help.return_value = fake_wrapper
    with patch(f"{__name__}.run_local_malvin_usage", return_value=fake_malvin):
        output = render_empty_argv_help(ctx)
    malvin_block, wrapper_block = output.split("\n\n", 1)
    assert malvin_block == fake_malvin.rstrip()
    assert wrapper_block == f"{fake_wrapper.rstrip()}\n"
    ctx.get_help.assert_called_once()

def _test_empty_argv_help() -> None:
    from toolchain_repos import load_ops_entry

    cli = load_ops_entry("malvin_modal").cli
    runner = CliRunner()
    fake_malvin = "Usage: malvin [COMMAND|REQUEST]...\n"
    with patch(f"{__name__}.run_local_malvin_usage", return_value=fake_malvin):
        with patch(f"{__name__}.run_malvin_remote") as mock_remote:
            result = runner.invoke(
                cli,
                [],
                prog_name="python ops/malvin_modal.py",
            )
    assert result.exit_code == 0, result.output
    malvin_block, wrapper_block = result.output.split("\n\n", 1)
    assert malvin_block == fake_malvin.rstrip()
    assert wrapper_block.startswith("Usage: python ops/malvin_modal.py")
    mock_remote.assert_not_called()

def _test_click_cli() -> None:
    from toolchain_repos import load_ops_entry

    cli = load_ops_entry("malvin_modal").cli
    runner = CliRunner()
    with patch(f"{__name__}.run_unit_tests") as mock_run_tests:
        result = runner.invoke(cli, ["--self-test"])
        assert result.exit_code == 0, result.output
        mock_run_tests.assert_called_once()
    fake_proc = SimpleNamespace(
        stdout=iter(["remote-out"]),
        stderr=iter(["remote-err"]),
        returncode=7,
        wait=lambda: None,
    )
    fake_sandbox = MagicMock()
    fake_sandbox.exec.return_value = fake_proc
    saved = {key: os.environ.pop(key, None) for key in CURSOR_ENV_KEYS}
    saved_secret = os.environ.pop(MODAL_CURSOR_SECRET_NAME_ENV, None)
    try:
        os.environ["CURSOR_API_KEY"] = "test-key"
        with patch.object(modal.App, "lookup", return_value=app):
            with patch.object(modal.Sandbox, "create", return_value=fake_sandbox):
                result = runner.invoke(cli, ["--version"])
        assert result.exit_code == 7
        fake_sandbox.detach.assert_called_once()
    finally:
        for key, value in saved.items():
            os.environ.pop(key, None) if value is None else os.environ.__setitem__(key, value)
        if saved_secret is None:
            os.environ.pop(MODAL_CURSOR_SECRET_NAME_ENV, None)
        else:
            os.environ[MODAL_CURSOR_SECRET_NAME_ENV] = saved_secret
