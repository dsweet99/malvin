#!/usr/bin/env python3
"""Prepare a DeepSWE task sandbox after workspace mount.

Harbor Dockerfiles install dependencies at image build time into ``/app``. At
runtime the host workspace is mounted over ``/app``, which can desynchronize
editable installs and leave site-packages inconsistent with the checkout
(HISTORY: pydantic v1 vs v2 on FastAPI tasks).

``prepare_task_sandbox`` replays Dockerfile dependency-install RUN lines against
the mounted workspace (skipping clone/checkout and network fetches), then probes
that quality-gate tools are callable offline.
"""

from __future__ import annotations

import re
import subprocess
from dataclasses import dataclass
from pathlib import Path
from typing import Any

import click


def _normalize_run_command(command: str) -> str:
    """Collapse Dockerfile line continuations into a single shell line."""
    no_continuations = command.replace("\\", " ")
    return " ".join(no_continuations.split())


def _canonical_tool(line: str) -> str:
    parts = line.strip().split()
    return parts[0].lower() if parts else ""


def _run_shell(command: str, workspace: Path) -> tuple[int, str]:
    proc = subprocess.run(
        ["bash", "-lc", command],
        cwd=str(workspace),
        text=True,
        capture_output=True,
        check=False,
    )
    if proc.stdout:
        click.echo(proc.stdout, nl=False)
    if proc.stderr:
        click.echo(proc.stderr, nl=False, err=True)
    detail = (proc.stderr or proc.stdout or "").strip()
    return proc.returncode, detail


# RUN bodies we never replay: workspace already has the checkout; no network in agent sandboxes.
_SKIP_RUN_SUBSTRINGS = (
    "git clone",
    "git checkout",
    "git submodule",
    "curl ",
    "wget ",
    "apt-get",
    "apt install",
    "rustup",
    "cargo install --path",
    "cursor.com/install",
)

# RUN bodies that reconcile deps after a workspace overlay.
_SYNC_RUN_SUBSTRINGS = (
    "pip install",
    "pip3 install",
    "python -m pip",
    "python3 -m pip",
    "uv sync",
    "uv pip",
    "go mod",
    "cargo build",
    "cargo fetch",
    "npm ci",
    "npm install",
    "poetry install",
    "pdm install",
)


@dataclass(frozen=True)
class SandboxPrepResult:
    sync_commands: tuple[str, ...]
    sync_warnings: tuple[str, ...]
    probe_errors: tuple[str, ...]
    ok: bool

    def as_dict(self) -> dict[str, Any]:
        return {
            "sync_commands": list(self.sync_commands),
            "sync_warnings": list(self.sync_warnings),
            "probe_errors": list(self.probe_errors),
            "ok": self.ok,
        }


def _join_continued_run_lines(lines: list[str]) -> list[str]:
    """Merge Dockerfile RUN instructions continued with backslashes."""
    runs: list[str] = []
    current: str | None = None
    for raw in lines:
        stripped = raw.strip()
        if not stripped or stripped.startswith("#"):
            continue
        if stripped.upper().startswith("RUN "):
            if current is not None:
                runs.append(current)
            current = stripped[4:].strip()
            if not stripped.endswith("\\"):
                runs.append(_normalize_run_command(current))
                current = None
            continue
        if current is None:
            continue
        if stripped.endswith("\\"):
            current += " " + stripped[:-1].strip()
        else:
            current += " " + stripped
            runs.append(_normalize_run_command(current))
            current = None
    if current is not None:
        runs.append(_normalize_run_command(current))
    return runs


def parse_dockerfile_run_commands(dockerfile_text: str) -> list[str]:
    """Return shell bodies of Dockerfile RUN instructions in file order."""
    return _join_continued_run_lines(dockerfile_text.splitlines())


def should_replay_run_command(command: str) -> bool:
    """True when a RUN line should be replayed after workspace mount."""
    lower = command.lower()
    if any(skip in lower for skip in _SKIP_RUN_SUBSTRINGS):
        return False
    return any(sync in lower for sync in _SYNC_RUN_SUBSTRINGS)


def _split_shell_segments(command: str) -> list[str]:
    return [segment.strip() for segment in re.split(r"\s*&&\s*", command) if segment.strip()]


_EDITABLE_PIP_FLAG = re.compile(r"(?:^|\s)(?:-e|--editable)\s")
_PIP_INSTALL_RE = re.compile(r"(?:^|\s)(?:pip3?|python3? -m pip)(?:\s|$)")


def _is_pip_install_segment(segment: str) -> bool:
    """True for ``pip`` / ``pip3`` / ``python -m pip`` install segments."""
    return bool(_PIP_INSTALL_RE.search(segment))


def _is_editable_pip_segment(segment: str) -> bool:
    """True when a shell segment is ``pip install -e`` (not ``dirty-equals``)."""
    return _is_pip_install_segment(segment) and bool(_EDITABLE_PIP_FLAG.search(segment))


def _is_bulk_pip_segment(segment: str) -> bool:
    """True for non-editable pip installs that require PyPI/registry network."""
    return _is_pip_install_segment(segment) and not _is_editable_pip_segment(segment)


def _offline_editable_command(command: str) -> str:
    """Replay editable installs without PyPI in offline agent sandboxes."""
    out = command.strip()
    if "--no-deps" not in out:
        out += " --no-deps"
    if "--no-build-isolation" not in out:
        out += " --no-build-isolation"
    return out


def _sync_commands_from_runs(runs: list[str]) -> list[str]:
    sync: list[str] = []
    for cmd in runs:
        if not should_replay_run_command(cmd):
            continue
        segments = _split_shell_segments(cmd)
        if any(_is_editable_pip_segment(segment) for segment in segments):
            # Harbor/Modal images already install editables at build time; offline
            # PEP 517 replay after workspace mount often fails (pdm.backend) and can
            # desync site-packages (pydantic v1/v2, httpx2).
            continue
        if any(_is_bulk_pip_segment(segment) for segment in segments):
            # Bulk pip needs PyPI; replay at Modal image build, not in agent sandbox.
            continue
        sync.append(cmd)
    return sync


def workspace_sync_commands_from_dockerfile(dockerfile: Path) -> list[str]:
    """Non-editable dependency-install RUN lines to replay against a mounted workspace.

    Editable ``pip install -e`` segments are skipped: Harbor/Modal images already
    install them at build time, and offline PEP 517 replay after a workspace overlay
    often fails or desyncs site-packages.
    """
    if not dockerfile.is_file():
        return []
    runs = parse_dockerfile_run_commands(dockerfile.read_text(encoding="utf-8"))
    return _sync_commands_from_runs(runs)


def dockerfile_image_build_commands(dockerfile: Path) -> list[str]:
    """Editable pip segments to re-run during Modal image build (network on).

    Modal may cache Dockerfile ``pip install -e`` layers incorrectly (e.g. mars-base
    pydantic v1 survives). Re-running editable segments after ``from_dockerfile``
    busts the cache without replaying bulk ``pip install`` waves that can upgrade
    transitive deps (starlette) and break Harbor verifiers (httpx2).
    """
    if not dockerfile.is_file():
        return []
    runs = parse_dockerfile_run_commands(dockerfile.read_text(encoding="utf-8"))
    commands: list[str] = []
    for cmd in runs:
        if not should_replay_run_command(cmd):
            continue
        segments = _split_shell_segments(cmd)
        editable = [segment for segment in segments if _is_editable_pip_segment(segment)]
        if editable:
            commands.extend(editable)
        else:
            commands.append(cmd)
    return commands


def dockerfile_bulk_pip_commands(dockerfile: Path) -> list[str]:
    """Non-editable ``pip install`` segments from Dockerfile RUN lines (build-time replay)."""
    if not dockerfile.is_file():
        return []
    runs = parse_dockerfile_run_commands(dockerfile.read_text(encoding="utf-8"))
    commands: list[str] = []
    for cmd in runs:
        if not should_replay_run_command(cmd):
            continue
        segments = _split_shell_segments(cmd)
        bulk = [segment for segment in segments if _is_bulk_pip_segment(segment)]
        commands.extend(bulk)
    return commands


def registry_image_cache_bust_commands(dockerfile: Path | None = None) -> list[str]:
    """Lightweight Modal registry cache bust that avoids starlette/httpx2 drift.

    Modal may serve stale Harbor registry layers (pydantic v1). Re-running the full
    editable ``pip install -e`` after pull can upgrade starlette and break Harbor
    verifiers that expect ``httpx`` (not ``httpx2``). Replaying bulk test-dependency
    pip segments can upgrade click/typer and fail baseline tests under
    ``filterwarnings = error``. Pin pydantic v2 and starlette only.
    """
    _ = dockerfile
    return [
        "pip install --no-cache-dir --force-reinstall "
        "'pydantic==2.13.4' 'starlette==1.0.0' "
        "'click==8.3.1' 'typer==0.25.1'",
    ]


def probe_check_tools(checks: str, workspace: Path) -> list[str]:
    """Return human-readable errors when gate tools are missing or broken offline."""
    errors: list[str] = []
    seen: set[str] = set()
    for line in checks.splitlines():
        trimmed = line.strip()
        if not trimmed:
            continue
        tool = _canonical_tool(trimmed)
        if tool in seen:
            continue
        seen.add(tool)
        if tool == "kiss":
            proc = subprocess.run(
                ["kiss", "--version"],
                cwd=workspace,
                capture_output=True,
                text=True,
                check=False,
            )
            if proc.returncode != 0:
                errors.append("kiss is not callable")
        elif tool == "ruff":
            proc = subprocess.run(
                ["ruff", "--version"],
                cwd=workspace,
                capture_output=True,
                text=True,
                check=False,
            )
            if proc.returncode != 0:
                errors.append("ruff is not callable")
        elif tool == "pytest":
            proc = subprocess.run(
                ["python3", "-c", "import pytest"],
                cwd=workspace,
                capture_output=True,
                text=True,
                check=False,
            )
            if proc.returncode != 0:
                detail = (proc.stderr or proc.stdout or "").strip()
                errors.append(f"pytest import failed: {detail or 'unknown'}")
        elif tool == "mypy":
            proc = subprocess.run(
                ["mypy", "--version"],
                cwd=workspace,
                capture_output=True,
                text=True,
                check=False,
            )
            if proc.returncode != 0:
                errors.append("mypy is not callable")
        elif tool == "cargo":
            if "clippy" in trimmed or "test" in trimmed or "nextest" in trimmed:
                proc = subprocess.run(
                    ["cargo", "--version"],
                    cwd=workspace,
                    capture_output=True,
                    text=True,
                    check=False,
                )
                if proc.returncode != 0:
                    errors.append("cargo is not callable")
    return errors


def prepare_task_sandbox(
    spec: Any,
    workspace: Path,
    *,
    checks: str,
    dry_run: bool = False,
) -> SandboxPrepResult:
    """Replay Harbor Dockerfile install steps and verify offline gate-tool readiness."""
    workspace = workspace.resolve()
    sync_commands = workspace_sync_commands_from_dockerfile(spec.dockerfile)
    if sync_commands:
        click.echo(
            f"Preparing sandbox: replaying {len(sync_commands)} Dockerfile install step(s)"
        )
    sync_warnings: list[str] = []
    for command in sync_commands:
        click.echo(f"Prep sync: {command}")
        if dry_run:
            continue
        code, detail = _run_shell(command, workspace)
        if code != 0:
            sync_warnings.append(
                f"sync exit {code} for {command!r}"
                + (f": {detail}" if detail else "")
            )
            click.echo(f"Prep sync warning (exit {code})", err=True)

    probe_errors: list[str] = []
    if checks.strip():
        if dry_run:
            click.echo("Prep probe (dry-run): would verify gate tools offline")
        else:
            probe_errors = probe_check_tools(checks, workspace)
            if probe_errors:
                for err in probe_errors:
                    click.echo(f"Prep probe failed: {err}", err=True)

    ok = not probe_errors
    if not ok and not dry_run:
        raise click.ClickException(
            "Sandbox prep failed: " + "; ".join(probe_errors)
        )
    return SandboxPrepResult(
        sync_commands=tuple(sync_commands),
        sync_warnings=tuple(sync_warnings),
        probe_errors=tuple(probe_errors),
        ok=ok,
    )


def _test_parse_dockerfile_run_commands_multiline() -> None:
    text = """FROM base
RUN pip install --no-cache-dir pytest && \\
    pip install -e .
RUN git clone https://example.com/foo .
"""
    runs = parse_dockerfile_run_commands(text)
    assert len(runs) == 2, runs
    assert "pip install --no-cache-dir pytest" in runs[0]
    assert runs[1].startswith("git clone")


def _test_workspace_sync_commands_bandit() -> None:
    text = """RUN git clone https://github.com/PyCQA/bandit.git . && git checkout abc
RUN pip install pytest && pip install -e .
"""
    runs = parse_dockerfile_run_commands(text)
    sync = _sync_commands_from_runs(runs)
    assert sync == [], sync


def _test_workspace_sync_commands_fastapi() -> None:
    text = """RUN git clone https://github.com/fastapi/fastapi .
RUN pip install --no-cache-dir -e ".[all]" && pip install --no-cache-dir pytest
"""
    runs = parse_dockerfile_run_commands(text)
    sync = _sync_commands_from_runs(runs)
    assert sync == [], sync


def _test_editable_pip_segment_ignores_dirty_equals() -> None:
    bulk = (
        "pip install --no-cache-dir pytest dirty-equals>=0.9.0 inline-snapshot>=0.21.1"
    )
    assert not _is_editable_pip_segment(bulk)
    assert _is_editable_pip_segment('pip install --no-cache-dir -e ".[all]"')
    assert _is_editable_pip_segment('pip3 install -e ".[pandas]"')
    assert _is_bulk_pip_segment("pip3 install pytest covdefaults")


def _test_infra_abort_dockerfile_sync_is_offline() -> None:
    """INFRA_ABORT_TASKS must not replay network-fetching pip in agent sandbox prep."""
    tasks_root = Path(__file__).resolve().parent.parent.parent / "deep-swe" / "tasks"
    if not tasks_root.is_dir():
        return
    slugs = (
        "igel-persist-feature-schema",
        "mnamer-daemon-watch-lifecycle",
        "narwhals-rolling-window-suite",
        "kombu-single-active-consumer-priority",
        "mashumaro-flattened-dataclass-fields",
    )
    for slug in slugs:
        dockerfile = tasks_root / slug / "environment" / "Dockerfile"
        if not dockerfile.is_file():
            continue
        sync = workspace_sync_commands_from_dockerfile(dockerfile)
        assert sync == [], (slug, sync)
        bulk = dockerfile_bulk_pip_commands(dockerfile)
        if bulk:
            assert all(_is_bulk_pip_segment(cmd) for cmd in bulk), (slug, bulk)


def _test_dockerfile_image_build_commands_fastapi() -> None:
    import tempfile

    text = """FROM base
RUN git clone https://github.com/fastapi/fastapi .
RUN pip install --no-cache-dir -e ".[all]" && pip install --no-cache-dir pytest dirty-equals>=0.9.0
"""
    with tempfile.TemporaryDirectory() as tmp:
        dockerfile = Path(tmp) / "Dockerfile"
        dockerfile.write_text(text, encoding="utf-8")
        build = dockerfile_image_build_commands(dockerfile)
    assert len(build) == 1, build
    assert '-e ".[all]"' in build[0]
    assert "pytest" not in build[0]


def _test_workspace_sync_commands_fastapi_task_dockerfile() -> None:
    tasks_root = Path(__file__).resolve().parent.parent.parent / "deep-swe" / "tasks"
    dockerfile = tasks_root / "fastapi-deprecation-response-headers" / "environment" / "Dockerfile"
    if not dockerfile.is_file():
        return
    sync = workspace_sync_commands_from_dockerfile(dockerfile)
    assert sync == [], sync


def _test_should_replay_skips_apt_and_git() -> None:
    assert not should_replay_run_command("apt-get update && apt-get install -y build-essential")
    assert not should_replay_run_command("git clone https://github.com/foo .")
    assert should_replay_run_command("go mod download")


def _test_probe_check_tools_unknown_tool_ignored() -> None:
    import tempfile

    with tempfile.TemporaryDirectory() as tmp:
        errors = probe_check_tools("custom-linter .\n", Path(tmp))
        assert errors == []


def _test_registry_image_cache_bust_commands() -> None:
    import tempfile

    text = """FROM base
RUN pip install --no-cache-dir -e ".[all]" && pip install --no-cache-dir pytest dirty-equals>=0.9.0
"""
    with tempfile.TemporaryDirectory() as tmp:
        dockerfile = Path(tmp) / "Dockerfile"
        dockerfile.write_text(text, encoding="utf-8")
        cmds = registry_image_cache_bust_commands(dockerfile)
    assert cmds[0].startswith("pip install")
    assert "pydantic==2.13.4" in cmds[0]
    assert "starlette==1.0.0" in cmds[0]
    assert "click==8.3.1" in cmds[0]
    assert "typer==0.25.1" in cmds[0]
    assert len(cmds) == 1


def _test_dockerfile_bulk_pip_commands_fastapi() -> None:
    tasks_root = Path(__file__).resolve().parent.parent.parent / "deep-swe" / "tasks"
    dockerfile = tasks_root / "fastapi-deprecation-response-headers" / "environment" / "Dockerfile"
    if not dockerfile.is_file():
        return
    bulk = dockerfile_bulk_pip_commands(dockerfile)
    assert bulk, bulk
    assert all("pip install" in cmd for cmd in bulk)
    assert all('-e "' not in cmd for cmd in bulk)


def run_self_tests() -> None:
    _test_parse_dockerfile_run_commands_multiline()
    _test_workspace_sync_commands_bandit()
    _test_workspace_sync_commands_fastapi()
    _test_editable_pip_segment_ignores_dirty_equals()
    _test_infra_abort_dockerfile_sync_is_offline()
    _test_dockerfile_image_build_commands_fastapi()
    _test_registry_image_cache_bust_commands()
    _test_dockerfile_bulk_pip_commands_fastapi()
    _test_workspace_sync_commands_fastapi_task_dockerfile()
    _test_should_replay_skips_apt_and_git()
    _test_probe_check_tools_unknown_tool_ignored()
    click.echo("sandbox_prep self-tests passed")


if __name__ == "__main__":
    run_self_tests()
