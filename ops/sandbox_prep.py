#!/usr/bin/env python3
"""Prepare a DeepSWE task sandbox after workspace mount.

Harbor Dockerfiles install dependencies at image build time into ``/app``. At
runtime the host workspace is mounted over ``/app``, which can desynchronize
editable installs and leave site-packages inconsistent with the checkout
(HISTORY: pydantic v1 vs v2 on FastAPI tasks).

``prepare_task_sandbox`` replays Dockerfile dependency-install RUN lines against
the mounted workspace (skipping clone/checkout and network fetches).
"""

from __future__ import annotations

import re
import subprocess
import time
from dataclasses import dataclass
from pathlib import Path
from typing import Any

import click

TIMEOUT_EXIT_CODE = 124


def _remaining_sec(deadline: float) -> float:
    return max(0.0, deadline - time.monotonic())


def _normalize_run_command(command: str) -> str:
    """Collapse Dockerfile line continuations into a single shell line."""
    no_continuations = command.replace("\\", " ")
    return " ".join(no_continuations.split())



def _run_shell(
    command: str,
    workspace: Path,
    *,
    timeout_sec: float | None = None,
) -> tuple[int, str, bool]:
    if timeout_sec is not None and timeout_sec <= 0:
        return TIMEOUT_EXIT_CODE, "phase deadline exhausted before prep command", True
    run_kwargs: dict[str, Any] = {
        "args": ["bash", "-lc", command],
        "cwd": str(workspace),
        "text": True,
        "capture_output": True,
        "check": False,
    }
    if timeout_sec is not None:
        run_kwargs["timeout"] = timeout_sec
    try:
        proc = subprocess.run(**run_kwargs)
    except subprocess.TimeoutExpired as exc:
        detail_parts: list[str] = []
        if exc.stdout:
            detail_parts.append(exc.stdout)
        if exc.stderr:
            detail_parts.append(exc.stderr)
        detail = "".join(detail_parts).strip() or "prep command timed out"
        click.echo(detail, err=True)
        return TIMEOUT_EXIT_CODE, detail, True
    if proc.stdout:
        click.echo(proc.stdout, nl=False)
    if proc.stderr:
        click.echo(proc.stderr, nl=False, err=True)
    detail = (proc.stderr or proc.stdout or "").strip()
    return proc.returncode, detail, False


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
    timed_out: bool = False

    def as_dict(self) -> dict[str, Any]:
        return {
            "sync_commands": list(self.sync_commands),
            "sync_warnings": list(self.sync_warnings),
            "probe_errors": list(self.probe_errors),
            "ok": self.ok,
            "timed_out": self.timed_out,
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


_REQUIREMENTS_FILE_RE = re.compile(r"-r\s+(\S+)")
_PKG_PIN_RE = re.compile(
    r"(?<![\w.-])([a-zA-Z0-9][a-zA-Z0-9._-]*)==([\d][\w.]*(?:\+[\w.-]+)?)"
)
_BASH_LC_RE = re.compile(r"""bash\s+-lc\s+(["'])(.*)\1""", re.DOTALL)
_PYDANTIC_PIN_RE = re.compile(r"^pydantic==([\d.]+)\s*(?:#.*)?$", re.MULTILINE)
_PYDANTIC_CORE_PIN_RE = re.compile(r"^pydantic-core==([\d.]+)\s*(?:#.*)?$", re.MULTILINE)


def collect_pip_install_intents(dockerfile_text: str) -> list[str]:
    """Return pip install shell segments from Dockerfile RUN lines (incl. ``bash -lc``)."""
    intents: list[str] = []
    for run in parse_dockerfile_run_commands(dockerfile_text):
        for segment in _split_shell_segments(run):
            if _is_pip_install_segment(segment):
                intents.append(segment)
            bash_match = _BASH_LC_RE.search(segment)
            if not bash_match:
                continue
            inner = bash_match.group(2)
            for part in re.split(r"[;&]", inner):
                part = part.strip()
                if part and _is_pip_install_segment(part):
                    intents.append(part)
    return intents


def _pins_from_requirements_file(requirements_path: Path) -> dict[str, str]:
    if not requirements_path.is_file():
        return {}
    pins: dict[str, str] = {}
    for raw in requirements_path.read_text(encoding="utf-8").splitlines():
        line = raw.strip()
        if not line or line.startswith("#"):
            continue
        match = _PKG_PIN_RE.search(line)
        if match:
            pins[match.group(1).lower()] = match.group(2)
    return pins


def collect_pinned_packages(workspace: Path, intents: list[str]) -> dict[str, str]:
    """Collect ``name==version`` pins from pip intents and referenced ``-r`` files."""
    pins: dict[str, str] = {}
    workspace = workspace.resolve()
    for intent in intents:
        for req_match in _REQUIREMENTS_FILE_RE.finditer(intent):
            pins.update(_pins_from_requirements_file(workspace / req_match.group(1)))
        for match in _PKG_PIN_RE.finditer(intent):
            pins[match.group(1).lower()] = match.group(2)
    return pins


def requirements_paths_from_dockerfile(dockerfile: Path) -> list[str]:
    """Return ``-r`` requirements paths referenced by Dockerfile bulk pip installs."""
    if not dockerfile.is_file():
        return []
    intents = collect_pip_install_intents(dockerfile.read_text(encoding="utf-8"))
    paths: list[str] = []
    for intent in intents:
        paths.extend(match.group(1) for match in _REQUIREMENTS_FILE_RE.finditer(intent))
    return paths


def read_pydantic_pins_from_requirements(requirements_path: Path) -> tuple[str | None, str | None]:
    """Return ``(pydantic, pydantic-core)`` pins from a requirements file, if present."""
    if not requirements_path.is_file():
        return None, None
    text = requirements_path.read_text(encoding="utf-8")
    pydantic_match = _PYDANTIC_PIN_RE.search(text)
    core_match = _PYDANTIC_CORE_PIN_RE.search(text)
    return (
        pydantic_match.group(1) if pydantic_match else None,
        core_match.group(1) if core_match else None,
    )


def pydantic_pins_for_cache_bust(
    dockerfile: Path | None,
    workspace: Path | None = None,
) -> tuple[str | None, str | None]:
    """Return task pydantic pins when present in workspace requirements."""
    if dockerfile is None or not dockerfile.is_file() or workspace is None:
        return None, None
    workspace = workspace.resolve()
    for req_rel in requirements_paths_from_dockerfile(dockerfile):
        pydantic_ver, core_ver = read_pydantic_pins_from_requirements(workspace / req_rel)
        if pydantic_ver is not None:
            return pydantic_ver, core_ver
    return None, None


def pins_for_task(
    dockerfile: Path | None,
    workspace: Path | None = None,
) -> dict[str, str]:
    """Pinned packages for a task workspace (Modal image build / cache bust)."""
    if dockerfile is None or not dockerfile.is_file() or workspace is None:
        return {}
    intents = collect_pip_install_intents(dockerfile.read_text(encoding="utf-8"))
    return collect_pinned_packages(workspace.resolve(), intents)


# Smoke imports after Harbor registry pull: pydantic v1 layers and httpx2 namespace drift.
_DRIFT_PROBE_SCRIPT = (
    "python3 -c '"
    "import importlib.util, sys; "
    "bad=[]; "
    "s=importlib.util.find_spec(chr(112)+chr(121)+chr(100)+chr(97)+chr(110)+chr(116)+chr(105)+chr(99)); "
    "exec(\"import pydantic\\nif pydantic.__version__.startswith(\\\"1.\\\"): bad.append(\\\"pydantic_v1\\\")\") if s else None; "
    "s=importlib.util.find_spec(chr(104)+chr(116)+chr(116)+chr(112)+chr(120)); "
    "exec(\"import httpx\\nif httpx.__name__ != \\\"httpx\\\": bad.append(\\\"httpx2\\\")\") if s else None; "
    "sys.exit(1 if bad else 0)'"
)


def run_drift_probe_commands() -> list[str]:
    """Shell commands that reinstall drift-prone packages only when probe fails."""
    drift_fix = (
        "'starlette==1.0.0' 'click==8.3.1' 'typer==0.25.1'"
    )
    return [
        f"{_DRIFT_PROBE_SCRIPT} || "
        f"pip install --no-cache-dir --force-reinstall {drift_fix}",
    ]


def _pydantic_v1_eviction_command() -> str:
    """Evict stale pydantic v1 when the image has pydantic but no task pin."""
    return (
        "python3 -c \""
        "import importlib.util, sys; "
        "spec=importlib.util.find_spec('pydantic'); "
        "import pydantic; "
        "sys.exit(1 if pydantic.__version__.startswith('1.') else 0)"
        "\" 2>/dev/null || "
        "pip install --no-cache-dir 'pydantic>=2,<3'"
    )


def registry_image_cache_bust_commands(
    dockerfile: Path | None = None,
    workspace: Path | None = None,
) -> list[str]:
    """Modal registry cache bust from task pins, then conditional drift repair.

    Modal may serve stale Harbor registry layers (pydantic v1). Re-running the full
    editable ``pip install -e`` after pull can upgrade starlette and break Harbor
    verifiers that expect ``httpx`` (not ``httpx2``). Task-derived pins are applied
    first; starlette/click/typer are reinstalled only when the drift probe fails.
    """
    pins = pins_for_task(dockerfile, workspace)
    cmds: list[str] = []
    if pins:
        pkg_args = [f"'{name}=={ver}'" for name, ver in sorted(pins.items())]
        cmds.append(
            "pip install --no-cache-dir --force-reinstall " + " ".join(pkg_args)
        )
    else:
        cmds.append(_pydantic_v1_eviction_command())
    cmds.extend(run_drift_probe_commands())
    return cmds


def prepare_task_sandbox(
    spec: Any,
    workspace: Path,
    *,
    checks: str,
    dry_run: bool = False,
    deadline: float | None = None,
) -> SandboxPrepResult:
    """Replay Harbor Dockerfile install steps against the mounted workspace."""
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
        timeout_sec = _remaining_sec(deadline) if deadline is not None else None
        code, detail, timed_out = _run_shell(command, workspace, timeout_sec=timeout_sec)
        if timed_out:
            sync_warnings.append(
                f"sync timed out for {command!r}"
                + (f": {detail}" if detail else "")
            )
            click.echo("Prep sync timed out", err=True)
            return SandboxPrepResult(
                sync_commands=tuple(sync_commands),
                sync_warnings=tuple(sync_warnings),
                probe_errors=(),
                ok=False,
                timed_out=True,
            )
        if code != 0:
            sync_warnings.append(
                f"sync exit {code} for {command!r}"
                + (f": {detail}" if detail else "")
            )
            click.echo(f"Prep sync warning (exit {code})", err=True)

    return SandboxPrepResult(
        sync_commands=tuple(sync_commands),
        sync_warnings=tuple(sync_warnings),
        probe_errors=(),
        ok=True,
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



def _test_registry_image_cache_bust_commands() -> None:
    import tempfile

    text = """FROM base
RUN pip install --no-cache-dir -e ".[all]" && pip install --no-cache-dir pytest dirty-equals>=0.9.0
"""
    with tempfile.TemporaryDirectory() as tmp:
        dockerfile = Path(tmp) / "Dockerfile"
        dockerfile.write_text(text, encoding="utf-8")
        cmds = registry_image_cache_bust_commands(dockerfile)
    assert len(cmds) >= 2, cmds
    assert cmds[0].startswith("python3 -c") or cmds[0].startswith("pip install")
    assert "starlette==1.0.0" in cmds[-1]
    assert "pydantic==2.13.4" not in " ".join(cmds)


def _test_registry_image_cache_bust_adaptix_pydantic_pin() -> None:
    import tempfile

    tasks_root = Path(__file__).resolve().parent.parent.parent / "deep-swe" / "tasks"
    dockerfile = tasks_root / "adaptix-name-mapping-aliases" / "environment" / "Dockerfile"
    if not dockerfile.is_file():
        return
    with tempfile.TemporaryDirectory() as tmp:
        workspace = Path(tmp)
        req_dir = workspace / "requirements"
        req_dir.mkdir()
        (req_dir / "test_extra_new.txt").write_text(
            "pydantic==2.10.3\npydantic-core==2.27.1\n",
            encoding="utf-8",
        )
        cmds = registry_image_cache_bust_commands(dockerfile, workspace=workspace)
    assert any("pydantic==2.10.3" in c for c in cmds), cmds
    assert any("pydantic-core==2.27.1" in c for c in cmds), cmds
    assert not any("pydantic==2.13.4" in c for c in cmds), cmds


def _test_pydantic_pins_for_cache_bust_reads_requirements() -> None:
    import tempfile

    with tempfile.TemporaryDirectory() as tmp:
        root = Path(tmp)
        dockerfile = root / "Dockerfile"
        dockerfile.write_text(
            "RUN pip install -r requirements/dev.txt\n",
            encoding="utf-8",
        )
        (root / "requirements").mkdir()
        (root / "requirements" / "dev.txt").write_text("pydantic==1.2.3\n", encoding="utf-8")
        pins = pins_for_task(dockerfile, workspace=root)
    assert pins.get("pydantic") == "1.2.3"


def _test_collect_pip_install_intents_bash_lc() -> None:
    text = """FROM base
RUN bash -lc "if [ -f requirements.txt ]; then pip install -r requirements.txt; fi; pip install -e . pytest"
"""
    intents = collect_pip_install_intents(text)
    assert any("-r requirements.txt" in i for i in intents), intents
    assert any("-e ." in i for i in intents), intents


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
    _test_registry_image_cache_bust_adaptix_pydantic_pin()
    _test_pydantic_pins_for_cache_bust_reads_requirements()
    _test_collect_pip_install_intents_bash_lc()
    _test_dockerfile_bulk_pip_commands_fastapi()
    _test_workspace_sync_commands_fastapi_task_dockerfile()
    _test_should_replay_skips_apt_and_git()
    click.echo("sandbox_prep self-tests passed")


if __name__ == "__main__":
    run_self_tests()
