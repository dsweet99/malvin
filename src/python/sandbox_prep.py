#!/usr/bin/env python3
"""Prepare a DeepSWE task sandbox with strict declared-dependency correctness.

Harbor Dockerfiles install dependencies at image build time into ``/app``. At
runtime the host workspace is mounted over ``/app``, which can desynchronize
editable installs and leave site-packages inconsistent with the checkout
(HISTORY: pydantic v1 vs v2 on FastAPI tasks).

Two-phase prep enforces a strict contract for Python tasks:

1. **Image build (network on):** reconcile declared dependencies from Dockerfile
   pins, ``pyproject.toml``, and ``uv.lock``, then run mandatory verification
   probes. Image build fails when probes fail after reconcile.
2. **Runtime prep (network off):** offline editable replay (``--no-deps
   --no-build-isolation``) plus verification probes. Fail fast with a clear error
   when sync or probes fail — do not run malvin in a known-bad environment.
"""

from __future__ import annotations

import base64
import os
import re
import shlex
import subprocess
import sys
import tempfile
import time
from dataclasses import dataclass
from pathlib import Path
from typing import Any

import click

try:
    import tomllib
except ModuleNotFoundError:  # pragma: no cover - py310
    import tomli as tomllib  # type: ignore[no-redef]

from harbor_tests import (
    added_python_sources_from_patch,
    collect_only_pytest_command,
    distribution_name_for_import,
    harbor_imports_from_tests_dir,
    resolve_harbor_test_sh_body,
    test_sh_invokes_pytest,
)
from toolchain_repos import malvin_repo_root

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


def _is_network_only_segment(segment: str) -> bool:
    """True for install segments that need registry/network (not offline replay)."""
    lower = segment.lower()
    return (
        _is_bulk_pip_segment(segment)
        or "poetry install" in lower
        or "pnpm install" in lower
        or "npm ci" in lower
        or "npm install" in lower
        or "cargo fetch" in lower
        or "cargo build" in lower
        or "go mod" in lower
    )


def _sync_commands_from_runs(runs: list[str], *, offline_editable: bool = True) -> list[str]:
    sync: list[str] = []
    for cmd in runs:
        if not should_replay_run_command(cmd):
            continue
        segments = _split_shell_segments(cmd)
        editable = [segment for segment in segments if _is_editable_pip_segment(segment)]
        network_only = [segment for segment in segments if _is_network_only_segment(segment)]
        non_pip = [
            segment
            for segment in segments
            if not _is_pip_install_segment(segment) and not _is_network_only_segment(segment)
        ]
        if offline_editable:
            for segment in editable:
                sync.append(_offline_editable_command(segment))
        if network_only or editable:
            continue
        if non_pip:
            sync.append(cmd)
    return sync


def workspace_sync_commands_from_dockerfile(
    dockerfile: Path,
    *,
    offline_editable: bool = True,
) -> list[str]:
    """Dependency-install RUN lines to replay offline against a mounted workspace.

    Bulk ``pip install`` segments are skipped (network). Editable ``pip install -e``
    segments are replayed with ``--no-deps --no-build-isolation``.
    """
    if not dockerfile.is_file():
        return []
    runs = parse_dockerfile_run_commands(dockerfile.read_text(encoding="utf-8"))
    return _sync_commands_from_runs(runs, offline_editable=offline_editable)


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


_REQUIREMENTS_FILE_RE = re.compile(r"(?:^|\s)-r\s+(\S+)")
_PKG_PIN_RE = re.compile(
    r"(?<![\w.-])([a-zA-Z0-9][a-zA-Z0-9._-]*)==([\d][\w.]*(?:\+[\w.-]+)?)"
)
_BASH_LC_RE = re.compile(r"""bash\s+-lc\s+(["'])(.*)\1""", re.DOTALL)
# Isolate real pip-install commands inside messy shell (conditionals, bash -lc).
_PIP_INSTALL_CMD_RE = re.compile(
    r"(?:(?:python3?|\$\{?PYTHON[^}\s]*\}?)\s+-m\s+)?pip3?\s+install\b[^;\n]*",
    re.IGNORECASE,
)
_PYDANTIC_PIN_RE = re.compile(r"^pydantic==([\d.]+)\s*(?:#.*)?$", re.MULTILINE)
_PYDANTIC_CORE_PIN_RE = re.compile(r"^pydantic-core==([\d.]+)\s*(?:#.*)?$", re.MULTILINE)
_SHELL_NOISE_NAMES = frozenset(
    {
        "fi",
        "if",
        "then",
        "else",
        "elif",
        "do",
        "done",
        "for",
        "while",
        "in",
        "pip",
        "pip3",
        "python",
        "python3",
        "install",
        "true",
        "false",
        "apt-get",
        "apt",
        "npm",
        "yarn",
        "poetry",
        "uv",
        "bash",
        "sh",
        "sudo",
        "command",
        "type",
        "which",
        # setup.py / metadata string literals often mistaken for requirements
        "any",
        "author",
        "contact",
        "doc",
        "docs",
        "extras",
        "homepage",
        "license",
        "requirements",
        "utf-8",
        "version",
        "description",
        "keywords",
        "maintainer",
        "platforms",
        "url",
        "setup",
        "test",
        "tests",
        "testing",
        "r",
        "t",
    }
)


def _is_plausible_distribution_name(name: str) -> bool:
    """False for setup.py string noise and requirements filenames mistaken as packages."""
    if not name or len(name) < 2:
        return False
    if name in _SHELL_NOISE_NAMES:
        return False
    # Filenames surviving normalize (``azureservicebus.txt``).
    if "." in name or "/" in name:
        return False
    return True


def _extract_pip_install_commands(shell_text: str) -> list[str]:
    """Return normalized ``pip install …`` commands found in *shell_text*."""
    found: list[str] = []
    for match in _PIP_INSTALL_CMD_RE.finditer(shell_text):
        cmd = " ".join(match.group(0).split()).rstrip('"').rstrip("'")
        if cmd:
            found.append(cmd)
    return found


def collect_pip_install_intents(dockerfile_text: str) -> list[str]:
    """Return pip install shell segments from Dockerfile RUN lines (incl. ``bash -lc``)."""
    intents: list[str] = []
    seen: set[str] = set()

    def _add(command: str) -> None:
        normalized = " ".join(command.split())
        if not normalized or normalized in seen:
            return
        if not _is_pip_install_segment(normalized):
            return
        seen.add(normalized)
        intents.append(normalized)

    for run in parse_dockerfile_run_commands(dockerfile_text):
        for segment in _split_shell_segments(run):
            _add(segment)
            bash_match = _BASH_LC_RE.search(segment)
            if bash_match:
                for cmd in _extract_pip_install_commands(bash_match.group(2)):
                    _add(cmd)
            else:
                for cmd in _extract_pip_install_commands(segment):
                    _add(cmd)
    return intents


def _pins_from_requirements_file(requirements_path: Path) -> dict[str, str]:
    if not requirements_path.is_file():
        return {}
    pins: dict[str, str] = {}
    for raw in requirements_path.read_text(encoding="utf-8").splitlines():
        line = _strip_requirement_comment(raw.strip())
        if not line or line.startswith("#"):
            continue
        match = _PKG_PIN_RE.search(line)
        if match:
            pins[match.group(1).lower()] = match.group(2)
    return pins


def _requirement_line_package(line: str) -> tuple[str, str] | None:
    """Return ``(normalized_name, remainder_spec)`` for a requirements line, if any."""
    stripped = _strip_requirement_comment(line.strip())
    if not stripped or stripped.startswith("#"):
        return None
    if stripped.startswith(("-e", "--editable", "-r", "--requirement", "-c", "--constraint")):
        return None
    if stripped.startswith(("-", ".", "/", "~")):
        return None
    dep = stripped.split(";", 1)[0].strip()
    dep = dep.split("[", 1)[0].strip()
    if "@" in dep:
        return None
    match = re.match(r"^([A-Za-z0-9][\w.-]*)(.*)$", dep)
    if not match:
        return None
    name = _normalize_package_name(match.group(1))
    if name in _SHELL_NOISE_NAMES:
        return None
    return name, match.group(2).strip()


def _constraints_from_requirements_file(requirements_path: Path) -> dict[str, str]:
    """Collect non-``==`` version constraints (``>=``, ``~=``, …) from a requirements file."""
    if not requirements_path.is_file():
        return {}
    constraints: dict[str, str] = {}
    for raw in requirements_path.read_text(encoding="utf-8").splitlines():
        parsed = _requirement_line_package(raw)
        if not parsed:
            continue
        name, rest = parsed
        if not rest or rest.startswith("=="):
            continue
        if rest.startswith((">=", "<=", "!=", "~=", ">", "<")):
            constraints[name] = rest
    return constraints


def _unpinned_from_requirements_file(requirements_path: Path) -> frozenset[str]:
    """Bare package names (no version operator) from a requirements file."""
    if not requirements_path.is_file():
        return frozenset()
    names: set[str] = set()
    for raw in requirements_path.read_text(encoding="utf-8").splitlines():
        parsed = _requirement_line_package(raw)
        if not parsed:
            continue
        name, rest = parsed
        if not rest:
            names.add(name)
    return frozenset(names)


def _editable_lines_from_requirements_file(requirements_path: Path) -> list[str]:
    """Return synthetic ``pip install -e …`` intents for editable lines in *requirements_path*."""
    if not requirements_path.is_file():
        return []
    lines: list[str] = []
    for raw in requirements_path.read_text(encoding="utf-8").splitlines():
        stripped = raw.strip()
        if not stripped or stripped.startswith("#"):
            continue
        if stripped.startswith("-e ") or stripped.startswith("--editable "):
            target = stripped.split(None, 1)[1].strip()
            lines.append(f"pip install -e {target}")
        elif stripped.startswith("-e=") or stripped.startswith("--editable="):
            target = stripped.split("=", 1)[1].strip()
            lines.append(f"pip install -e {target}")
    return lines


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


def collect_requirement_constraints(workspace: Path, intents: list[str]) -> dict[str, str]:
    """Collect non-equality version constraints from referenced requirements files."""
    constraints: dict[str, str] = {}
    workspace = workspace.resolve()
    for intent in intents:
        for req_match in _REQUIREMENTS_FILE_RE.finditer(intent):
            constraints.update(
                _constraints_from_requirements_file(workspace / req_match.group(1))
            )
    return constraints


def collect_requirement_unpinned_names(workspace: Path, intents: list[str]) -> frozenset[str]:
    """Bare names from referenced requirements files (and nested ``-r`` editables handled elsewhere)."""
    names: set[str] = set()
    workspace = workspace.resolve()
    for intent in intents:
        for req_match in _REQUIREMENTS_FILE_RE.finditer(intent):
            names |= set(
                _unpinned_from_requirements_file(workspace / req_match.group(1))
            )
    return frozenset(names)


def collect_requirement_editable_intents(workspace: Path, intents: list[str]) -> list[str]:
    """Editable install intents declared inside ``-r`` requirements files."""
    found: list[str] = []
    seen: set[str] = set()
    workspace = workspace.resolve()
    for intent in intents:
        for req_match in _REQUIREMENTS_FILE_RE.finditer(intent):
            for editable in _editable_lines_from_requirements_file(
                workspace / req_match.group(1)
            ):
                if editable not in seen:
                    seen.add(editable)
                    found.append(editable)
    return found

_PIP_OPTION_WITH_VALUE = frozenset(
    {
        "-c",
        "--constraint",
        "-e",
        "--editable",
        "-f",
        "--find-links",
        "-i",
        "--index-url",
        "--extra-index-url",
        "--trusted-host",
        "-r",
        "--requirement",
        "-t",
        "--target",
        "--platform",
        "--python-version",
        "--implementation",
        "--abi",
        "--root",
        "--prefix",
        "--src",
        "--config-settings",
        "--global-option",
        "--no-binary",
        "--only-binary",
    }
)


def collect_unpinned_package_names(intents: list[str]) -> frozenset[str]:
    """Bare distribution names from ``pip install`` intents (bulk or alongside ``-e``)."""
    names: set[str] = set()
    for intent in intents:
        if not _is_pip_install_segment(intent):
            continue
        try:
            tokens = shlex.split(intent)
        except ValueError:
            # Tolerate trailing quote noise from Dockerfile string wrapping.
            try:
                tokens = shlex.split(intent.rstrip('"').rstrip("'"))
            except ValueError:
                continue
        install_at = None
        for idx, tok in enumerate(tokens):
            if tok == "install":
                install_at = idx
                break
        if install_at is None:
            continue
        i = install_at + 1
        while i < len(tokens):
            tok = tokens[i].rstrip('"').rstrip("'")
            if tok.startswith("-"):
                opt = tok.split("=", 1)[0]
                if opt in _PIP_OPTION_WITH_VALUE and "=" not in tok:
                    i += 2
                    continue
                i += 1
                continue
            if tok.startswith(("/", ".", "~")) or tok.endswith((".txt", ".in")):
                i += 1
                continue
            # Strip extras / environment markers / version operators for the name.
            bare = tok.split(";", 1)[0].strip()
            bare = bare.split("[", 1)[0].strip()
            if "@" in bare:
                i += 1
                continue
            name_match = re.match(r"^([A-Za-z0-9][\w.-]*)", bare)
            if not name_match:
                i += 1
                continue
            name = _normalize_package_name(name_match.group(1))
            if name in _SHELL_NOISE_NAMES:
                i += 1
                continue
            rest = bare[len(name_match.group(1)) :].strip()
            # Pinned == versions are handled by collect_pinned_packages.
            if rest.startswith("=="):
                i += 1
                continue
            names.add(name)
            i += 1
    return frozenset(names)


_EDITABLE_TARGET_RE = re.compile(
    r"(?:^|\s)(?:-e|--editable)(?:\s*=\s*|\s+)(\S+)",
)


def _editable_target_paths(segment: str, workspace: Path) -> list[Path]:
    """Local paths targeted by ``pip install -e`` / ``--editable`` in *segment*."""
    paths: list[Path] = []
    for match in _EDITABLE_TARGET_RE.finditer(segment):
        raw = match.group(1).strip().strip("'\"")
        raw = raw.split("[", 1)[0].strip()
        if not raw or raw.startswith(("git+", "http://", "https://", "svn+", "hg+")):
            continue
        if raw.startswith("file:"):
            raw = raw[len("file:") :]
            if raw.startswith("//"):
                raw = raw[2:]
        candidate = Path(raw)
        if not candidate.is_absolute():
            candidate = (workspace / candidate).resolve()
        else:
            candidate = candidate.resolve()
        if candidate.is_file():
            candidate = candidate.parent
        if candidate.is_dir():
            paths.append(candidate)
    return paths


def _read_distribution_name(project_root: Path) -> str | None:
    """Return the packaging distribution name declared at *project_root*, if any."""
    pyproject = project_root / "pyproject.toml"
    if pyproject.is_file():
        try:
            raw = tomllib.loads(pyproject.read_text(encoding="utf-8"))
        except (OSError, tomllib.TOMLDecodeError, TypeError):
            raw = {}
        project = raw.get("project") if isinstance(raw, dict) else None
        if isinstance(project, dict):
            name = project.get("name")
            if isinstance(name, str) and name.strip():
                return _normalize_package_name(name)
        tool = raw.get("tool") if isinstance(raw, dict) else None
        if isinstance(tool, dict):
            poetry = tool.get("poetry")
            if isinstance(poetry, dict):
                name = poetry.get("name")
                if isinstance(name, str) and name.strip():
                    return _normalize_package_name(name)
    setup_cfg = project_root / "setup.cfg"
    if setup_cfg.is_file():
        try:
            text = setup_cfg.read_text(encoding="utf-8")
        except OSError:
            text = ""
        match = re.search(
            r"(?m)^\s*name\s*=\s*([A-Za-z0-9][\w.-]*)\s*$",
            text,
        )
        if match:
            return _normalize_package_name(match.group(1))
    for pattern in (
        r"""(?m)^\s*name\s*=\s*['"]([^'"]+)['"]""",
        r"""(?m)^\s*NAME\s*=\s*['"]([^'"]+)['"]""",
    ):
        setup_py = project_root / "setup.py"
        if not setup_py.is_file():
            break
        try:
            text = setup_py.read_text(encoding="utf-8")
        except OSError:
            break
        match = re.search(pattern, text)
        if match:
            return _normalize_package_name(match.group(1))
    return None


def _top_level_txt_roots(project_root: Path) -> set[str]:
    """Import roots listed in egg-info / dist-info ``top_level.txt`` files."""
    roots: set[str] = set()
    for path in project_root.glob("*.egg-info/top_level.txt"):
        try:
            text = path.read_text(encoding="utf-8")
        except OSError:
            continue
        for line in text.splitlines():
            name = line.strip()
            if name:
                roots.add(name.split(".", 1)[0])
    for path in project_root.glob("*.dist-info/top_level.txt"):
        try:
            text = path.read_text(encoding="utf-8")
        except OSError:
            continue
        for line in text.splitlines():
            name = line.strip()
            if name:
                roots.add(name.split(".", 1)[0])
    return roots


def _filesystem_package_roots(project_root: Path) -> set[str]:
    """Heuristic import roots from common src-/flat- layouts under *project_root*."""
    roots: set[str] = set()
    skip = {
        "tests",
        "test",
        "docs",
        "doc",
        "examples",
        "example",
        "scripts",
        "benchmarks",
        "benchmark",
        "build",
        "dist",
        "requirements",
        "venv",
        ".venv",
        "node_modules",
        "__pycache__",
    }
    src = project_root / "src"
    search_roots = [src] if src.is_dir() else [project_root]
    for base in search_roots:
        try:
            entries = list(base.iterdir())
        except OSError:
            continue
        for entry in entries:
            if not entry.is_dir() or entry.name.startswith(".") or entry.name in skip:
                continue
            if (entry / "__init__.py").is_file() or (entry / "__init__.pyi").is_file():
                roots.add(entry.name)
    return roots


def import_roots_provided_by_project(project_root: Path) -> set[str]:
    """Import roots satisfied by installing the project at *project_root* editable."""
    roots = set(_top_level_txt_roots(project_root))
    roots |= _filesystem_package_roots(project_root)
    dist_name = _read_distribution_name(project_root)
    if dist_name:
        roots.add(dist_name.replace("-", "_"))
        # Harbor / DeclaredDeps keys often use the distribution spelling.
        roots.add(dist_name)
    return {r for r in roots if r}


def dockerfile_uses_poetry_install(dockerfile_text: str) -> bool:
    """True when a Dockerfile RUN installs the project via Poetry."""
    return bool(re.search(r"\bpoetry\s+install\b", dockerfile_text, re.IGNORECASE))


def pythonpath_entries_from_dockerfile(
    dockerfile_text: str,
    workspace: Path,
) -> list[Path]:
    """Resolve ``ENV PYTHONPATH=…`` entries that fall under *workspace*."""
    workspace = workspace.resolve()
    paths: list[Path] = []
    for match in re.finditer(
        r"(?im)^\s*ENV\s+PYTHONPATH=(\S+)",
        dockerfile_text,
    ):
        raw = match.group(1).strip().strip("'\"")
        for part in raw.split(":"):
            part = part.strip()
            if not part:
                continue
            if part in ("/app", "."):
                paths.append(workspace)
                continue
            if part.startswith("/app/"):
                rel = part[len("/app/") :]
                candidate = (workspace / rel).resolve()
            else:
                candidate = Path(part)
                if not candidate.is_absolute():
                    candidate = (workspace / candidate).resolve()
            if candidate == workspace or workspace in candidate.parents or candidate.is_dir():
                paths.append(candidate)
    return paths


def workspace_mount_provided_import_roots(
    workspace: Path,
    dockerfile: Path | None = None,
) -> set[str]:
    """Import roots satisfied by the mounted workspace (editable, PYTHONPATH, or layout).

    Harbor grades run with ``cwd=/app``. Flat layouts are importable via ``sys.path``;
    ``ENV PYTHONPATH`` and Poetry installs also expose the project without a separate
    DeclaredDeps pin. Always include filesystem/distribution roots for the workspace
    itself so package-under-test imports are not marked unmapped.
    """
    workspace = workspace.resolve()
    provided = import_roots_provided_by_project(workspace)
    if dockerfile is None or not dockerfile.is_file():
        return provided
    text = dockerfile.read_text(encoding="utf-8")
    for path in pythonpath_entries_from_dockerfile(text, workspace):
        provided |= import_roots_provided_by_project(path)
        # src-layout roots when PYTHONPATH points at src/
        provided |= _filesystem_package_roots(
            path if path.name != "src" else path.parent
        )
        if path.name == "src" or (path / "src").is_dir():
            provided |= _filesystem_package_roots(
                path if path.name == "src" else path / "src"
            )
    return provided


def editable_provided_import_roots(
    workspace: Path,
    editable_segments: tuple[str, ...],
    dockerfile: Path | None = None,
) -> set[str]:
    """Union of import roots from editable installs plus the mounted workspace project."""
    provided: set[str] = set()
    workspace = workspace.resolve()
    for segment in editable_segments:
        for path in _editable_target_paths(segment, workspace):
            provided |= import_roots_provided_by_project(path)
    provided |= workspace_mount_provided_import_roots(workspace, dockerfile)
    return provided


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


def _precommit_pin_from_workspace(workspace: Path) -> str | None:
    """Return a pinned ``pre-commit`` version declared by the workspace, if any."""
    workspace = workspace.resolve()
    candidates: list[Path] = []
    req_dir = workspace / "requirements"
    if req_dir.is_dir():
        candidates.extend(sorted(req_dir.rglob("*.txt")))
    for name in ("requirements.txt", "dev-requirements.txt", "lint-requirements.txt"):
        path = workspace / name
        if path.is_file():
            candidates.append(path)
    seen: set[Path] = set()
    for path in candidates:
        resolved = path.resolve()
        if resolved in seen:
            continue
        seen.add(resolved)
        pin = _pins_from_requirements_file(path).get("pre-commit")
        if pin:
            return pin
    pyproject = workspace / "pyproject.toml"
    if pyproject.is_file():
        raw = tomllib.loads(pyproject.read_text(encoding="utf-8"))
        dep_lists: list[list[str]] = []
        optional = raw.get("project", {}).get("optional-dependencies")
        if isinstance(optional, dict):
            dep_lists.extend(
                deps for deps in optional.values() if isinstance(deps, list)
            )
        groups = raw.get("dependency-groups")
        if isinstance(groups, dict):
            dep_lists.extend(deps for deps in groups.values() if isinstance(deps, list))
        for deps in dep_lists:
            for dep in deps:
                if not isinstance(dep, str):
                    continue
                parsed = _parse_dependency_spec(dep.split(";", 1)[0].strip())
                if parsed and parsed[0] == "pre-commit" and parsed[1].startswith("=="):
                    return parsed[1][2:]
    lockfile = workspace / "uv.lock"
    if lockfile.is_file():
        for match in _UV_LOCK_PACKAGE_RE.finditer(lockfile.read_text(encoding="utf-8")):
            if match.group(1).lower() == "pre-commit":
                return match.group(2)
    return None


def pins_for_task(
    dockerfile: Path | None,
    workspace: Path | None = None,
) -> dict[str, str]:
    """Pinned packages for a task workspace (Modal image build / cache bust)."""
    if dockerfile is None or not dockerfile.is_file() or workspace is None:
        return {}
    intents = collect_pip_install_intents(dockerfile.read_text(encoding="utf-8"))
    return collect_pinned_packages(workspace.resolve(), intents)


_DEP_SPEC_RE = re.compile(
    r"^([A-Za-z0-9][\w.-]*)(\[[^\]]+\])?\s*([^;]*?)(?:\s*;\s*.*)?$"
)
_UV_LOCK_PACKAGE_RE = re.compile(
    r'^\[\[package\]\]\s*\nname = "([^"]+)"\s*\nversion = "([^"]+)"',
    re.MULTILINE,
)


@dataclass(frozen=True)
class DeclaredDeps:
    """Canonical Python dependency declarations for one task workspace."""

    bulk_pins: dict[str, str]
    constraints: dict[str, str]
    editable_segments: tuple[str, ...]
    lockfile_pins: dict[str, str]
    unpinned_names: frozenset[str] = frozenset()

    def package_names(self) -> set[str]:
        keys = (
            set(self.bulk_pins)
            | set(self.constraints)
            | set(self.lockfile_pins)
            | set(self.unpinned_names)
        )
        return {name.lower() for name in keys}

    def effective_spec(self, name: str) -> str | None:
        key = name.lower()
        if key in self.bulk_pins:
            return f"=={self.bulk_pins[key]}"
        constraint = self.constraints.get(key)
        if constraint is not None:
            return constraint
        if key in self.lockfile_pins:
            return f"=={self.lockfile_pins[key]}"
        return None

    def pip_install_spec(self, name: str) -> str | None:
        """Return a pip package argument for *name*, or None when not declared."""
        key = name.lower()
        spec = self.effective_spec(key)
        if spec is None:
            if key in self.unpinned_names:
                return key
            return None
        if not spec:
            return key
        # Extras-first remainders (``[standard]>=0.0.8``) must not gain a fake ``==``.
        if spec.startswith("[") or spec.startswith(
            ("==", ">=", "<=", "!=", "~=", ">", "<")
        ):
            return f"{key}{spec}"
        return f"{key}=={spec}"


def _normalize_package_name(name: str) -> str:
    return name.lower().replace("_", "-")


def _strip_requirement_comment(line: str) -> str:
    """Strip unquoted ``# …`` tails from requirements lines (OpenStack-style license tags)."""
    in_quote: str | None = None
    for i, ch in enumerate(line):
        if in_quote is not None:
            if ch == in_quote:
                in_quote = None
            continue
        if ch in ("'", '"'):
            in_quote = ch
            continue
        if ch == "#":
            return line[:i].rstrip()
    return line.rstrip()


def _parse_dependency_spec(raw: str) -> tuple[str, str] | None:
    line = _strip_requirement_comment(raw.strip())
    if not line or line.startswith("#"):
        return None
    match = _DEP_SPEC_RE.match(line)
    if not match:
        return None
    name = _normalize_package_name(match.group(1))
    extras = match.group(2) or ""
    ver = (match.group(3) or "").strip()
    return name, f"{extras}{ver}"


def _split_pyproject_dependency(raw: str) -> tuple[str, str, str | None] | None:
    """Return ``(name, spec, marker)`` from one PEP 508 dependency string."""
    line = _strip_requirement_comment(raw.strip())
    if not line or line.startswith("#"):
        return None
    marker: str | None = None
    dep_part = line
    if ";" in line:
        dep_part, marker_text = line.split(";", 1)
        marker = marker_text.strip() or None
    parsed = _parse_dependency_spec(dep_part)
    if not parsed:
        return None
    return parsed[0], parsed[1], marker


def _version_tuple(version: str) -> tuple[int, ...]:
    parts: list[int] = []
    for piece in version.split("."):
        digits = re.match(r"(\d+)", piece)
        if not digits:
            break
        parts.append(int(digits.group(1)))
    return tuple(parts)


def _compare_version_tuple(left: tuple[int, ...], op: str, right: tuple[int, ...]) -> bool:
    width = max(len(left), len(right))
    left_padded = left + (0,) * (width - len(left))
    right_padded = right + (0,) * (width - len(right))
    if op == "<":
        return left_padded < right_padded
    if op == "<=":
        return left_padded <= right_padded
    if op == ">":
        return left_padded > right_padded
    if op == ">=":
        return left_padded >= right_padded
    if op == "==":
        return left_padded == right_padded
    if op == "!=":
        return left_padded != right_padded
    return True


def _environment_marker_applies(marker: str | None) -> bool:
    """True when a PEP 508 environment marker matches the current interpreter."""
    if not marker:
        return True
    try:
        from packaging.markers import Marker

        return Marker(marker).evaluate()
    except Exception:
        pass
    match = re.match(
        r"^python_version\s*(<|<=|>=|>|==|!=)\s*['\"]([\d.]+)['\"]\s*$",
        marker.strip(),
    )
    if match:
        op, bound = match.group(1), _version_tuple(match.group(2))
        current = sys.version_info[: max(len(bound), 2)]
        return _compare_version_tuple(current, op, bound)
    return False


def _read_pyproject_dependencies(
    pyproject: Path,
) -> tuple[dict[str, str], frozenset[str]]:
    """Return ``(versioned_constraints, bare_unpinned_names)`` from ``[project]`` deps."""
    if not pyproject.is_file():
        return {}, frozenset()
    raw = tomllib.loads(pyproject.read_text(encoding="utf-8"))
    constraints: dict[str, str] = {}
    bare: set[str] = set()
    project = raw.get("project") or {}
    for dep in project.get("dependencies") or []:
        if not isinstance(dep, str):
            continue
        parsed = _split_pyproject_dependency(dep)
        if not parsed:
            continue
        name, spec, marker = parsed
        if not _environment_marker_applies(marker):
            continue
        if not spec:
            bare.add(name)
        else:
            constraints[name] = spec
    return constraints, frozenset(bare)


def _read_uv_lock_pins(lock_path: Path, names: set[str]) -> dict[str, str]:
    if not lock_path.is_file() or not names:
        return {}
    text = lock_path.read_text(encoding="utf-8")
    pins: dict[str, str] = {}
    for match in _UV_LOCK_PACKAGE_RE.finditer(text):
        pkg = _normalize_package_name(match.group(1))
        if pkg in names:
            pins[pkg] = match.group(2)
    return pins


def _editable_segments_from_dockerfile(dockerfile_text: str) -> tuple[str, ...]:
    segments: list[str] = []
    for run in parse_dockerfile_run_commands(dockerfile_text):
        if not should_replay_run_command(run):
            continue
        for segment in _split_shell_segments(run):
            if _is_editable_pip_segment(segment):
                segments.append(segment)
    return tuple(segments)


def _extras_names_from_editable_target(target: str) -> list[str]:
    """Return extras names from an editable target like ``.[test,dev]`` or ``pkg[extra]``."""
    match = re.search(r"\[([^\]]+)\]", target)
    if not match:
        return []
    return [part.strip() for part in match.group(1).split(",") if part.strip()]


def _optional_dependency_specs_from_pyproject(
    pyproject: Path,
    extras: list[str],
) -> tuple[dict[str, str], frozenset[str]]:
    """Return ``(constraints, bare_names)`` from ``[project.optional-dependencies]`` extras."""
    if not extras or not pyproject.is_file():
        return {}, frozenset()
    try:
        raw = tomllib.loads(pyproject.read_text(encoding="utf-8"))
    except (OSError, tomllib.TOMLDecodeError, TypeError):
        return {}, frozenset()
    optional = (raw.get("project") or {}).get("optional-dependencies") or {}
    if not isinstance(optional, dict):
        return {}, frozenset()
    constraints: dict[str, str] = {}
    bare: set[str] = set()
    for extra in extras:
        deps = optional.get(extra) or optional.get(extra.replace("-", "_")) or []
        if not isinstance(deps, list):
            continue
        for dep in deps:
            if not isinstance(dep, str):
                continue
            parsed = _split_pyproject_dependency(dep)
            if not parsed:
                continue
            name, spec, marker = parsed
            if not _environment_marker_applies(marker):
                continue
            if not spec:
                bare.add(name)
            else:
                constraints[name] = spec
    return constraints, frozenset(bare)


def _poetry_dependency_names(
    pyproject: Path,
    *,
    include_groups: tuple[str, ...] = ("dev",),
    include_optional: bool = False,
) -> frozenset[str]:
    """Distribution names declared under Poetry dependencies / groups / extras tables."""
    if not pyproject.is_file():
        return frozenset()
    try:
        raw = tomllib.loads(pyproject.read_text(encoding="utf-8"))
    except (OSError, tomllib.TOMLDecodeError, TypeError):
        return frozenset()
    tool = raw.get("tool") or {}
    poetry = tool.get("poetry") if isinstance(tool, dict) else None
    if not isinstance(poetry, dict):
        return frozenset()
    names: set[str] = set()

    def _absorb(section: object, *, optional_ok: bool) -> None:
        if not isinstance(section, dict):
            return
        for key, value in section.items():
            if not isinstance(key, str):
                continue
            if key.lower() == "python":
                continue
            if isinstance(value, dict) and value.get("optional") and not optional_ok:
                continue
            names.add(_normalize_package_name(key))

    _absorb(poetry.get("dependencies"), optional_ok=include_optional)
    group = poetry.get("group")
    if isinstance(group, dict):
        for group_name in include_groups:
            block = group.get(group_name)
            if isinstance(block, dict):
                _absorb(block.get("dependencies"), optional_ok=True)
    _absorb(poetry.get("dev-dependencies"), optional_ok=True)
    return frozenset(names)


def _poetry_extra_package_names(pyproject: Path, extras: list[str]) -> frozenset[str]:
    """Package names listed in Poetry ``extras.<name> = [...]`` for requested extras."""
    if not extras or not pyproject.is_file():
        return frozenset()
    try:
        raw = tomllib.loads(pyproject.read_text(encoding="utf-8"))
    except (OSError, tomllib.TOMLDecodeError, TypeError):
        return frozenset()
    poetry = ((raw.get("tool") or {}).get("poetry") or {})
    if not isinstance(poetry, dict):
        return frozenset()
    extra_table = poetry.get("extras")
    if not isinstance(extra_table, dict):
        return frozenset()
    names: set[str] = set()
    for extra in extras:
        listed = extra_table.get(extra) or extra_table.get(extra.replace("_", "-"))
        if not isinstance(listed, list):
            continue
        for item in listed:
            if isinstance(item, str) and item.strip():
                names.add(_normalize_package_name(item.strip()))
    return frozenset(names)


_SETUP_REQ_STRING_RE = re.compile(
    r"""['"]([A-Za-z0-9][\w.-]*(?:\[[^\]]+\])?(?:\s*(?:==|>=|<=|!=|~=|<|>)[^'"]*)?)['"]"""
)
_SETUP_EXTRAS_REQUIRE_KEYS_RE = re.compile(
    r"""extras_require\s*=\s*\{(.*)\}\s*,""",
    re.DOTALL,
)
_SETUP_DICT_KEY_RE = re.compile(r"""['"]([A-Za-z0-9][\w.-]*)['"]\s*:""")


def _extras_require_keys_from_setup_py(setup_py: Path) -> frozenset[str]:
    """Return ``extras_require`` dict keys (setuptools extra names, not packages)."""
    if not setup_py.is_file():
        return frozenset()
    try:
        text = setup_py.read_text(encoding="utf-8")
    except OSError:
        return frozenset()
    match = _SETUP_EXTRAS_REQUIRE_KEYS_RE.search(text)
    if not match:
        return frozenset()
    return frozenset(
        _normalize_package_name(key)
        for key in _SETUP_DICT_KEY_RE.findall(match.group(1))
    )


def _requirement_files_for_setuptools_extra(workspace: Path, extra: str) -> list[Path]:
    """Conventional requirement-file locations for a setuptools extra name."""
    candidates = (
        workspace / "requirements" / "extras" / f"{extra}.txt",
        workspace / "requirements" / "extras" / extra,
        workspace / "requirements" / f"{extra}.txt",
        workspace / "requirements" / extra,
    )
    return [path for path in candidates if path.is_file()]


def _specs_from_setuptools_extra_files(
    workspace: Path,
    extras: list[str],
) -> tuple[dict[str, str], dict[str, str], frozenset[str]]:
    """Return ``(pins, constraints, bare_names)`` from requirements files for *extras*.

    Celery/Kombu-style projects map ``extras_require`` values to
    ``requirements/extras/<name>.txt`` instead of inline PEP 508 strings. Prefer
    those files over scraping setup.py string literals (which otherwise picks up
    the extra *keys* as fake PyPI names).
    """
    pins: dict[str, str] = {}
    constraints: dict[str, str] = {}
    bare: set[str] = set()
    for extra in extras:
        for req_path in _requirement_files_for_setuptools_extra(workspace, extra):
            pins.update(_pins_from_requirements_file(req_path))
            constraints.update(_constraints_from_requirements_file(req_path))
            bare |= set(_unpinned_from_requirements_file(req_path))
    return pins, constraints, frozenset(bare)


def _requirement_names_from_setup_py(setup_py: Path) -> frozenset[str]:
    """Best-effort package names from quoted requirement strings in ``setup.py``.

    Accepts versioned PEP 508 strings anywhere, and bare names only when the whole
    line is a list item (``\"aiofiles\",``). That drops ``name='pkg'``, ``hasattr``
    string args, and other metadata noise while keeping inline extras bodies (gql).
    """
    if not setup_py.is_file():
        return frozenset()
    try:
        text = setup_py.read_text(encoding="utf-8")
    except OSError:
        return frozenset()
    extra_keys = _extras_require_keys_from_setup_py(setup_py)
    names: set[str] = set()
    for match in _SETUP_REQ_STRING_RE.finditer(text):
        raw = match.group(1).split(";", 1)[0].strip()
        bare = raw.split("[", 1)[0].strip()
        name_match = re.match(r"^([A-Za-z0-9][\w.-]*)", bare)
        if not name_match:
            continue
        name = _normalize_package_name(name_match.group(1))
        if not _is_plausible_distribution_name(name) or name in {"gql", "returns"}:
            continue
        # Extra *keys* are not installable distributions (e.g. kombu ``msgpack``).
        if name in extra_keys:
            continue
        has_version = any(
            op in raw for op in ("==", ">=", "<=", "!=", "~=", ">", "<")
        )
        if not has_version:
            line_start = text.rfind("\n", 0, match.start()) + 1
            line_end = text.find("\n", match.end())
            line = text[line_start : line_end if line_end != -1 else None].strip()
            # Bare requirement list items only: ``"aiofiles",``
            if not re.fullmatch(r"""['"][^'"]+['"]\s*,?""", line):
                continue
        # Skip obvious non-requirement string literals.
        if name_match.group(1)[0].isupper() and not has_version:
            continue
        names.add(name)
    return frozenset(names)


def declared_python_dependencies(
    workspace: Path,
    dockerfile: Path | None = None,
) -> DeclaredDeps:
    """Collect declared Python deps from Dockerfile pins, pyproject, and uv.lock."""
    workspace = workspace.resolve()
    dockerfile_text = dockerfile.read_text(encoding="utf-8") if dockerfile and dockerfile.is_file() else ""
    intents = collect_pip_install_intents(dockerfile_text) if dockerfile_text else []
    req_editables = collect_requirement_editable_intents(workspace, intents) if intents else []
    bulk_pins = collect_pinned_packages(workspace, intents) if intents else {}
    constraints, pyproject_bare = _read_pyproject_dependencies(workspace / "pyproject.toml")
    req_constraints = collect_requirement_constraints(workspace, intents) if intents else {}
    for name, spec in req_constraints.items():
        constraints.setdefault(name, spec)
    for key in bulk_pins:
        constraints.pop(key, None)
    unpinned = collect_unpinned_package_names(intents) if intents else frozenset()
    unpinned |= collect_requirement_unpinned_names(workspace, intents) if intents else frozenset()
    unpinned |= pyproject_bare
    # Expand extras referenced by editable installs (Dockerfile or requirements -e).
    editable_seed = list(_editable_segments_from_dockerfile(dockerfile_text)) if dockerfile_text else []
    editable_seed.extend(req_editables)
    extras_requested: list[str] = []
    for segment in editable_seed:
        for match in _EDITABLE_TARGET_RE.finditer(segment):
            extras = _extras_names_from_editable_target(match.group(1))
            extras_requested.extend(extras)
            extra_constraints, extra_bare = _optional_dependency_specs_from_pyproject(
                workspace / "pyproject.toml",
                extras,
            )
            for name, spec in extra_constraints.items():
                constraints.setdefault(name, spec)
            unpinned |= extra_bare
            unpinned |= _poetry_extra_package_names(workspace / "pyproject.toml", extras)
    if extras_requested:
        # Prefer requirements/extras/<name>.txt (kombu/celery style) so extra *keys*
        # are not mistaken for PyPI names. Still scrape setup.py for inline PEP 508
        # bodies (e.g. gql); that scraper skips extras_require keys.
        extra_pins, extra_constraints, extra_bare = _specs_from_setuptools_extra_files(
            workspace,
            extras_requested,
        )
        for name, ver in extra_pins.items():
            bulk_pins.setdefault(name, ver)
            constraints.pop(name, None)
        for name, spec in extra_constraints.items():
            if name not in bulk_pins:
                constraints.setdefault(name, spec)
        unpinned |= extra_bare
        unpinned |= _requirement_names_from_setup_py(workspace / "setup.py")
    # Celery/Kombu: install_requires often lives in requirements/default.txt via
    # ``reqs('default.txt')``. Editable verifier replay uses ``--no-deps``, so
    # absorb that file when an editable install is present and Dockerfile never
    # ``-r``'d it.
    if editable_seed:
        default_req = workspace / "requirements" / "default.txt"
        if default_req.is_file():
            for name, ver in _pins_from_requirements_file(default_req).items():
                bulk_pins.setdefault(name, ver)
                constraints.pop(name, None)
            for name, spec in _constraints_from_requirements_file(default_req).items():
                if name not in bulk_pins:
                    constraints.setdefault(name, spec)
            unpinned |= _unpinned_from_requirements_file(default_req)
        # Editable replay uses ``--no-deps``, so absorb each editable target's
        # ``[project].dependencies`` (langchain monorepo: ``libs/core`` → pydantic).
        for segment in editable_seed:
            for target in _editable_target_paths(segment, workspace):
                pkg_constraints, pkg_bare = _read_pyproject_dependencies(
                    target / "pyproject.toml"
                )
                for name, spec in pkg_constraints.items():
                    if name not in bulk_pins:
                        constraints.setdefault(name, spec)
                unpinned |= pkg_bare
    # Poetry declares runtime deps even when the Dockerfile uses pip -e / poetry install.
    unpinned |= _poetry_dependency_names(
        workspace / "pyproject.toml",
        include_groups=("dev",) if (
            dockerfile_text and dockerfile_uses_poetry_install(dockerfile_text)
        ) else (),
        include_optional=bool(extras_requested),
    )
    if dockerfile_text and dockerfile_uses_poetry_install(dockerfile_text):
        # Poetry installs the project itself into the env.
        if "pip install -e ." not in editable_seed:
            editable_seed.append("pip install -e .")
    unpinned = frozenset(
        name
        for name in unpinned
        if name not in bulk_pins
        and name not in constraints
        and _is_plausible_distribution_name(name)
    )
    lockfile_pins = _read_uv_lock_pins(
        workspace / "uv.lock",
        {name.lower() for name in constraints} | set(bulk_pins) | set(unpinned),
    )
    # Deduplicate editable segments while preserving order.
    seen_edit: set[str] = set()
    editable_segments: list[str] = []
    for segment in editable_seed:
        if segment not in seen_edit:
            seen_edit.add(segment)
            editable_segments.append(segment)
    # Harbor PYTHONPATH images expose the project without an editable install.
    # Do not synthesize ``pip install -e .`` here: that forces a build (often
    # needing ``[build-system].requires`` like Cython) that Harbor never runs.
    # Verifier grade injects Dockerfile PYTHONPATH via VerifierSpec instead.
    return DeclaredDeps(
        bulk_pins=bulk_pins,
        constraints=constraints,
        editable_segments=tuple(editable_segments),
        lockfile_pins=lockfile_pins,
        unpinned_names=unpinned,
    )


def format_prep_error(
    task_id: str,
    *,
    phase: str,
    package: str | None = None,
    observed: str | None = None,
    expected: str | None = None,
    detail: str | None = None,
    hint: str | None = None,
) -> str:
    """Human-readable short-abort message for dependency prep failures."""
    parts = [f"sandbox {phase} failed ({task_id})"]
    if package:
        parts.append(f": {package}")
    if observed is not None and expected is not None:
        parts.append(f" — observed {observed}, expected {expected}")
    elif detail:
        parts.append(f" — {detail}")
    if hint:
        parts.append(f"; hint: {hint}")
    return "".join(parts)


VERIFIER_VENV_PATH = "/opt/malvin-verifier"
VERIFIER_PYTHON = f"{VERIFIER_VENV_PATH}/bin/python"
VERIFIER_PIP = f"{VERIFIER_VENV_PATH}/bin/pip"


def _verifier_pip(spec: VerifierSpec | None = None, *, venv_path: str | None = None) -> str:
    """Pip binary inside the verifier venv (honors ``spec.venv_path`` overrides)."""
    root = venv_path
    if root is None and spec is not None:
        root = spec.venv_path
    if root is None:
        root = VERIFIER_VENV_PATH
    return f"{root}/bin/pip"


@dataclass(frozen=True)
class PluginPolicy:
    """Grade-subprocess-only pytest plugin policy (never bake into agent image env).

    ``as_env`` sets ``PYTEST_DISABLE_PLUGIN_AUTOLOAD`` and optional ``-p`` allowlist
    tokens. Callers that merge into an existing env must append allowlist tokens to
    any pre-existing ``PYTEST_ADDOPTS`` (see ``verifier_grade_subprocess_env``).
    ``MALVIN_VERIFIER_PLUGIN_ALLOWLIST`` is debug metadata only; pytest does not read it.
    """

    disable_autoload: bool = False
    allowlist: tuple[str, ...] = ()

    def as_env(self) -> dict[str, str]:
        if not self.disable_autoload:
            return {}
        env = {"PYTEST_DISABLE_PLUGIN_AUTOLOAD": "1"}
        if self.allowlist:
            # Re-enable selected plugins after autoload disable (consumed by pytest).
            env["PYTEST_ADDOPTS"] = " ".join(f"-p {name}" for name in self.allowlist)
            env["MALVIN_VERIFIER_PLUGIN_ALLOWLIST"] = ",".join(self.allowlist)
        return env


def _merge_pytest_addopts(existing: str | None, addition: str | None) -> str:
    """Append *addition* tokens to *existing* ``PYTEST_ADDOPTS`` without dropping either."""
    parts = [p for p in ((existing or "").strip(), (addition or "").strip()) if p]
    return " ".join(parts)


@dataclass(frozen=True)
class VerifierSpec:
    """Public + grade-only Harbor verifier dependency discovery result.

    Public fields may appear on the agent image. Grade-only fields (``harbor_imports``,
    closure install specs, plugin policy, unmapped imports) are verifier secrets —
    persist them only in grade-phase / host metadata, never in agent-readable
    ``sandbox_prep`` payloads.
    """

    declared: DeclaredDeps
    public_install_specs: tuple[str, ...]
    editable_segments: tuple[str, ...]
    harbor_imports: tuple[str, ...] = ()
    grade_closure_install_specs: tuple[str, ...] = ()
    unmapped_imports: tuple[str, ...] = ()
    test_sh_body: str | None = None
    plugin_policy: PluginPolicy | None = None
    venv_path: str = VERIFIER_VENV_PATH
    # Absolute host/sandbox paths from Dockerfile ``ENV PYTHONPATH`` (Harbor layout).
    grade_pythonpath: tuple[str, ...] = ()

    def public_view(self) -> dict[str, Any]:
        """Agent-safe summary: no ``test.patch``-derived import or closure fields."""
        return {
            "venv_path": self.venv_path,
            "public_install_specs": list(self.public_install_specs),
            "editable_segments": list(self.editable_segments),
            "declared_packages": sorted(self.declared.package_names()),
        }

    def grade_view(self) -> dict[str, Any]:
        """Host/grade-only view including secret discovery fields."""
        payload = self.public_view()
        payload.update(
            {
                "harbor_imports": list(self.harbor_imports),
                "grade_closure_install_specs": list(self.grade_closure_install_specs),
                "unmapped_imports": list(self.unmapped_imports),
                "grade_pythonpath": list(self.grade_pythonpath),
                "plugin_policy": (
                    {
                        "disable_autoload": self.plugin_policy.disable_autoload,
                        "allowlist": list(self.plugin_policy.allowlist),
                    }
                    if self.plugin_policy
                    else None
                ),
            }
        )
        return payload


def _public_install_specs(declared: DeclaredDeps) -> tuple[str, ...]:
    specs: list[str] = []
    for name in sorted(declared.package_names()):
        spec = declared.pip_install_spec(name)
        if spec:
            specs.append(spec)
    return tuple(specs)


def discover_verifier_spec(
    workspace: Path,
    tests_dir: Path | None = None,
    dockerfile: Path | None = None,
) -> VerifierSpec:
    """Discover public DeclaredDeps and optional grade-only Harbor import closure.

    When ``tests_dir`` is None (agent image path), grade-only fields stay empty so
    ``test.patch`` secrets are never ingested.

    ``grade_closure_install_specs`` lists declared pin specs required by Harbor
    imports (even when those pins are already in ``public_install_specs``). Grade
    prep may reinstall them into ``/opt/malvin-verifier``; agent-image materialize
    never runs those grade-only commands. Unmapped third-party imports are recorded
    for probe handling and are never invented as unpinned PyPI installs. Imports
    satisfied by Dockerfile editable installs or the mounted workspace project are
    not unmapped (editable replay / workspace layout provides them).
    """
    workspace = workspace.resolve()
    declared = declared_python_dependencies(workspace, dockerfile)
    editable_segments = list(declared.editable_segments)
    dockerfile_text = (
        dockerfile.read_text(encoding="utf-8")
        if dockerfile is not None and dockerfile.is_file()
        else ""
    )
    pythonpath_entries = (
        pythonpath_entries_from_dockerfile(dockerfile_text, workspace)
        if dockerfile_text
        else []
    )
    grade_pythonpath = tuple(str(p) for p in pythonpath_entries)
    # Verifier collect runs outside /app; install the package-under-test into the
    # verifier venv when needed. Harbor PYTHONPATH images already expose the
    # project without a build — prefer that over synthesizing ``pip install -e .``
    # (which often needs Cython / other build-system deps Harbor never installs).
    if import_roots_provided_by_project(workspace) and not pythonpath_entries:
        covers_workspace = False
        for segment in editable_segments:
            for path in _editable_target_paths(segment, workspace):
                if path.resolve() == workspace:
                    covers_workspace = True
                    break
            if covers_workspace:
                break
        if not covers_workspace:
            editable_segments.append("pip install --no-deps -e .")
            declared = DeclaredDeps(
                bulk_pins=declared.bulk_pins,
                constraints=declared.constraints,
                editable_segments=tuple(editable_segments),
                lockfile_pins=declared.lockfile_pins,
                unpinned_names=declared.unpinned_names,
            )
    public_specs = _public_install_specs(declared)
    harbor_imports = harbor_imports_from_tests_dir(tests_dir)
    editable_imports = editable_provided_import_roots(
        workspace,
        declared.editable_segments,
        dockerfile=dockerfile,
    )
    editable_imports_normalized = {
        name.replace("-", "_").lower() for name in editable_imports
    } | {name.lower() for name in editable_imports}
    closure: list[str] = []
    unmapped: list[str] = []
    for import_name in harbor_imports:
        import_key = import_name.replace("-", "_").lower()
        if (
            import_name in editable_imports
            or import_key in editable_imports_normalized
            or import_name.lower() in editable_imports_normalized
        ):
            # Satisfied by pip install -e replay into the verifier venv.
            continue
        dist = distribution_name_for_import(import_name)
        spec = declared.pip_install_spec(dist)
        if spec is None:
            # Also try the raw import root (e.g. already hyphenated).
            spec = declared.pip_install_spec(import_name)
        if spec is None:
            unmapped.append(import_name)
            continue
        # Harbor need-set from declared pins (may overlap public specs; grade-only apply).
        if spec not in closure:
            closure.append(spec)
    test_sh = resolve_harbor_test_sh_body(tests_dir)
    return VerifierSpec(
        declared=declared,
        public_install_specs=public_specs,
        editable_segments=declared.editable_segments,
        harbor_imports=harbor_imports,
        grade_closure_install_specs=tuple(closure),
        unmapped_imports=tuple(unmapped),
        test_sh_body=test_sh,
        grade_pythonpath=grade_pythonpath,
    )


def verifier_venv_materialize_public_commands(
    spec: VerifierSpec,
    *,
    workspace: Path | None = None,
) -> list[str]:
    """Create ``/opt/malvin-verifier`` and install **public** DeclaredDeps only."""
    pip_bin = _verifier_pip(spec)
    commands = [
        f"python3 -m venv {shlex.quote(spec.venv_path)}",
        f"{shlex.quote(pip_bin)} install --upgrade pip setuptools wheel",
    ]
    if spec.public_install_specs:
        pkgs = " ".join(shlex.quote(s) for s in spec.public_install_specs)
        commands.append(
            f"{shlex.quote(pip_bin)} install --no-cache-dir {pkgs}"
        )
    if workspace is not None and spec.editable_segments:
        commands.extend(
            verifier_venv_build_system_commands(workspace, spec=spec)
        )
    for segment in spec.editable_segments:
        # Replay editable installs into the verifier venv with --no-deps (pin fidelity).
        rewritten = _rewrite_pip_segment_python(segment, pip_bin)
        if "--no-deps" not in rewritten:
            rewritten += " --no-deps"
        commands.append(rewritten)
    return commands


def _rewrite_pip_segment_python(segment: str, pip_bin: str) -> str:
    """Point a Dockerfile pip segment at *pip_bin*."""
    out = segment.strip()
    replacements = (
        ("python3 -m pip", pip_bin),
        ("python -m pip", pip_bin),
        ("pip3 ", f"{pip_bin} "),
        ("pip ", f"{pip_bin} "),
    )
    for old, new in replacements:
        if old in out:
            return out.replace(old, new, 1)
    if out.startswith(pip_bin):
        return out
    return f"{pip_bin} {out}" if not out.startswith("pip") else out.replace("pip", pip_bin, 1)


def verifier_venv_apply_grade_closure_commands(spec: VerifierSpec) -> list[str]:
    """Install grade-only closure specs into the verifier venv (requires ``/tests``)."""
    if not spec.grade_closure_install_specs:
        return []
    pkgs = " ".join(shlex.quote(s) for s in spec.grade_closure_install_specs)
    pip_bin = _verifier_pip(spec)
    return [f"{shlex.quote(pip_bin)} install --no-cache-dir {pkgs}"]


def verifier_venv_replay_editable_commands(spec: VerifierSpec) -> list[str]:
    """Replay Dockerfile editables into the verifier venv against the live workspace.

    Image-build materialize may have installed editables against a copied ``/app``.
    Runtime remounts replace ``/app``, so grade prep must re-link editables offline.
    """
    pip_bin = _verifier_pip(spec)
    commands: list[str] = []
    for segment in spec.editable_segments:
        rewritten = _rewrite_pip_segment_python(segment, pip_bin)
        if "--no-deps" not in rewritten:
            rewritten += " --no-deps"
        if "--no-build-isolation" not in rewritten:
            rewritten += " --no-build-isolation"
        commands.append(rewritten)
    return commands


def _distribution_name_from_requirement(req: str) -> str:
    """Best-effort PEP 508 name token for dedupe (ignores extras/markers/versions)."""
    token = req.split(";", 1)[0].strip()
    token = token.split("[", 1)[0].strip()
    for sep in ("===", "==", ">=", "<=", "!=", "~=", ">", "<"):
        if sep in token:
            token = token.split(sep, 1)[0].strip()
            break
    return _normalize_package_name(token)


def _editable_offline_seed_specs(
    workspace: Path,
    *,
    dockerfile: Path | None = None,
    editable_segments: tuple[str, ...] | None = None,
) -> list[str]:
    """Packages required in the target env before offline ``--no-build-isolation`` editables.

    Hatchling imports ``editables`` at editable-build time even when it is absent
    from ``[build-system].requires`` (python-statemachine: requires = [\"hatchling\"] only).

    Build backends are collected from the workspace root pyproject and from every local
    path targeted by Dockerfile / verifier ``pip install -e`` segments (langchain-style
    monorepos keep hatchling under ``libs/*/pyproject.toml``, not the repo root).
    """
    pyprojects: list[Path] = [workspace / "pyproject.toml"]
    segments: list[str] = []
    if editable_segments:
        segments.extend(editable_segments)
    elif dockerfile is not None and dockerfile.is_file():
        segments.extend(
            _editable_segments_from_dockerfile(dockerfile.read_text(encoding="utf-8"))
        )
    for segment in segments:
        for target in _editable_target_paths(segment, workspace):
            pyprojects.append(target / "pyproject.toml")
    requires: list[str] = []
    seen_names: set[str] = set()
    for pyproject in pyprojects:
        for req in _read_pyproject_build_system_requires(pyproject):
            name = _distribution_name_from_requirement(req)
            if name in seen_names:
                continue
            seen_names.add(name)
            requires.append(req)
    if "editables" not in seen_names:
        requires.append("editables")
    return requires


def default_pip_editable_seed_command(
    workspace: Path,
    dockerfile: Path | None,
) -> str | None:
    """Seed default ``pip`` with build backends before offline Dockerfile editable replay.

    Image warm installs Hatchling/editables into ``.venv`` via uv, but agent Prep sync
    replays Dockerfile ``pip install -e`` against system/default pip. Without this seed,
    ``--no-build-isolation`` fails with ``ModuleNotFoundError: editables``.
    """
    if dockerfile is None or not dockerfile.is_file():
        return None
    if not workspace_sync_commands_from_dockerfile(dockerfile, offline_editable=True):
        return None
    specs = _editable_offline_seed_specs(workspace, dockerfile=dockerfile)
    if not specs:
        return None
    pkgs = " ".join(shlex.quote(s) for s in specs)
    return f"pip install --no-cache-dir {pkgs}"


def verifier_venv_build_system_commands(
    workspace: Path,
    *,
    venv_path: str | None = None,
    spec: VerifierSpec | None = None,
) -> list[str]:
    """Install build backends (+ ``editables``) into the verifier venv before editable replay."""
    segments = spec.editable_segments if spec is not None else None
    requires = _editable_offline_seed_specs(workspace, editable_segments=segments)
    if not requires:
        return []
    pip_bin = _verifier_pip(spec, venv_path=venv_path)
    pkgs = " ".join(shlex.quote(r) for r in requires)
    return [f"{shlex.quote(pip_bin)} install --no-cache-dir {pkgs}"]


def _plugin_closure_probe_python() -> str:
    return (
        "import importlib.metadata, sys\n"
        "ok, errors = [], []\n"
        "eps = importlib.metadata.entry_points()\n"
        "group = eps.select(group='pytest11') if hasattr(eps, 'select') else eps.get('pytest11', [])\n"
        "for ep in group:\n"
        "    try:\n"
        "        ep.load()\n"
        "        ok.append(ep.name)\n"
        "    except Exception as exc:\n"
        "        errors.append(f'{ep.name}: {type(exc).__name__}: {exc}')\n"
        "print('PLUGIN_OK:' + ','.join(ok))\n"
        "if errors:\n"
        "    print('PLUGIN_CONFLICTS:' + '; '.join(errors))\n"
        "    sys.exit(2)\n"
        "sys.exit(0)\n"
    )


def _parse_plugin_probe_names(stdout: str, *, prefix: str) -> tuple[str, ...]:
    """Parse ``PLUGIN_OK:a,b`` or conflict names from ``PLUGIN_CONFLICTS:name: Err; ...``."""
    for line in (stdout or "").splitlines():
        line = line.strip()
        if not line.startswith(prefix):
            continue
        payload = line[len(prefix) :].strip()
        if not payload:
            return ()
        if prefix.startswith("PLUGIN_CONFLICTS"):
            names: list[str] = []
            for part in payload.split(";"):
                part = part.strip()
                if not part:
                    continue
                names.append(part.split(":", 1)[0].strip())
            return tuple(n for n in names if n)
        return tuple(n for n in payload.split(",") if n.strip())
    return ()


def _materialize_harbor_probe_tree(tests_dir: Path | None, dest: Path) -> tuple[str, ...]:
    """Write Harbor hidden ``.py`` sources from ``test.patch`` hunks into *dest*.

    Prefer parse-added-hunks (plan Q6). *dest* must be outside ``/app`` so agent
    remounts never see the files. Returns relative paths written.
    """
    if tests_dir is None:
        return ()
    written: list[str] = []
    patch_path = tests_dir / "test.patch"
    for rel, body in added_python_sources_from_patch(patch_path).items():
        target = dest / rel
        target.parent.mkdir(parents=True, exist_ok=True)
        text = body if body.endswith("\n") else body + "\n"
        target.write_text(text, encoding="utf-8")
        written.append(rel)
    return tuple(written)


_MISSING_MODULE_RE = re.compile(
    r"No module named ['\"]([^'\"]+)['\"]"
)
_CANNOT_IMPORT_FROM_RE = re.compile(
    r"cannot import name ['\"][^'\"]+['\"] from ['\"]([^'\"]+)['\"]"
)


def _missing_module_from_import_error(err: str) -> str | None:
    """Best-effort module path extracted from a pytest collect ImportError."""
    match = _MISSING_MODULE_RE.search(err)
    if match:
        return match.group(1)
    match = _CANNOT_IMPORT_FROM_RE.search(err)
    if match:
        return match.group(1)
    # Truncated traces often end mid-statement: ``from pwnlib.tubes.mux import``.
    match = re.search(
        r"^\s*from\s+([A-Za-z_][\w.]*)\s+import\b",
        err,
        re.MULTILINE,
    )
    if match:
        return match.group(1)
    # Detail truncation may cut before ``import``: ``from pwnlib.tubes.mux``.
    match = re.search(r"^\s*from\s+([A-Za-z_][\w.]*)\s*$", err, re.MULTILINE)
    if match:
        return match.group(1)
    match = re.search(r"^\s*import\s+([A-Za-z_][\w.]*)\b", err, re.MULTILINE)
    if match:
        return match.group(1)
    return None


def collect_import_error_is_editable_feature_gap(
    err: str,
    provided_roots: set[str],
) -> bool:
    """True when collect ImportError is a missing workspace submodule (pre-solution).

    Top-level missing packages (``No module named 'pwnlib'``) remain prep failures
    when detected explicitly. Missing feature submodules and truncated
    ``from <editable_root>...`` traces soft-succeed after a prior top-level import
    probe has already confirmed the editable root is importable.

    Explicit ``No module named '<third_party>'`` always fails closed, even when the
    traceback also shows ``from <editable_root>`` frames (e.g. pwntools → socks).
    """
    if not provided_roots:
        return False
    roots = {r.replace("-", "_").lower() for r in provided_roots} | {
        r.lower() for r in provided_roots
    }
    # Any explicit missing module outside editable roots is a real dep gap.
    for match in _MISSING_MODULE_RE.finditer(err):
        missing_mod = match.group(1)
        missing_root = missing_mod.split(".", 1)[0].replace("-", "_").lower()
        if missing_root not in roots:
            return False
        # Explicit top-level miss of an editable root ⇒ install failed.
        if "." not in missing_mod:
            return False
    missing = _missing_module_from_import_error(err)
    if missing is not None:
        parts = [p for p in missing.split(".") if p]
        if parts:
            root = parts[0].replace("-", "_").lower()
            if root in roots and len(parts) > 1:
                return True
    # Truncated pytest traces: ``from pwnlib`` cut before submodule / ModuleNotFound.
    # Only soft-succeed when no third-party No module named was found above.
    for root in roots:
        if re.search(rf"(?m)^\s*from\s+{re.escape(root)}\.", err):
            return True
        if re.search(rf"(?m)^\s*from\s+{re.escape(root)}\s+import\b", err):
            return True
        if re.search(rf"(?m)^\s*from\s+{re.escape(root)}\s*$", err):
            return True
    return False


def _probe_editable_roots_importable(
    python_bin: str,
    provided_roots: set[str],
    *,
    workspace: Path,
    harbor_imports: tuple[str, ...] = (),
    env: dict[str, str] | None = None,
) -> str | None:
    """Return an error detail when Harbor-needed editable roots cannot be imported.

    Only probes import roots that Harbor tests actually import and that editable
    discovery claims to provide (e.g. ``pwnlib``, not the ``pwntools`` dist name).

    Import spelling comes from Harbor AST (``IPython``), not from the distribution
    name (``ipython``): ``import_roots_provided_by_project`` may list both, and
    case folding would otherwise probe the wrong module.
    """
    harbor_original: dict[str, str] = {}
    for raw in harbor_imports:
        key = raw.replace("-", "_").lower()
        # Prefer mixed-case AST spellings when duplicates appear.
        prev = harbor_original.get(key)
        if prev is None or (prev.lower() == prev and raw.lower() != raw):
            harbor_original[key] = raw
    harbor = set(harbor_original)
    provided_norm = {
        r.replace("-", "_").lower() for r in provided_roots if "-" not in r
    }
    candidates = sorted(
        {
            harbor_original[h]
            for h in harbor
            if h in provided_norm and harbor_original[h].isidentifier()
        }
    )
    if not candidates:
        return None
    script = (
        "import importlib, sys\n"
        f"roots = {candidates!r}\n"
        "errors = []\n"
        "for r in roots:\n"
        "    try:\n"
        "        importlib.import_module(r)\n"
        "    except Exception as exc:\n"
        "        errors.append(f'{r}: {type(exc).__name__}: {exc}')\n"
        "if errors:\n"
        "    print('; '.join(errors))\n"
        "    sys.exit(1)\n"
        "sys.exit(0)\n"
    )
    proc = subprocess.run(
        [python_bin, "-c", script],
        cwd=str(workspace),
        text=True,
        capture_output=True,
        check=False,
        env=env,
    )
    if proc.returncode == 0:
        return None
    missing = (proc.stdout or proc.stderr or "").strip() or "unknown"
    return f"editable import roots not importable in verifier venv: {missing}"


def probe_verifier_env(
    spec: VerifierSpec,
    *,
    workspace: Path,
    task_id: str = "unknown",
    dry_run: bool = False,
    run_collect: bool = True,
    tests_dir: Path | None = None,
) -> tuple[bool, str | None, PluginPolicy | None]:
    """Probe verifier venv collect/import + pytest plugin closure.

    Returns ``(ok, error_message, plugin_policy)``. On plugin conflicts with declared
    pins, returns a grade-subprocess-only ``PluginPolicy`` that disables autoload.
    Does **not** run full Harbor ``test.sh`` (avoids reward / patch side effects).

    When ``tests_dir`` is set, materializes ``test.patch`` Python hunks into a temp
    tree outside ``/app`` before collect-only so Adaptix-class ImportErrors are not
    masked by pre-apply missing paths.
    """
    python_bin = f"{spec.venv_path}/bin/python"
    if dry_run:
        return True, None, None
    if not Path(python_bin).is_file():
        return (
            False,
            format_prep_error(
                task_id,
                phase="verifier prep",
                detail=(
                    f"verifier venv missing at {spec.venv_path} "
                    "(no system-Python fallback)"
                ),
            ),
            None,
        )

    if spec.unmapped_imports and not run_collect:
        # Q7: never invent unpinned PyPI installs; abort rather than silent drop
        # when we cannot empirically verify via collect-only.
        return (
            False,
            format_prep_error(
                task_id,
                phase="verifier prep",
                detail=(
                    "unmapped Harbor imports (no DeclaredDeps pin): "
                    + ", ".join(spec.unmapped_imports)
                ),
            ),
            None,
        )

    policy: PluginPolicy | None = None
    plugin_cmd = [python_bin, "-c", _plugin_closure_probe_python()]
    plugin_proc = subprocess.run(
        plugin_cmd,
        cwd=str(workspace),
        text=True,
        capture_output=True,
        check=False,
    )
    if plugin_proc.returncode == 2:
        detail = (plugin_proc.stdout or plugin_proc.stderr or "").strip()
        # Prefer disabling conflicting plugins for Harbor grade over upgrading pins.
        # Re-enable plugins that loaded cleanly via allowlist (-p name) after autoload off.
        allow = _parse_plugin_probe_names(plugin_proc.stdout or "", prefix="PLUGIN_OK:")
        policy = PluginPolicy(disable_autoload=True, allowlist=allow)
        # Re-check collect with policy applied below.
    elif plugin_proc.returncode != 0:
        detail = (plugin_proc.stderr or plugin_proc.stdout or "plugin probe failed").strip()
        return (
            False,
            format_prep_error(task_id, phase="verifier prep", detail=detail),
            None,
        )

    if run_collect:
        provided = editable_provided_import_roots(
            workspace,
            spec.editable_segments,
            dockerfile=None,
        )
        # Include PYTHONPATH roots claimed by Harbor Dockerfile layout.
        if spec.grade_pythonpath:
            for entry in spec.grade_pythonpath:
                provided |= import_roots_provided_by_project(Path(entry))
                path = Path(entry)
                if path.name == "src" or (path / "src").is_dir():
                    provided |= _filesystem_package_roots(
                        path if path.name == "src" else path / "src"
                    )
        grade_env = verifier_grade_subprocess_env(spec, plugin_policy=policy)
        editable_err = _probe_editable_roots_importable(
            python_bin,
            provided,
            workspace=workspace,
            harbor_imports=spec.harbor_imports,
            env=grade_env,
        )
        if editable_err:
            return (
                False,
                format_prep_error(
                    task_id, phase="verifier prep", detail=editable_err
                ),
                policy,
            )
        # stestr / custom Harbor runners never call pytest; do not invent a pytest
        # collect probe (bandit test-requirements has no pytest pin).
        if not test_sh_invokes_pytest(spec.test_sh_body):
            return True, None, policy
        collect_cmd = collect_only_pytest_command(python_bin, spec.test_sh_body)
        env = grade_env
        # Materialize hidden tests outside /app (never leave them in the agent workspace).
        with tempfile.TemporaryDirectory(prefix="malvin-verifier-probe-") as tmp:
            probe_root = Path(tmp)
            written = _materialize_harbor_probe_tree(tests_dir, probe_root)
            collect_cwd = probe_root if written else workspace
            collect_proc = subprocess.run(
                ["bash", "-lc", collect_cmd],
                cwd=str(collect_cwd),
                text=True,
                capture_output=True,
                check=False,
                env=env,
            )
            if collect_proc.returncode != 0:
                err = (collect_proc.stderr or collect_proc.stdout or "").strip()
                if "ModuleNotFoundError" in err or "ImportError" in err:
                    if collect_import_error_is_editable_feature_gap(err, provided):
                        # Harbor tests importing not-yet-implemented workspace APIs.
                        return True, None, policy
                    missing = _missing_module_from_import_error(err)
                    if missing is not None:
                        missing_root = missing.split(".", 1)[0]
                        unmapped_norm = {
                            u.replace("-", "_").lower() for u in spec.unmapped_imports
                        } | {u.lower() for u in spec.unmapped_imports}
                        if (
                            missing_root.replace("-", "_").lower() in unmapped_norm
                            or missing_root.lower() in unmapped_norm
                        ):
                            return (
                                False,
                                format_prep_error(
                                    task_id,
                                    phase="verifier prep",
                                    detail=(
                                        "unmapped Harbor imports (no DeclaredDeps pin): "
                                        + ", ".join(spec.unmapped_imports)
                                    ),
                                ),
                                policy,
                            )
                    return (
                        False,
                        format_prep_error(
                            task_id, phase="verifier prep", detail=err[:800]
                        ),
                        policy,
                    )
                # Missing collect paths only soft-succeed when we could not materialize
                # them from Harbor ``test.patch`` (true pre-apply gap). If we wrote the
                # hunks and collect still cannot find them, fail closed.
                err_l = err.lower()
                missing_path = (
                    "file or directory not found" in err_l
                    or "no such file or directory" in err_l
                )
                if missing_path and not written:
                    return True, None, policy
                if missing_path and written:
                    return (
                        False,
                        format_prep_error(
                            task_id,
                            phase="verifier prep",
                            detail=(
                                "collect-only missing path after materializing "
                                f"test.patch hunks ({', '.join(written)}): {err[:600]}"
                            ),
                        ),
                        policy,
                    )
                # Fail closed: plugin policy alone does not green-light a failed collect.
                # Policy is still returned so callers can inspect it, but ok=False.
                return (
                    False,
                    format_prep_error(
                        task_id, phase="verifier prep", detail=err[:800]
                    ),
                    policy,
                )
    return True, None, policy


def verifier_grade_subprocess_env(
    spec: VerifierSpec,
    *,
    base_env: dict[str, str] | None = None,
    plugin_policy: PluginPolicy | None = None,
) -> dict[str, str]:
    """Subprocess env for Harbor ``test.sh`` inside ``/opt/malvin-verifier`` only."""
    env = dict(base_env) if base_env is not None else os.environ.copy()
    env["VIRTUAL_ENV"] = spec.venv_path
    env["PATH"] = f"{spec.venv_path}/bin:" + env.get("PATH", "")
    if spec.grade_pythonpath:
        # Harbor PYTHONPATH layouts (e.g. ``ENV PYTHONPATH=/app/src``) expose the
        # package without an editable install; preserve that for grade/collect.
        existing = env.get("PYTHONPATH", "")
        merged = list(spec.grade_pythonpath)
        if existing:
            merged.extend(p for p in existing.split(":") if p)
        # Deduplicate while preserving order.
        seen: set[str] = set()
        ordered: list[str] = []
        for part in merged:
            if part not in seen:
                seen.add(part)
                ordered.append(part)
        env["PYTHONPATH"] = ":".join(ordered)
    policy = plugin_policy if plugin_policy is not None else spec.plugin_policy
    if policy is not None:
        policy_env = policy.as_env()
        added_opts = policy_env.pop("PYTEST_ADDOPTS", None)
        env.update(policy_env)
        if added_opts:
            env["PYTEST_ADDOPTS"] = _merge_pytest_addopts(
                env.get("PYTEST_ADDOPTS"), added_opts
            )
    return env


@dataclass
class VerifierPrepResult:
    ok: bool
    error: str | None = None
    spec: VerifierSpec | None = None
    plugin_policy: PluginPolicy | None = None
    public_venv_present: bool = False

    def as_dict(self) -> dict[str, Any]:
        """Agent-safe status only (no rich grade-only VerifierSpec fields)."""
        return {
            "ok": self.ok,
            "error": self.error,
            "public_venv_present": self.public_venv_present,
            "venv_path": VERIFIER_VENV_PATH,
        }


def prepare_verifier_grade(
    workspace: Path,
    *,
    tests_dir: Path | None,
    dockerfile: Path | None = None,
    task_id: str = "unknown",
    dry_run: bool = False,
) -> VerifierPrepResult:
    """Grade-only prep: apply ``test.patch`` closure + probe. Not for pre-agent path."""
    if tests_dir is None or not tests_dir.exists():
        return VerifierPrepResult(
            ok=False,
            error=format_prep_error(
                task_id,
                phase="verifier prep",
                detail="tests_dir missing for grade prep",
            ),
        )
    spec = discover_verifier_spec(workspace, tests_dir=tests_dir, dockerfile=dockerfile)
    public_present = Path(f"{spec.venv_path}/bin/python").is_file()
    if dry_run:
        return VerifierPrepResult(
            ok=True, spec=spec, public_venv_present=public_present
        )
    # Local Docker grade uses the Harbor base image (no Modal bake). Materialize
    # the public verifier venv here when missing so grade never falls back to
    # system Python.
    if not public_present:
        for command in verifier_venv_materialize_public_commands(
            spec, workspace=workspace
        ):
            code, detail, _timed_out = _run_shell(command, workspace)
            if code != 0:
                return VerifierPrepResult(
                    ok=False,
                    error=format_prep_error(
                        task_id,
                        phase="verifier prep",
                        detail=detail or f"public venv materialize failed: {command}",
                    ),
                    spec=spec,
                    public_venv_present=False,
                )
        public_present = Path(f"{spec.venv_path}/bin/python").is_file()
        if not public_present:
            return VerifierPrepResult(
                ok=False,
                error=format_prep_error(
                    task_id,
                    phase="verifier prep",
                    detail=f"verifier venv missing after materialize at {spec.venv_path}",
                ),
                spec=spec,
                public_venv_present=False,
            )
    # Seed build backends before offline ``--no-build-isolation`` editable replay.
    if spec.editable_segments:
        for command in verifier_venv_build_system_commands(workspace, spec=spec):
            code, detail, _timed_out = _run_shell(command, workspace)
            if code != 0:
                return VerifierPrepResult(
                    ok=False,
                    error=format_prep_error(
                        task_id,
                        phase="verifier prep",
                        detail=detail or f"build-system install failed: {command}",
                    ),
                    spec=spec,
                    public_venv_present=public_present,
                )
    # Always re-link editables to the mounted workspace (image bake may be stale).
    for command in verifier_venv_replay_editable_commands(spec):
        code, detail, _timed_out = _run_shell(command, workspace)
        if code != 0:
            return VerifierPrepResult(
                ok=False,
                error=format_prep_error(
                    task_id,
                    phase="verifier prep",
                    detail=detail or f"editable replay failed: {command}",
                ),
                spec=spec,
                public_venv_present=public_present,
            )
    for command in verifier_venv_apply_grade_closure_commands(spec):
        code, detail, _timed_out = _run_shell(command, workspace)
        if code != 0:
            return VerifierPrepResult(
                ok=False,
                error=format_prep_error(
                    task_id,
                    phase="verifier prep",
                    detail=detail or f"closure install failed: {command}",
                ),
                spec=spec,
                public_venv_present=public_present,
            )
    ok, err, policy = probe_verifier_env(
        spec,
        workspace=workspace,
        tests_dir=tests_dir,
        task_id=task_id,
        dry_run=False,
    )
    if not ok:
        return VerifierPrepResult(
            ok=False,
            error=err,
            spec=spec,
            plugin_policy=policy,
            public_venv_present=public_present,
        )
    final_spec = VerifierSpec(
        declared=spec.declared,
        public_install_specs=spec.public_install_specs,
        editable_segments=spec.editable_segments,
        harbor_imports=spec.harbor_imports,
        grade_closure_install_specs=spec.grade_closure_install_specs,
        unmapped_imports=spec.unmapped_imports,
        test_sh_body=spec.test_sh_body,
        plugin_policy=policy,
        venv_path=spec.venv_path,
        grade_pythonpath=spec.grade_pythonpath,
    )
    return VerifierPrepResult(
        ok=True,
        spec=final_spec,
        plugin_policy=policy,
        public_venv_present=public_present,
    )


# PyPI distribution names whose importable module differs from ``name.replace("-", "_")``.
_PACKAGE_PROBE_IMPORT_ALIASES: dict[str, str] = {
    "beautifulsoup4": "bs4",
    "opencv-python": "cv2",
    "phonenumberslite": "phonenumbers",
    "pillow": "PIL",
    "pyelftools": "elftools",
    "pyserial": "serial",
    "pysocks": "socks",
    "python-dateutil": "dateutil",
    "pyyaml": "yaml",
    "scikit-image": "skimage",
    "scikit-learn": "sklearn",
}


def _probe_import_name(package_name: str) -> str:
    return _PACKAGE_PROBE_IMPORT_ALIASES.get(
        package_name, package_name.replace("-", "_")
    )


def _probe_checks_for_declared(declared: DeclaredDeps) -> list[tuple[str, str, str]]:
    """Return ``(import_name, expected_spec, display_name)`` probe tuples."""
    checks: list[tuple[str, str, str]] = []
    seen: set[str] = set()
    for name in sorted(declared.package_names()):
        if name in seen:
            continue
        spec = declared.effective_spec(name)
        if spec is None:
            continue
        import_name = _probe_import_name(name)
        checks.append((import_name, spec, name))
        seen.add(name)
    return checks


def _mandatory_probe_python(declared: DeclaredDeps) -> str:
    """Python source run by image-build and runtime verification probes.

    Prefer ``importlib.metadata.version(distribution)`` so packages whose import
    root differs from the distribution name (``pyelftools`` → ``elftools``) still
    pass when the pin is installed. Fall back to import-based discovery only when
    metadata is absent.
    """
    checks = _probe_checks_for_declared(declared)
    check_lines = [f"    ({import_name!r}, {spec!r}, {display!r})," for import_name, spec, display in checks]
    checks_literal = "\n".join(check_lines) if check_lines else ""
    return (
        "import importlib, importlib.util, sys\n"
        "errors = []\n"
        "checks = [\n"
        f"{checks_literal}\n"
        "]\n"
        "for import_name, spec_str, display_name in checks:\n"
        "    version = None\n"
        "    try:\n"
        "        from importlib.metadata import version as pkg_version\n"
        "        version = pkg_version(display_name)\n"
        "    except Exception:\n"
        "        version = None\n"
        "    if version is None:\n"
        "        try:\n"
        "            spec = importlib.util.find_spec(import_name)\n"
        "        except (ImportError, ModuleNotFoundError, ValueError) as exc:\n"
        "            errors.append(f'{display_name}: import check failed ({exc})')\n"
        "            continue\n"
        "        if spec is None:\n"
        "            errors.append(f'{display_name}: not installed (expected {spec_str})')\n"
        "            continue\n"
        "        mod = importlib.import_module(import_name)\n"
        "        version = getattr(mod, '__version__', None)\n"
        "    if version is None:\n"
        "        errors.append(f'{display_name}: installed but version unknown (expected {spec_str})')\n"
        "        continue\n"
        "    try:\n"
        "        from packaging.specifiers import SpecifierSet\n"
        "        from packaging.version import Version\n"
        "        _ops = ('==', '>=', '<=', '!=', '~=', '>', '<')\n"
        "        ver_spec = spec_str\n"
        "        if ver_spec.startswith('['):\n"
        "            end = ver_spec.find(']')\n"
        "            if end != -1:\n"
        "                ver_spec = ver_spec[end + 1 :].lstrip()\n"
        "        if not ver_spec:\n"
        "            continue\n"
        "        normalized = (\n"
        "            ver_spec if any(ver_spec.startswith(op) for op in _ops)\n"
        "            else f'=={ver_spec}'\n"
        "        )\n"
        "        if Version(str(version)) not in SpecifierSet(normalized):\n"
        "            errors.append(f'{display_name} {version} violates {spec_str}')\n"
        "    except Exception as exc:\n"
        "        if display_name == 'pydantic' and spec_str.startswith('>=2'):\n"
        "            if str(version).startswith('1.'):\n"
        "                errors.append(f'pydantic {version} violates {spec_str}')\n"
        "            continue\n"
        "        errors.append(f'{display_name}: version check failed ({version!r} vs {spec_str}: {exc})')\n"
        "if importlib.util.find_spec('httpx'):\n"
        "    import httpx\n"
        "    if httpx.__name__ != 'httpx':\n"
        "        errors.append(f'httpx namespace drift: {httpx.__name__}')\n"
        "if errors:\n"
        "    print('; '.join(errors), file=sys.stderr)\n"
        "    sys.exit(1)\n"
    )


def _mandatory_probe_command(declared: DeclaredDeps) -> str:
    body = _mandatory_probe_python(declared)
    return f"python3 -c {shlex.quote(body)}"


MANDATORY_PROBE_SCRIPT_PATH = "/tmp/malvin_mandatory_probe.py"


def mandatory_probe_script_write_command(declared: DeclaredDeps) -> str:
    """Write mandatory probe source to a fixed path (Modal/Docker image-build safe)."""
    encoded = base64.b64encode(_mandatory_probe_python(declared).encode()).decode()
    return f"echo {shlex.quote(encoded)} | base64 -d > {MANDATORY_PROBE_SCRIPT_PATH}"


def mandatory_probe_script_run_command() -> str:
    return f"python3 {MANDATORY_PROBE_SCRIPT_PATH}"


def mandatory_probe_script_commands(declared: DeclaredDeps) -> list[str]:
    """Return write-then-run shell steps for image-build mandatory probes."""
    return [
        mandatory_probe_script_write_command(declared),
        mandatory_probe_script_run_command(),
    ]


_HTTPX_DRIFT_FIX = "'starlette==1.0.0' 'click==8.3.1' 'typer==0.25.1'"
_HTTPX_DRIFT_PROBE_SCRIPT_PATH = "/tmp/malvin_httpx_drift_probe.py"


def _httpx_drift_probe_python() -> str:
    return (
        "import importlib.util, sys\n"
        "spec = importlib.util.find_spec('httpx')\n"
        "if spec is None:\n"
        "    raise SystemExit(0)\n"
        "import httpx\n"
        "raise SystemExit(1 if httpx.__name__ != 'httpx' else 0)\n"
    )


def _httpx_drift_probe_script_write_command() -> str:
    encoded = base64.b64encode(_httpx_drift_probe_python().encode()).decode()
    return f"echo {shlex.quote(encoded)} | base64 -d > {_HTTPX_DRIFT_PROBE_SCRIPT_PATH}"


def _httpx_drift_probe_script_run_command() -> str:
    return f"python3 {_HTTPX_DRIFT_PROBE_SCRIPT_PATH}"


def _httpx_drift_fix_command() -> str:
    """Run httpx namespace probe; on drift, reinstall starlette/click/typer pins."""
    return (
        f"{_httpx_drift_probe_script_run_command()} || "
        f"pip install --no-cache-dir --force-reinstall {_HTTPX_DRIFT_FIX}"
    )


_PROBE_VIOLATION_RE = re.compile(
    r"(?P<package>[a-zA-Z0-9][\w.-]*)\s+(?P<observed>[\d.]+)\s+violates\s+(?P<expected>.+)"
)


def _parse_probe_stderr_fragments(detail: str) -> list[tuple[str | None, str | None, str | None]]:
    """Return one ``(package, observed, expected)`` tuple per probe stderr fragment."""
    parsed: list[tuple[str | None, str | None, str | None]] = []
    for fragment in detail.replace(";", "\n").splitlines():
        fragment = fragment.strip()
        if not fragment:
            continue
        match = _PROBE_VIOLATION_RE.search(fragment)
        if match:
            parsed.append(
                (match.group("package"), match.group("observed"), match.group("expected"))
            )
            continue
        if "namespace drift" in fragment:
            parsed.append(("httpx", fragment, "httpx"))
            continue
        pkg_match = re.match(r"(\S+):", fragment)
        if pkg_match:
            pkg = pkg_match.group(1)
            if "not installed" in fragment:
                parsed.append((pkg, "not installed", fragment))
                continue
            if "import check failed" in fragment:
                parsed.append((pkg, "import failed", fragment))
                continue
            if "version unknown" in fragment:
                parsed.append((pkg, "unknown", fragment))
                continue
            if "version check failed" in fragment:
                parsed.append((pkg, "check failed", fragment))
    return parsed


def _parse_probe_stderr(detail: str) -> tuple[str | None, str | None, str | None]:
    """Return the first ``(package, observed, expected)`` parsed from mandatory-probe stderr."""
    fragments = _parse_probe_stderr_fragments(detail)
    if fragments:
        return fragments[0]
    return None, None, None


def _reconcile_declared_deps_commands(
    declared: DeclaredDeps,
    *,
    registry_pull: bool = False,
) -> list[str]:
    """Force-reinstall declared pins and pyproject/lockfile packages at image build."""
    cmds: list[str] = []
    if declared.bulk_pins and not registry_pull:
        pkg_args = [f"'{name}=={ver}'" for name, ver in sorted(declared.bulk_pins.items())]
        cmds.append("pip install --no-cache-dir --force-reinstall " + " ".join(pkg_args))
    reconcile_names = set(declared.constraints) | set(declared.lockfile_pins)
    if registry_pull:
        # Modal base-image layering can downgrade Harbor bulk pins (e.g. aiohttp,
        # pydantic) after registry pull; reconcile declared pins without replaying
        # every Dockerfile pip step.
        reconcile_names |= set(declared.bulk_pins)
    covered = {name.lower() for name in declared.bulk_pins} if not registry_pull else set()
    extras: list[str] = []
    for name in sorted(reconcile_names):
        if name in covered:
            continue
        pip_spec = declared.pip_install_spec(name)
        if pip_spec:
            extras.append(f"'{pip_spec}'")
    if extras:
        cmds.append("pip install --no-cache-dir --force-reinstall " + " ".join(extras))
    return cmds


def run_post_prep_probes(
    workspace: Path,
    declared: DeclaredDeps,
    *,
    task_id: str,
    phase: str = "runtime probe",
) -> list[str]:
    """Run verification probes; return human-readable errors (empty when ok)."""
    command = _mandatory_probe_command(declared)
    code, detail, _timed_out = _run_shell(command, workspace)
    if code == 0:
        return []
    observed_text = detail.strip() or "probe failed"
    fragments = _parse_probe_stderr_fragments(observed_text)
    if not fragments:
        return [
            format_prep_error(
                task_id,
                phase=phase,
                detail=observed_text,
                hint="check registry cache bust / pyproject.toml reconcile",
            )
        ]
    return [
        format_prep_error(
            task_id,
            phase=phase,
            package=package,
            observed=observed,
            expected=expected,
            detail=None if package is not None else observed_text,
            hint="check registry cache bust / pyproject.toml reconcile",
        )
        for package, observed, expected in fragments
    ]


def pydantic_pins_for_cache_bust(
    dockerfile: Path | None,
    workspace: Path | None = None,
) -> tuple[str | None, str | None]:
    """Return task pydantic pins when present in declared dependencies."""
    if dockerfile is None or not dockerfile.is_file() or workspace is None:
        return None, None
    workspace = workspace.resolve()
    declared = declared_python_dependencies(workspace, dockerfile)
    pydantic_spec = declared.effective_spec("pydantic")
    if pydantic_spec is not None and pydantic_spec.startswith("=="):
        return pydantic_spec[2:], declared.lockfile_pins.get("pydantic-core")
    for req_rel in requirements_paths_from_dockerfile(dockerfile):
        pydantic_ver, core_ver = read_pydantic_pins_from_requirements(workspace / req_rel)
        if pydantic_ver is not None:
            return pydantic_ver, core_ver
    return None, None


def _pydantic_v1_eviction_command() -> str:
    """Evict stale pydantic v1 when the image has pydantic but no task declaration."""
    return (
        "python3 -c \""
        "import importlib.util, sys; "
        "spec=importlib.util.find_spec('pydantic'); "
        "ver = None; "
        "exec('try:\\n from importlib.metadata import version as pkg_version\\n ver=pkg_version(\\\"pydantic\\\")\\nexcept Exception: pass') if spec else None; "
        "import pydantic; "
        "ver = ver or getattr(pydantic, '__version__', ''); "
        "sys.exit(1 if str(ver).startswith('1.') else 0)"
        "\" 2>/dev/null || "
        "pip install --no-cache-dir 'pydantic>=2,<3'"
    )


_LINT_GATE_TOOLS = ("ruff", "mypy", "pre-commit")
_TOX_RUNNER_TOOLS = ("tox", "invoke")
_TOX_VARS_RE = re.compile(r"\{\[vars\]([^\}]+)\}")


def _tox_ini_section_text(tox_text: str, header: str) -> str | None:
    """Return the body of a tox.ini section named *header* (e.g. ``[vars]``)."""
    lines = tox_text.splitlines()
    start: int | None = None
    for index, line in enumerate(lines):
        if line.strip() == header:
            start = index + 1
            break
    if start is None:
        return None
    section_lines: list[str] = []
    for line in lines[start:]:
        stripped = line.strip()
        if stripped.startswith("[") and stripped.endswith("]"):
            break
        section_lines.append(line)
    return "\n".join(section_lines) if section_lines else ""


def tox_lint_section_text(tox_text: str) -> str | None:
    """Return the body of ``[testenv:lint]`` from a ``tox.ini`` string."""
    return _tox_ini_section_text(tox_text, "[testenv:lint]")


def tox_ini_vars(tox_text: str) -> dict[str, str]:
    """Parse ``[vars]`` substitutions from a tox.ini string."""
    section = _tox_ini_section_text(tox_text, "[vars]")
    if section is None:
        return {}
    vars_map: dict[str, str] = {}
    for raw in section.splitlines():
        stripped = raw.strip()
        if not stripped or stripped.startswith("#") or "=" not in stripped:
            continue
        key, value = stripped.split("=", 1)
        vars_map[key.strip()] = value.strip()
    return vars_map


def expand_tox_vars(command: str, vars_map: dict[str, str]) -> str:
    """Replace ``{[vars]name}`` placeholders using *vars_map*."""

    def _replace(match: re.Match[str]) -> str:
        return vars_map.get(match.group(1).strip(), match.group(0))

    return _TOX_VARS_RE.sub(_replace, command)


def workspace_has_justfile(workspace: Path) -> bool:
    return (workspace / "justfile").is_file() or (workspace / "Justfile").is_file()


def tox_lint_check_commands(workspace: Path) -> list[str]:
    """Return ``commands`` from ``[testenv:lint]`` when present (tox vars expanded)."""
    tox_path = workspace / "tox.ini"
    if not tox_path.is_file():
        return []
    tox_text = tox_path.read_text(encoding="utf-8")
    section = tox_lint_section_text(tox_text)
    if section is None:
        return []
    vars_map = tox_ini_vars(tox_text)
    commands: list[str] = []
    in_commands = False
    for raw in section.splitlines():
        line = raw.rstrip()
        stripped = line.strip()
        if not stripped or stripped.startswith("#"):
            continue
        if re.match(r"^commands\s*=", stripped, re.I):
            in_commands = True
            continue
        if in_commands:
            if line and not line[0].isspace():
                in_commands = False
                continue
            if stripped:
                commands.append(expand_tox_vars(stripped, vars_map))
    return commands


def lint_gate_tool_pins(workspace: Path) -> dict[str, str]:
    """Return pinned lint-gate tool versions declared by the workspace."""
    candidates = (
        workspace / "requirements" / "lint.txt",
        workspace / "requirements" / "dev.txt",
        workspace / "requirements" / "raw" / "lint.txt",
    )
    for path in candidates:
        pins = _pins_from_requirements_file(path)
        tools = {name: pins[name] for name in _LINT_GATE_TOOLS if name in pins}
        if tools:
            return tools
    return {}


def tox_runner_tool_pins(workspace: Path) -> dict[str, str]:
    """Return pinned tox/invoke versions from workspace requirements, if any."""
    candidates = (
        workspace / "requirements" / "runner.txt",
        workspace / "requirements" / "dev.txt",
        workspace / "requirements" / "raw" / "runner.txt",
    )
    for path in candidates:
        pins = _pins_from_requirements_file(path)
        tools = {name: pins[name] for name in _TOX_RUNNER_TOOLS if name in pins}
        if tools:
            return tools
    return {}


def just_install_command(workspace: Path) -> str | None:
    """Install the ``just`` binary when the workspace has a justfile.

    Prefers a prebuilt GitHub release tarball over ``cargo install`` so image
    builds do not recompile the Rust crate on every warm layer.
    """
    if not workspace_has_justfile(workspace):
        return None
    # Keep the URL pinned; bump intentionally when upgrading just.
    just_version = "1.40.0"
    archive = f"just-{just_version}-x86_64-unknown-linux-musl.tar.gz"
    url = (
        "https://github.com/casey/just/releases/download/"
        f"{just_version}/{archive}"
    )
    return (
        "command -v just >/dev/null 2>&1 || "
        f"(curl -fsSL {shlex.quote(url)} -o /tmp/just.tgz && "
        "tar -xzf /tmp/just.tgz -C /usr/local/bin just && "
        "chmod +x /usr/local/bin/just && rm -f /tmp/just.tgz)"
    )


def tox_runner_install_command(workspace: Path) -> str | None:
    """Install tox/invoke when the workspace uses tox or just recipes that call them.

    Always installs (no soft ``command -v`` skip). Tox is clamped to
    :data:`tox_gates.MIN_TOX_FOR_SKIP_ENV_INSTALL` so offline agent checks that
    inject ``--skip-env-install`` resolve a capable runner under TOOLCHAIN_PATH.
    """
    from tox_gates import clamp_tox_version, image_build_pip_install_command

    needs_tox = (workspace / "tox.ini").is_file() or workspace_has_justfile(workspace)
    if not needs_tox:
        return None
    pins = dict(tox_runner_tool_pins(workspace))
    if "tox" in pins:
        pins["tox"] = clamp_tox_version(pins["tox"])
    if pins:
        args = " ".join(
            shlex.quote(f"{name}=={version}") for name, version in sorted(pins.items())
        )
    else:
        args = shlex.quote(f"tox=={clamp_tox_version(None)}")
    return image_build_pip_install_command(args)


def workspace_lint_tool_install_command(workspace: Path) -> str | None:
    """Install tox lint-gate CLIs at image build for offline malvin quality gates."""
    if (workspace / "uv.lock").is_file():
        return None
    if not tox_lint_check_commands(workspace):
        return None
    pins = lint_gate_tool_pins(workspace)
    if not pins:
        return None
    args = " ".join(shlex.quote(f"{name}=={version}") for name, version in sorted(pins.items()))
    return f"python3 -m pip install --no-cache-dir {args}"


PRECOMMIT_WARM_SCRIPT_PATH = "/tmp/malvin_precommit_warm.sh"


def _precommit_warm_script_body(workspace: Path) -> str:
    """Bash script to bootstrap ``pre-commit`` and warm hook environments.

    ``install-hooks`` is best-effort: configs often pin ``default_language_version``
    to interpreters absent from Harbor images (e.g. python3.8). Failing closed on
    that aborts image build even when ``--test`` only needs ecosystem smoke.
    """
    pin = _precommit_pin_from_workspace(workspace)
    pip_spec = f"pre-commit=={pin}" if pin else "pre-commit"
    venv_bin = f"{_UV_PROJECT_VENV}/bin/pre-commit"
    soft = ' || echo "malvin: pre-commit install-hooks failed (continuing)" >&2'
    return (
        "#!/usr/bin/env bash\n"
        "set -euo pipefail\n"
        "if command -v pre-commit >/dev/null 2>&1; then\n"
        f"  pre-commit install-hooks{soft}\n"
        f"elif test -x {shlex.quote(venv_bin)}; then\n"
        f"  PATH={shlex.quote(_UV_PROJECT_VENV + '/bin')}:\"$PATH\" "
        f"pre-commit install-hooks{soft}\n"
        "else\n"
        f"  python3 -m pip install --no-cache-dir {shlex.quote(pip_spec)}\n"
        f"  pre-commit install-hooks{soft}\n"
        "fi\n"
    )


def precommit_warm_script_write_command(workspace: Path) -> str | None:
    """Write pre-commit warm script to a fixed path (Modal/Docker image-build safe)."""
    if not (workspace / ".pre-commit-config.yaml").is_file():
        return None
    encoded = base64.b64encode(_precommit_warm_script_body(workspace).encode()).decode()
    return f"echo {shlex.quote(encoded)} | base64 -d > {PRECOMMIT_WARM_SCRIPT_PATH}"


def precommit_warm_script_run_command() -> str:
    return f"bash {PRECOMMIT_WARM_SCRIPT_PATH}"


def precommit_warm_script_commands(workspace: Path) -> list[str]:
    """Return write-then-run shell steps for image-build pre-commit hook warming."""
    write = precommit_warm_script_write_command(workspace)
    if write is None:
        return []
    return [write, precommit_warm_script_run_command()]


def precommit_install_hooks_command(workspace: Path) -> str | None:
    """Backward-compatible alias returning only the run step."""
    commands = precommit_warm_script_commands(workspace)
    return commands[-1] if commands else None


_UV_BOOTSTRAP_SHELL = (
    "command -v uv >/dev/null 2>&1 || python3 -m pip install --no-cache-dir uv"
)
_UV_PROJECT_VENV = ".venv"


def _pyproject_has_uv_dev_group(workspace: Path) -> bool:
    pyproject = workspace / "pyproject.toml"
    if not pyproject.is_file():
        return False
    raw = tomllib.loads(pyproject.read_text(encoding="utf-8"))
    groups = raw.get("dependency-groups")
    return isinstance(groups, dict) and "dev" in groups


def _read_pyproject_build_system_requires(pyproject: Path) -> list[str]:
    """Return ``[build-system].requires`` entries from ``pyproject.toml``."""
    if not pyproject.is_file():
        return []
    raw = tomllib.loads(pyproject.read_text(encoding="utf-8"))
    build_system = raw.get("build-system")
    if not isinstance(build_system, dict):
        return []
    requires = build_system.get("requires")
    if not isinstance(requires, list):
        return []
    return [req for req in requires if isinstance(req, str) and req.strip()]


def _workspace_has_ruff_signal(workspace: Path) -> bool:
    """True when ruff is likely used by malvin quality gates for this workspace."""
    pyproject = workspace / "pyproject.toml"
    if pyproject.is_file():
        raw = tomllib.loads(pyproject.read_text(encoding="utf-8"))
        groups = raw.get("dependency-groups")
        if isinstance(groups, dict):
            dev = groups.get("dev")
            if isinstance(dev, list) and any(
                isinstance(dep, str) and dep.split("[", 1)[0].strip() == "ruff" for dep in dev
            ):
                return True
    lockfile = workspace / "uv.lock"
    if lockfile.is_file():
        return 'name = "ruff"' in lockfile.read_text(encoding="utf-8")
    return False


_UV_OFFLINE_SMOKE_PREFIX = "UV_OFFLINE=1 UV_NO_SYNC=1"


def uv_sync_dev_command(workspace: Path) -> str | None:
    """Return shell steps to warm a uv venv when the workspace uses uv.

    Callers must run the returned command in ``/app`` during image build with network
    access so later offline ``uv sync`` / ``uv run`` gates can succeed.
    """
    if not (workspace / "uv.lock").is_file():
        return None
    sync = "uv sync --group dev" if _pyproject_has_uv_dev_group(workspace) else "uv sync"
    return f"{_UV_BOOTSTRAP_SHELL} && {sync}"


def uv_pip_build_system_command(workspace: Path) -> str | None:
    """Return shell steps to cache ``[build-system].requires`` for offline ``uv run``."""
    if not (workspace / "uv.lock").is_file():
        return None
    requires = _read_pyproject_build_system_requires(workspace / "pyproject.toml")
    if not requires:
        return None
    quoted = " ".join(shlex.quote(req) for req in requires)
    return (
        f"{_UV_BOOTSTRAP_SHELL} && uv pip install --python {_UV_PROJECT_VENV} {quoted}"
    )


def uv_editable_install_command(workspace: Path) -> str | None:
    """Return shell steps to pre-install the project editable for offline rebuilds.

    Hatchling editable installs need the ``editables`` package even when it is not
    listed in ``[build-system].requires``; install it before ``-e .``.
    """
    if not (workspace / "uv.lock").is_file():
        return None
    return (
        f"{_UV_BOOTSTRAP_SHELL} && "
        f"uv pip install --python {_UV_PROJECT_VENV} editables && "
        f"uv pip install --python {_UV_PROJECT_VENV} -e . --no-build-isolation"
    )


def uv_offline_smoke_commands(workspace: Path) -> list[str]:
    """Gate-equivalent offline checks to run at image build after cache warming.

    Lint smokes (``uv run ruff check``) soft-fail like pre-commit hook install:
    a missing console script must not abort the image build after deps warmed.
    """
    if not (workspace / "uv.lock").is_file():
        return []
    commands: list[str] = []
    sync = "uv sync --offline --group dev" if _pyproject_has_uv_dev_group(workspace) else "uv sync --offline"
    commands.append(f"{_UV_OFFLINE_SMOKE_PREFIX} {sync}")
    if _workspace_has_ruff_signal(workspace):
        commands.append(
            f"{_UV_OFFLINE_SMOKE_PREFIX} uv run ruff check "
            '|| echo "malvin: uv run ruff check failed (continuing)" >&2'
        )
    return commands


def workspace_declared_repin_command(
    workspace: Path,
    dockerfile: Path | None = None,
) -> str | None:
    """Force-reinstall declared pins after warm pip installs that may clobber them.

    Example: installing tox upgrades ``packaging``, which then fails the mandatory probe
    against Adaptix's ``packaging==24.2`` pin.

    Bulk pins and pyproject/lockfile constraints share one ``pip install`` so transitive
    deps of bulk packages cannot float past declared ranges (httpx: ``twine`` pulling
    ``rich`` 15 while ``rich>=10,<15`` is declared).
    """
    if dockerfile is None or not dockerfile.is_file():
        return None
    declared = declared_python_dependencies(workspace.resolve(), dockerfile)
    specs: list[str] = []
    covered: set[str] = set()
    for name, ver in sorted(declared.bulk_pins.items()):
        specs.append(f"'{name}=={ver}'")
        covered.add(name.lower())
    for name in sorted(set(declared.constraints) | set(declared.lockfile_pins)):
        key = name.lower()
        if key in covered:
            continue
        pip_spec = declared.pip_install_spec(name)
        if pip_spec:
            specs.append(f"'{pip_spec}'")
            covered.add(key)
    if not specs:
        return None
    return "pip install --no-cache-dir --force-reinstall " + " ".join(specs)


def workspace_image_warm_commands(
    workspace: Path,
    dockerfile: Path | None = None,
) -> list[str]:
    """Shell commands to warm offline agent quality gates at Modal image build."""
    commands: list[str] = []
    just_install = just_install_command(workspace)
    if just_install:
        commands.append(just_install)
    tox_runner = tox_runner_install_command(workspace)
    if tox_runner:
        commands.append(tox_runner)
    lint_install = workspace_lint_tool_install_command(workspace)
    if lint_install:
        commands.append(lint_install)
    # Seed system/default pip before uv warm so offline Prep sync (-e --no-build-isolation)
    # finds hatchling/editables outside .venv.
    pip_seed = default_pip_editable_seed_command(workspace, dockerfile)
    if pip_seed:
        commands.append(pip_seed)
    uv_sync = uv_sync_dev_command(workspace)
    if uv_sync:
        commands.append(uv_sync)
    build_system = uv_pip_build_system_command(workspace)
    if build_system:
        commands.append(build_system)
    editable = uv_editable_install_command(workspace)
    if editable:
        commands.append(editable)
    precommit_cmds = precommit_warm_script_commands(workspace)
    commands.extend(precommit_cmds)
    from tox_gates import tox_gate_env_warm_command, tox_gate_precommit_warm_command

    tox_gate_env = tox_gate_env_warm_command(workspace)
    if tox_gate_env:
        commands.append(tox_gate_env)
    elif tox_lint_check_commands(workspace):
        commands.append("tox -e lint --notest --skip-missing-interpreters true")
    tox_pc = tox_gate_precommit_warm_command(workspace)
    if tox_pc:
        commands.append(tox_pc)
    # Tox/lint pip installs can upgrade transitive pins (e.g. packaging); restore declared pins.
    repin = workspace_declared_repin_command(workspace, dockerfile)
    if repin:
        commands.append(repin)
    smoke = uv_offline_smoke_commands(workspace)
    if smoke and build_system:
        # ``uv sync --offline`` reconciles the venv to the lockfile and drops
        # build-system packages that are not declared as runtime deps.
        commands.append(smoke[0])
        commands.append(build_system)
        commands.extend(smoke[1:])
    else:
        commands.extend(smoke)
    return commands


def registry_image_cache_bust_commands(
    dockerfile: Path | None = None,
    workspace: Path | None = None,
    *,
    registry_pull: bool = False,
) -> list[str]:
    """Modal registry cache bust: reconcile declared deps, drift fixes, mandatory probe.

    When ``pyproject.toml`` declares packages omitted from Dockerfile bulk pins (e.g.
    aiomonitor ``pydantic>=2.0.0``), reconcile commands install them unconditionally
    after bulk pin replay — not only when bulk pins are absent.

    With ``registry_pull=True``, skip Dockerfile bulk-pin replay (full ``RUN pip install``
    replay) because Harbor registry images already ship those pins; still reconcile
    declared bulk pins when Modal base-image layering may have clobbered them.
    """
    declared = (
        declared_python_dependencies(workspace.resolve(), dockerfile)
        if workspace is not None and dockerfile is not None and dockerfile.is_file()
        else DeclaredDeps({}, {}, (), {})
    )
    cmds: list[str] = []
    reconcile = _reconcile_declared_deps_commands(declared, registry_pull=registry_pull)
    cmds.extend(reconcile)
    if not declared.package_names():
        cmds.append(_pydantic_v1_eviction_command())
    cmds.append(_httpx_drift_probe_script_write_command())
    cmds.append(_httpx_drift_fix_command())
    # httpx drift fix can upgrade transitive pins (e.g. typing-extensions); re-pin before probe.
    if reconcile:
        cmds.extend(reconcile)
    cmds.extend(mandatory_probe_script_commands(declared))
    return cmds


def prepare_task_sandbox(
    spec: Any,
    workspace: Path,
    *,
    dry_run: bool = False,
    deadline: float | None = None,
    offline_editable: bool = True,
    verify_probes: bool = True,
) -> SandboxPrepResult:
    """Offline editable replay and declared-dependency verification probes."""
    workspace = workspace.resolve()
    task_id = getattr(spec, "task_id", "unknown")
    dockerfile = spec.dockerfile if getattr(spec, "dockerfile", None) and spec.dockerfile.is_file() else None
    declared = declared_python_dependencies(workspace, dockerfile)
    sync_commands = workspace_sync_commands_from_dockerfile(
        spec.dockerfile,
        offline_editable=offline_editable,
    )
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
            err = format_prep_error(
                task_id,
                phase="runtime sync",
                detail=f"sync timed out for {command!r}" + (f": {detail}" if detail else ""),
                hint="check offline editable replay",
            )
            click.echo(err, err=True)
            return SandboxPrepResult(
                sync_commands=tuple(sync_commands),
                sync_warnings=tuple(sync_warnings),
                probe_errors=(err,),
                ok=False,
                timed_out=True,
            )
        if code != 0:
            err = format_prep_error(
                task_id,
                phase="runtime sync",
                detail=f"exit {code} for {command!r}" + (f": {detail}" if detail else ""),
                hint="check offline editable replay",
            )
            sync_warnings.append(err)
            click.echo(err, err=True)
            return SandboxPrepResult(
                sync_commands=tuple(sync_commands),
                sync_warnings=tuple(sync_warnings),
                probe_errors=(err,),
                ok=False,
            )

    if dry_run or not verify_probes:
        return SandboxPrepResult(
            sync_commands=tuple(sync_commands),
            sync_warnings=tuple(sync_warnings),
            probe_errors=(),
            ok=True,
        )

    probe_errors = run_post_prep_probes(workspace, declared, task_id=task_id)
    if probe_errors:
        for err in probe_errors:
            click.echo(err, err=True)
        return SandboxPrepResult(
            sync_commands=tuple(sync_commands),
            sync_warnings=tuple(sync_warnings),
            probe_errors=tuple(probe_errors),
            ok=False,
        )

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
    assert len(sync) == 1, sync
    assert "-e" in sync[0] and "--no-deps" in sync[0]


def _test_workspace_sync_commands_fastapi() -> None:
    text = """RUN git clone https://github.com/fastapi/fastapi .
RUN pip install --no-cache-dir -e ".[all]" && pip install --no-cache-dir pytest
"""
    runs = parse_dockerfile_run_commands(text)
    sync = _sync_commands_from_runs(runs)
    assert len(sync) == 1, sync
    assert '-e ".[all]"' in sync[0] and "--no-deps" in sync[0]


def _test_bash_lc_pip_intents_ignore_shell_noise() -> None:
    text = (
        "FROM x\n"
        'RUN bash -lc "if [ -f requirements.txt ]; then pip install -r requirements.txt; fi; '
        'pip install --no-cache-dir -e . pytest pint"\n'
    )
    intents = collect_pip_install_intents(text)
    joined = " ".join(intents)
    assert "pip install --no-cache-dir -e . pytest pint" in joined
    unpinned = collect_unpinned_package_names(intents)
    assert "pytest" in unpinned
    assert "pint" in unpinned
    assert "fi" not in unpinned
    assert "if" not in unpinned


def _test_requirement_inline_comments_stripped_for_pip() -> None:
    """OpenStack-style ``pkg>=1 # MIT`` must not reach pip install args."""
    assert _strip_requirement_comment("beautifulsoup4>=4.8.0 # MIT") == "beautifulsoup4>=4.8.0"
    assert _requirement_line_package("beautifulsoup4>=4.8.0 # MIT") == (
        "beautifulsoup4",
        ">=4.8.0",
    )
    assert _parse_dependency_spec("PyYAML>=5.3.1 # MIT") == ("pyyaml", ">=5.3.1")
    with tempfile.TemporaryDirectory() as tmp:
        root = Path(tmp)
        (root / "requirements.txt").write_text(
            "beautifulsoup4>=4.8.0 # MIT\nPyYAML>=5.3.1 # MIT\n",
            encoding="utf-8",
        )
        dockerfile = root / "Dockerfile"
        dockerfile.write_text(
            "FROM x\nRUN pip install -r requirements.txt\n",
            encoding="utf-8",
        )
        declared = declared_python_dependencies(root, dockerfile)
        soup = declared.pip_install_spec("beautifulsoup4")
        assert soup == "beautifulsoup4>=4.8.0"
        assert "#" not in soup
        cmds = registry_image_cache_bust_commands(
            dockerfile, workspace=root, registry_pull=True
        )
        joined = " ".join(cmds)
        assert "# MIT" not in joined
        assert "beautifulsoup4>=4.8.0" in joined


def _test_pep508_extras_preserved_in_pip_install_spec() -> None:
    """``fastapi-cli[standard] >=0.0.8`` must not become ``fastapi-cli==[standard]…``."""
    import tempfile

    assert _parse_dependency_spec("fastapi-cli[standard] >=0.0.8") == (
        "fastapi-cli",
        "[standard]>=0.0.8",
    )
    assert _parse_dependency_spec("uvicorn[standard] >=0.12.0") == (
        "uvicorn",
        "[standard]>=0.12.0",
    )
    with tempfile.TemporaryDirectory() as tmp:
        root = Path(tmp)
        (root / "pyproject.toml").write_text(
            '[project]\nname = "demo"\nversion = "0"\n'
            "dependencies = [\n"
            '  "fastapi-cli[standard] >=0.0.8",\n'
            '  "uvicorn[standard] >=0.12.0",\n'
            "]\n",
            encoding="utf-8",
        )
        dockerfile = root / "Dockerfile"
        dockerfile.write_text("FROM x\nRUN pip install -e .\n", encoding="utf-8")
        declared = declared_python_dependencies(root, dockerfile)
        assert declared.pip_install_spec("fastapi-cli") == "fastapi-cli[standard]>=0.0.8"
        assert declared.pip_install_spec("uvicorn") == "uvicorn[standard]>=0.12.0"
        cmds = registry_image_cache_bust_commands(
            dockerfile, workspace=root, registry_pull=True
        )
        joined = " ".join(cmds)
        assert "fastapi-cli==[" not in joined
        assert "uvicorn==[" not in joined
        assert "fastapi-cli[standard]>=0.0.8" in joined


def _test_requirements_editable_and_constraints_declared() -> None:
    with tempfile.TemporaryDirectory() as tmp:
        root = Path(tmp)
        (root / "requirements.txt").write_text(
            "-e .[cli]\nfixtures>=3.0.0\nrich\n",
            encoding="utf-8",
        )
        (root / "pyproject.toml").write_text(
            '[project]\nname = "demo"\nversion = "0"\n'
            'optional-dependencies = { cli = ["click>=8"] }\n',
            encoding="utf-8",
        )
        dockerfile = root / "Dockerfile"
        dockerfile.write_text(
            "FROM x\nRUN pip install -r requirements.txt\n",
            encoding="utf-8",
        )
        declared = declared_python_dependencies(root, dockerfile)
        assert any("-e" in seg for seg in declared.editable_segments)
        assert "fixtures" in declared.constraints or "fixtures" in declared.package_names()
        assert "rich" in declared.package_names()
        assert "click" in declared.package_names()


def _test_poetry_extra_and_runtime_deps_declared() -> None:
    with tempfile.TemporaryDirectory() as tmp:
        root = Path(tmp)
        (root / "pyproject.toml").write_text(
            "[tool.poetry]\n"
            'name = "demo"\n'
            'version = "0"\n'
            'dependencies.python = "^3.10"\n'
            'dependencies.rich = ">=14"\n'
            'dependencies.pytest = { version = ">=8", optional = true }\n'
            'extras.check = [ "pytest" ]\n',
            encoding="utf-8",
        )
        (root / "demo").mkdir()
        (root / "demo" / "__init__.py").write_text("", encoding="utf-8")
        dockerfile = root / "Dockerfile"
        dockerfile.write_text(
            'FROM x\nRUN pip install -e ".[check]"\n',
            encoding="utf-8",
        )
        declared = declared_python_dependencies(root, dockerfile)
        assert "rich" in declared.package_names()
        assert "pytest" in declared.package_names()


def _test_fixture_imports_not_unmapped_for_workspace_project() -> None:
    with tempfile.TemporaryDirectory() as tmp:
        root = Path(tmp)
        workspace = root / "ws"
        workspace.mkdir()
        (workspace / "pkg").mkdir()
        (workspace / "pkg" / "__init__.py").write_text("", encoding="utf-8")
        (workspace / "pyproject.toml").write_text(
            '[project]\nname = "pkg"\nversion = "0"\n',
            encoding="utf-8",
        )
        tests = root / "tests"
        tests.mkdir()
        (tests / "test.patch").write_text(
            "diff --git a/fixtures/sample.py b/fixtures/sample.py\n"
            "--- /dev/null\n"
            "+++ b/fixtures/sample.py\n"
            "@@ -0,0 +1,1 @@\n"
            "+import flask\n"
            "diff --git a/tests/test_pkg.py b/tests/test_pkg.py\n"
            "--- /dev/null\n"
            "+++ b/tests/test_pkg.py\n"
            "@@ -0,0 +1,1 @@\n"
            "+import pkg\n",
            encoding="utf-8",
        )
        dockerfile = root / "Dockerfile"
        dockerfile.write_text("FROM x\nRUN pip install pytest\n", encoding="utf-8")
        spec = discover_verifier_spec(workspace, tests_dir=tests, dockerfile=dockerfile)
        assert "flask" not in spec.harbor_imports
        assert "pkg" not in spec.unmapped_imports
        assert any("-e" in seg for seg in spec.editable_segments)


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
    tasks_root = malvin_repo_root().parent / "deep-swe" / "tasks"
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
        for cmd in sync:
            assert _is_editable_pip_segment(cmd) or not _is_bulk_pip_segment(cmd), (
                slug,
                cmd,
            )
            if _is_editable_pip_segment(cmd):
                assert "--no-deps" in cmd and "--no-build-isolation" in cmd, (slug, cmd)
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
    tasks_root = malvin_repo_root().parent / "deep-swe" / "tasks"
    dockerfile = tasks_root / "fastapi-deprecation-response-headers" / "environment" / "Dockerfile"
    if not dockerfile.is_file():
        return
    sync = workspace_sync_commands_from_dockerfile(dockerfile)
    assert len(sync) == 1, sync
    assert "-e" in sync[0] and "--no-deps" in sync[0]


def _test_should_replay_skips_apt_and_git() -> None:
    assert not should_replay_run_command("apt-get update && apt-get install -y build-essential")
    assert not should_replay_run_command("git clone https://github.com/foo .")
    assert should_replay_run_command("go mod download")



def _test_hybrid_poetry_runtime_sync_skipped() -> None:
    tasks_root = malvin_repo_root().parent / "deep-swe" / "tasks"
    dockerfile = tasks_root / "textual-kitty-key-phases" / "environment" / "Dockerfile"
    if not dockerfile.is_file():
        return
    sync = workspace_sync_commands_from_dockerfile(dockerfile)
    assert sync == [], sync


def _test_hybrid_pnpm_runtime_sync_skipped() -> None:
    tasks_root = malvin_repo_root().parent / "deep-swe" / "tasks"
    dockerfile = tasks_root / "koota-entity-snapshot-rollback" / "environment" / "Dockerfile"
    if not dockerfile.is_file():
        return
    sync = workspace_sync_commands_from_dockerfile(dockerfile)
    assert sync == [], sync


def _test_tox_lint_check_commands() -> None:
    import tempfile

    with tempfile.TemporaryDirectory() as tmp:
        root = Path(tmp)
        assert tox_lint_check_commands(root) == []
        (root / "tox.ini").write_text(
            "[vars]\n"
            "lint_all = src/ tests/\n"
            "lint_mypy = src/\n"
            "[testenv:lint]\n"
            "commands =\n"
            "  ruff check {[vars]lint_all} --fix\n"
            "  mypy {[vars]lint_mypy}\n",
            encoding="utf-8",
        )
        assert tox_lint_check_commands(root) == [
            "ruff check src/ tests/ --fix",
            "mypy src/",
        ]


def _test_just_and_tox_runner_install_commands() -> None:
    import tempfile

    with tempfile.TemporaryDirectory() as tmp:
        root = Path(tmp)
        assert just_install_command(root) is None
        assert tox_runner_install_command(root) is None
        (root / "justfile").write_text("lint:\n    tox -e lint\n", encoding="utf-8")
        just_cmd = just_install_command(root)
        assert just_cmd is not None
        assert "github.com/casey/just/releases" in just_cmd
        assert "cargo install just" not in just_cmd
        tox_cmd = tox_runner_install_command(root)
        assert tox_cmd is not None
        assert "tox" in tox_cmd
        req_dir = root / "requirements"
        req_dir.mkdir()
        (req_dir / "runner.txt").write_text("tox==4.23.2\ninvoke==2.2.0\n", encoding="utf-8")
        pinned = tox_runner_install_command(root)
        assert pinned is not None
        # 4.23.2 predates --skip-env-install; clamp to the offline floor.
        assert "tox==4.42.0" in pinned
        assert "tox==4.23.2" not in pinned
        assert "invoke==2.2.0" in pinned
        assert "/opt/venv/bin/python -m pip install" in pinned
        assert "command -v tox" not in pinned
        (req_dir / "runner.txt").write_text("tox==4.50.0\n", encoding="utf-8")
        newer = tox_runner_install_command(root)
        assert newer is not None
        assert "tox==4.50.0" in newer


def _test_workspace_lint_tool_install_command() -> None:
    import tempfile

    with tempfile.TemporaryDirectory() as tmp:
        root = Path(tmp)
        assert workspace_lint_tool_install_command(root) is None
        req_dir = root / "requirements"
        req_dir.mkdir()
        (req_dir / "lint.txt").write_text(
            "ruff==0.9.1\nmypy==1.14.0\npre-commit==4.0.1\n",
            encoding="utf-8",
        )
        (root / "tox.ini").write_text(
            "[testenv:lint]\n"
            "deps = -r requirements/lint.txt\n"
            "commands =\n"
            "  ruff check src/ --fix\n",
            encoding="utf-8",
        )
        cmd = workspace_lint_tool_install_command(root)
        assert cmd is not None
        assert "ruff==0.9.1" in cmd
        assert "mypy==1.14.0" in cmd
        assert "pre-commit==4.0.1" in cmd


def _test_precommit_install_hooks_command() -> None:
    import tempfile

    with tempfile.TemporaryDirectory() as tmp:
        root = Path(tmp)
        assert precommit_install_hooks_command(root) is None
        assert precommit_warm_script_commands(root) == []
        (root / ".pre-commit-config.yaml").write_text("repos: []\n", encoding="utf-8")
        cmds = precommit_warm_script_commands(root)
        assert len(cmds) == 2
        assert "base64 -d" in cmds[0]
        assert cmds[1] == f"bash {PRECOMMIT_WARM_SCRIPT_PATH}"
        body = base64.b64decode(
            shlex.split(cmds[0].split("|")[0].removeprefix("echo ").strip())[0]
        ).decode()
        assert "pre-commit install-hooks" in body
        assert "pip install --no-cache-dir pre-commit" in body
        assert "PRE_COMMIT" not in body
        req_dir = root / "requirements"
        req_dir.mkdir()
        (req_dir / "lint.txt").write_text("pre-commit==4.0.1\n", encoding="utf-8")
        pinned_body = base64.b64decode(
            shlex.split(precommit_warm_script_commands(root)[0].split("|")[0].removeprefix("echo ").strip())[0]
        ).decode()
        assert "pre-commit==4.0.1" in pinned_body
        assert ".venv/bin/pre-commit" in pinned_body


def _test_precommit_pin_from_workspace_pyproject() -> None:
    import tempfile

    with tempfile.TemporaryDirectory() as tmp:
        root = Path(tmp)
        (root / "pyproject.toml").write_text(
            '[project]\nname = "demo"\nversion = "0.1.0"\n'
            "[dependency-groups]\ndev = [\"pre-commit==3.5.0\"]\n",
            encoding="utf-8",
        )
        assert _precommit_pin_from_workspace(root) == "3.5.0"


def _test_uv_sync_dev_command() -> None:
    import tempfile

    with tempfile.TemporaryDirectory() as tmp:
        root = Path(tmp)
        assert uv_sync_dev_command(root) is None
        (root / "uv.lock").write_text("# lock\n", encoding="utf-8")
        cmd = uv_sync_dev_command(root)
        assert cmd is not None
        assert cmd.endswith("uv sync")
        (root / "pyproject.toml").write_text(
            '[project]\nname = "demo"\nversion = "0.1.0"\n',
            encoding="utf-8",
        )
        cmd = uv_sync_dev_command(root)
        assert cmd is not None
        assert "pip install" in cmd and "uv" in cmd
        assert cmd.endswith("uv sync")
        (root / "pyproject.toml").write_text(
            '[project]\nname = "demo"\nversion = "0.1.0"\n'
            "[dependency-groups]\ndev = [\"pytest\"]\n",
            encoding="utf-8",
        )
        cmd = uv_sync_dev_command(root)
        assert cmd is not None
        assert "pip install" in cmd and "uv" in cmd
        assert cmd.endswith("uv sync --group dev")


def _test_uv_pip_build_system_command() -> None:
    import tempfile

    with tempfile.TemporaryDirectory() as tmp:
        root = Path(tmp)
        assert uv_pip_build_system_command(root) is None
        (root / "uv.lock").write_text("# lock\n", encoding="utf-8")
        (root / "pyproject.toml").write_text(
            '[project]\nname = "demo"\nversion = "0.1.0"\n',
            encoding="utf-8",
        )
        assert uv_pip_build_system_command(root) is None
        (root / "pyproject.toml").write_text(
            '[project]\nname = "demo"\nversion = "0.1.0"\n'
            '[build-system]\nrequires = ["setuptools>=69.2", "setuptools-scm[toml]>=8.0"]\n',
            encoding="utf-8",
        )
        cmd = uv_pip_build_system_command(root)
        assert cmd is not None
        assert "uv pip install --python .venv" in cmd
        assert shlex.quote("setuptools-scm[toml]>=8.0") in cmd


def _test_uv_editable_install_command() -> None:
    import tempfile

    with tempfile.TemporaryDirectory() as tmp:
        root = Path(tmp)
        assert uv_editable_install_command(root) is None
        (root / "uv.lock").write_text("# lock\n", encoding="utf-8")
        cmd = uv_editable_install_command(root)
        assert cmd is not None
        assert "uv pip install --python .venv editables" in cmd
        assert "uv pip install --python .venv -e . --no-build-isolation" in cmd


def _test_default_pip_editable_seed_for_offline_sync() -> None:
    """Dockerfile ``pip install -e`` + hatchling ⇒ system pip gets editables at warm."""
    import tempfile

    with tempfile.TemporaryDirectory() as tmp:
        root = Path(tmp)
        dockerfile = root / "Dockerfile"
        dockerfile.write_text(
            'FROM x\nRUN pip install -e ".[diagrams]"\n',
            encoding="utf-8",
        )
        (root / "pyproject.toml").write_text(
            '[project]\nname = "demo"\nversion = "0"\n'
            '[build-system]\nrequires = ["hatchling"]\n'
            'build-backend = "hatchling.build"\n',
            encoding="utf-8",
        )
        seed = default_pip_editable_seed_command(root, dockerfile)
        assert seed is not None
        assert "pip install --no-cache-dir" in seed
        assert "hatchling" in seed
        assert "editables" in seed
        warm = workspace_image_warm_commands(root, dockerfile=dockerfile)
        assert seed in warm
        venv_cmds = verifier_venv_build_system_commands(root)
        assert len(venv_cmds) == 1
        assert "hatchling" in venv_cmds[0]
        assert "editables" in venv_cmds[0]
        # No Dockerfile editables ⇒ no system-pip seed.
        bare = root / "Dockerfile.bare"
        bare.write_text("FROM x\nRUN pip install pytest\n", encoding="utf-8")
        assert default_pip_editable_seed_command(root, bare) is None


def _test_editable_seed_reads_monorepo_build_backends() -> None:
    """Editable targets under libs/*/pyproject.toml must contribute hatchling seeds."""
    import tempfile

    with tempfile.TemporaryDirectory() as tmp:
        root = Path(tmp)
        core = root / "libs" / "core"
        core.mkdir(parents=True)
        (core / "pyproject.toml").write_text(
            '[project]\nname = "langchain-core"\nversion = "0"\n'
            '[build-system]\nrequires = ["hatchling"]\n'
            'build-backend = "hatchling.build"\n',
            encoding="utf-8",
        )
        dockerfile = root / "Dockerfile"
        dockerfile.write_text(
            "FROM x\nRUN pip install -e libs/core\n",
            encoding="utf-8",
        )
        seed = default_pip_editable_seed_command(root, dockerfile)
        assert seed is not None
        assert "hatchling" in seed
        assert "editables" in seed
        specs = _editable_offline_seed_specs(root, dockerfile=dockerfile)
        assert any("hatchling" in s for s in specs)
        empty = DeclaredDeps({}, {}, (), {})
        venv_cmds = verifier_venv_build_system_commands(
            root,
            spec=VerifierSpec(
                declared=empty,
                public_install_specs=(),
                editable_segments=("pip install -e libs/core",),
            ),
        )
        assert venv_cmds and "hatchling" in venv_cmds[0]


def _test_editable_target_project_deps_enter_declared() -> None:
    """``pip install -e libs/core --no-deps`` still needs libs/core's pydantic pin."""
    import tempfile

    with tempfile.TemporaryDirectory() as tmp:
        root = Path(tmp)
        core = root / "libs" / "core"
        core.mkdir(parents=True)
        (core / "pyproject.toml").write_text(
            '[project]\nname = "langchain-core"\nversion = "0"\n'
            'dependencies = ["pydantic>=2.7.4,<3.0.0", "tenacity"]\n'
            '[build-system]\nrequires = ["hatchling"]\n'
            'build-backend = "hatchling.build"\n',
            encoding="utf-8",
        )
        dockerfile = root / "Dockerfile"
        dockerfile.write_text(
            "FROM x\nRUN pip install -e libs/core\n",
            encoding="utf-8",
        )
        declared = declared_python_dependencies(root, dockerfile)
        assert declared.effective_spec("pydantic") == ">=2.7.4,<3.0.0"
        assert "tenacity" in declared.unpinned_names or declared.effective_spec("tenacity")
        specs = _public_install_specs(declared)
        assert any(s.startswith("pydantic") for s in specs)


def _test_uv_offline_smoke_commands() -> None:
    import tempfile

    with tempfile.TemporaryDirectory() as tmp:
        root = Path(tmp)
        assert uv_offline_smoke_commands(root) == []
        (root / "uv.lock").write_text("# lock\n", encoding="utf-8")
        (root / "pyproject.toml").write_text(
            '[project]\nname = "demo"\nversion = "0.1.0"\n',
            encoding="utf-8",
        )
        smoke = uv_offline_smoke_commands(root)
        assert smoke == ["UV_OFFLINE=1 UV_NO_SYNC=1 uv sync --offline"]
        (root / "pyproject.toml").write_text(
            '[project]\nname = "demo"\nversion = "0.1.0"\n'
            "[dependency-groups]\ndev = [\"ruff\"]\n",
            encoding="utf-8",
        )
        smoke = uv_offline_smoke_commands(root)
        assert len(smoke) == 2
        assert smoke[0] == "UV_OFFLINE=1 UV_NO_SYNC=1 uv sync --offline --group dev"
        assert smoke[1].startswith("UV_OFFLINE=1 UV_NO_SYNC=1 uv run ruff check")
        assert "continuing" in smoke[1]


def _test_workspace_declared_repin_command() -> None:
    import tempfile

    with tempfile.TemporaryDirectory() as tmp:
        root = Path(tmp)
        assert workspace_declared_repin_command(root) is None
        dockerfile = root / "Dockerfile"
        dockerfile.write_text(
            "RUN pip install -r requirements/test_extra_new.txt\n",
            encoding="utf-8",
        )
        req_dir = root / "requirements"
        req_dir.mkdir()
        (req_dir / "test_extra_new.txt").write_text(
            "packaging==24.2\npydantic==2.10.3\n",
            encoding="utf-8",
        )
        cmd = workspace_declared_repin_command(root, dockerfile)
        assert cmd is not None
        assert "packaging==24.2" in cmd
        assert "pydantic==2.10.3" in cmd

        # Constraints not in Dockerfile bulk pins must share the same force-reinstall
        # (twine→rich must not float past rich>=10,<15).
        (root / "pyproject.toml").write_text(
            '[project]\nname = "httpx"\nversion = "0"\n'
            'dependencies = ["rich>=10,<15", "httpcore==1.*"]\n',
            encoding="utf-8",
        )
        (req_dir / "test_extra_new.txt").write_text(
            "twine==6.1.0\nmkdocs==1.6.1\n",
            encoding="utf-8",
        )
        cmd = workspace_declared_repin_command(root, dockerfile)
        assert cmd is not None
        assert "twine==6.1.0" in cmd
        assert "rich>=10,<15" in cmd
        assert "httpcore==1.*" in cmd
        assert cmd.count("pip install") == 1


def _test_workspace_image_warm_commands() -> None:
    import tempfile

    with tempfile.TemporaryDirectory() as tmp:
        root = Path(tmp)
        assert workspace_image_warm_commands(root) == []
        (root / ".pre-commit-config.yaml").write_text("repos: []\n", encoding="utf-8")
        precommit_only = workspace_image_warm_commands(root)
        assert len(precommit_only) == 2
        assert "base64 -d" in precommit_only[0]
        assert precommit_only[1] == f"bash {PRECOMMIT_WARM_SCRIPT_PATH}"
        req_dir = root / "requirements"
        req_dir.mkdir()
        (req_dir / "lint.txt").write_text("ruff==0.9.1\nmypy==1.14.0\n", encoding="utf-8")
        (root / "tox.ini").write_text(
            "[testenv:lint]\ncommands =\n  ruff check src/ --fix\n",
            encoding="utf-8",
        )
        lint_warm = workspace_image_warm_commands(root)
        assert any("tox" in cmd for cmd in lint_warm)
        assert any("ruff==0.9.1" in cmd for cmd in lint_warm)
        assert any(
            cmd == "tox run -e lint --notest --skip-missing-interpreters true"
            for cmd in lint_warm
        )
        assert any(".tox/lint/bin/python" in cmd and "pre_commit" in cmd for cmd in lint_warm)
        assert len(lint_warm) == 6
        (root / "justfile").write_text("lint:\n    tox -e lint\n", encoding="utf-8")
        with_just = workspace_image_warm_commands(root)
        assert any("github.com/casey/just/releases" in cmd for cmd in with_just)
        assert len(with_just) == 7
        (root / "uv.lock").write_text("# lock\n", encoding="utf-8")
        (root / "pyproject.toml").write_text(
            '[project]\nname = "demo"\nversion = "0.1.0"\n'
            '[build-system]\nrequires = ["setuptools>=69.2"]\n'
            "[dependency-groups]\ndev = [\"ruff\"]\n",
            encoding="utf-8",
        )
        cmds = workspace_image_warm_commands(root)
        precommit_script = precommit_warm_script_commands(root)
        assert cmds[0] == just_install_command(root)
        assert "tox" in cmds[1]
        assert cmds[2:5] == [
            f"{_UV_BOOTSTRAP_SHELL} && uv sync --group dev",
            (
                f"{_UV_BOOTSTRAP_SHELL} && uv pip install --python {_UV_PROJECT_VENV} "
                f"{shlex.quote('setuptools>=69.2')}"
            ),
            (
                f"{_UV_BOOTSTRAP_SHELL} && "
                f"uv pip install --python {_UV_PROJECT_VENV} editables && "
                f"uv pip install --python {_UV_PROJECT_VENV} "
                "-e . --no-build-isolation"
            ),
        ]
        assert cmds[5:7] == precommit_script
        assert any("tox run -e lint --notest" in cmd for cmd in cmds)
        assert "uv run ruff check" in cmds[-1]
        assert "continuing" in cmds[-1]


def _test_setuptools_extra_requirement_files_not_extra_keys() -> None:
    """Kombu-style extras map to requirements files; extra keys are not PyPI names."""
    with tempfile.TemporaryDirectory() as tmp:
        root = Path(tmp)
        (root / "setup.py").write_text(
            "from setuptools import setup\n"
            "setup(\n"
            "    name='demo',\n"
            "    extras_require={\n"
            "        'msgpack': extras('msgpack.txt'),\n"
            "        'redis': extras('redis.txt'),\n"
            "        'azureservicebus': extras('azureservicebus.txt'),\n"
            "    },\n"
            ")\n",
            encoding="utf-8",
        )
        extras_dir = root / "requirements" / "extras"
        extras_dir.mkdir(parents=True)
        (extras_dir / "msgpack.txt").write_text("msgpack==1.1.2\n", encoding="utf-8")
        (extras_dir / "redis.txt").write_text(
            "redis>=4.5.2,!=4.5.5,<7.1\n",
            encoding="utf-8",
        )
        (extras_dir / "azureservicebus.txt").write_text(
            "azure-servicebus>=7.12.0\n",
            encoding="utf-8",
        )
        dockerfile = root / "Dockerfile"
        dockerfile.write_text(
            'FROM x\nRUN pip install -e ".[msgpack,redis]"\n'
            "RUN pip install -r requirements/test.txt\n",
            encoding="utf-8",
        )
        (root / "requirements").mkdir(exist_ok=True)
        (root / "requirements" / "test.txt").write_text(
            "pytest==9.0.2\n",
            encoding="utf-8",
        )
        (root / "requirements" / "default.txt").write_text(
            "amqp>=5.1.1,<6.0.0\nvine==5.1.0\npackaging\n",
            encoding="utf-8",
        )
        declared = declared_python_dependencies(root, dockerfile)
        names = declared.package_names()
        assert "msgpack" in names
        assert declared.pip_install_spec("msgpack") == "msgpack==1.1.2"
        assert declared.pip_install_spec("redis") == "redis>=4.5.2,!=4.5.5,<7.1"
        assert declared.pip_install_spec("vine") == "vine==5.1.0"
        assert "packaging" in names
        # Extra keys that were not requested must not become install specs.
        assert "azureservicebus" not in names
        # Scraped setup.py must not treat extras_require keys / metadata as packages.
        scraped = _requirement_names_from_setup_py(root / "setup.py")
        assert "msgpack" not in scraped
        assert "redis" not in scraped
        assert "azureservicebus" not in scraped
        assert "demo" not in scraped


def _test_registry_image_cache_bust_commands() -> None:
    import tempfile

    text = """FROM base
RUN pip install --no-cache-dir -e ".[all]" && pip install --no-cache-dir pytest dirty-equals>=0.9.0
"""
    with tempfile.TemporaryDirectory() as tmp:
        dockerfile = Path(tmp) / "Dockerfile"
        dockerfile.write_text(text, encoding="utf-8")
        cmds = registry_image_cache_bust_commands(dockerfile)
    assert len(cmds) >= 4, cmds
    assert cmds[0].startswith("python3 -c") or cmds[0].startswith("pip install")
    joined = " ".join(cmds)
    assert "starlette==1.0.0" in joined
    assert "pydantic==2.13.4" not in joined
    assert cmds[-1] == f"python3 {MANDATORY_PROBE_SCRIPT_PATH}", cmds
    assert "base64 -d" in cmds[-2], cmds


def _test_registry_image_cache_bust_aiomonitor_shape() -> None:
    tasks_root = malvin_repo_root().parent / "deep-swe" / "tasks"
    dockerfile = tasks_root / "aiomonitor-task-snapshots-diff" / "environment" / "Dockerfile"
    workspace = (
        Path.home()
        / ".malvin_home"
        / "deepswe-results"
        / "aiomonitor-task-snapshots-diff"
        / "workspace"
    )
    if not dockerfile.is_file() or not workspace.is_dir():
        return
    declared = declared_python_dependencies(workspace, dockerfile)
    assert "pydantic" in declared.constraints, declared
    assert declared.constraints["pydantic"] == ">=2.0.0", declared
    assert declared.effective_spec("pydantic") == ">=2.0.0", declared
    cmds = registry_image_cache_bust_commands(
        dockerfile, workspace=workspace, registry_pull=True
    )
    joined = " ".join(cmds)
    assert "pydantic" in joined, cmds
    assert "pydantic==2.12.5" not in joined, cmds
    assert "pydantic==2.13.4" not in joined, cmds
    assert "aiohttp==3.10.10" in joined, cmds
    dockerfile_replay = [
        cmd for cmd in cmds if cmd.startswith("pip install") and "pytest==8.3.3" in cmd
    ]
    assert len(dockerfile_replay) == 2, cmds
    assert "aiohttp==3.10.10" in dockerfile_replay[0], cmds


def _test_registry_image_cache_bust_pydantic_v1_legitimate() -> None:
    import tempfile

    with tempfile.TemporaryDirectory() as tmp:
        root = Path(tmp)
        dockerfile = root / "Dockerfile"
        dockerfile.write_text(
            "RUN pip install --no-cache-dir pydantic==1.10.26 pytest\n",
            encoding="utf-8",
        )
        cmds = registry_image_cache_bust_commands(dockerfile, workspace=root)
    joined = " ".join(cmds)
    assert "pydantic==1.10.26" in joined, cmds
    assert "pydantic>=2" not in joined, cmds
    assert cmds[-1] == f"python3 {MANDATORY_PROBE_SCRIPT_PATH}", cmds


def _test_run_post_prep_probes_structured_error() -> None:
    import sys
    import tempfile
    from unittest.mock import patch

    with tempfile.TemporaryDirectory() as tmp:
        workspace = Path(tmp)
        declared = DeclaredDeps({}, {"pydantic": ">=2.0.0"}, (), {})
        mod = sys.modules[__name__]
        with patch.object(
            mod,
            "_run_shell",
            return_value=(1, "pydantic 1.10.26 violates >=2.0.0", False),
        ):
            errors = run_post_prep_probes(
                workspace, declared, task_id="probe-test", phase="runtime probe"
            )
    assert len(errors) == 1, errors
    assert "probe-test" in errors[0]
    assert "pydantic" in errors[0]
    assert "1.10.26" in errors[0]
    assert ">=2.0.0" in errors[0]


def _test_run_post_prep_probes_multi_violation_errors() -> None:
    import sys
    import tempfile
    from unittest.mock import patch

    stderr = (
        "pydantic 2.13.4 violates ==2.12.5; "
        "terminaltables 3.1.0 violates ==3.1.10"
    )
    with tempfile.TemporaryDirectory() as tmp:
        workspace = Path(tmp)
        declared = DeclaredDeps({}, {"pydantic": ">=2.0.0"}, (), {"terminaltables": "3.1.10"})
        mod = sys.modules[__name__]
        with patch.object(mod, "_run_shell", return_value=(1, stderr, False)):
            errors = run_post_prep_probes(
                workspace, declared, task_id="multi-probe", phase="runtime probe"
            )
    assert len(errors) == 2, errors
    assert any("pydantic" in err and "2.13.4" in err for err in errors), errors
    assert any("terminaltables" in err and "3.1.0" in err for err in errors), errors


def _test_run_post_prep_probes_mixed_import_and_violation_errors() -> None:
    import sys
    import tempfile
    from unittest.mock import patch

    stderr = (
        "backports.strenum: import check failed (No module named 'backports'); "
        "pydantic 1.10.26 violates >=2.0.0"
    )
    with tempfile.TemporaryDirectory() as tmp:
        workspace = Path(tmp)
        declared = DeclaredDeps(
            {},
            {"pydantic": ">=2.0.0", "backports.strenum": "==1.3.1"},
            (),
            {},
        )
        mod = sys.modules[__name__]
        with patch.object(mod, "_run_shell", return_value=(1, stderr, False)):
            errors = run_post_prep_probes(
                workspace, declared, task_id="mixed-probe", phase="runtime probe"
            )
    assert len(errors) == 2, errors
    assert any("backports.strenum" in err for err in errors), errors
    assert any("pydantic" in err and "1.10.26" in err for err in errors), errors


def _test_mandatory_probe_accepts_single_char_version_ops() -> None:
    """Constraints like ``>4.6`` / ``<7`` must not become ``==>4.6`` / ``==<7``."""
    import tempfile

    with tempfile.TemporaryDirectory() as tmp:
        workspace = Path(tmp)
        # Installed packages satisfying the constraints.
        (workspace / "pkg_a.py").write_text("__version__ = '4.9.0'\n", encoding="utf-8")
        probe_body = _mandatory_probe_python(
            DeclaredDeps(
                {},
                {"pexpect": ">4.6", "hypothesis": "<7"},
                (),
                {},
            )
        )
        # Monkeypatch metadata to return satisfying versions without real installs.
        wrapped = (
            "import importlib.metadata as _im\n"
            "_orig = _im.version\n"
            "def _fake(name):\n"
            "    if name == 'pexpect':\n"
            "        return '4.9.0'\n"
            "    if name == 'hypothesis':\n"
            "        return '6.156.6'\n"
            "    return _orig(name)\n"
            "_im.version = _fake\n"
            + probe_body
        )
        proc = subprocess.run(
            [sys.executable, "-c", wrapped],
            cwd=str(workspace),
            capture_output=True,
            text=True,
            check=False,
        )
    assert proc.returncode == 0, proc.stderr


def _test_mandatory_probe_strips_pep508_extras_before_specifier() -> None:
    """Remainders like ``[standard]>=0.0.8`` must not become ``==[standard]…``."""
    probe_body = _mandatory_probe_python(
        DeclaredDeps(
            {},
            {
                "fastapi-cli": "[standard]>=0.0.8",
                "uvicorn": "[standard]>=0.12.0",
            },
            (),
            {},
        )
    )
    assert "ver_spec.startswith('[')" in probe_body
    wrapped = (
        "import importlib.metadata as _im\n"
        "def _fake(name):\n"
        "    if name == 'fastapi-cli':\n"
        "        return '0.0.29'\n"
        "    if name == 'uvicorn':\n"
        "        return '0.51.0'\n"
        "    raise _im.PackageNotFoundError(name)\n"
        "_im.version = _fake\n"
        + probe_body
    )
    proc = subprocess.run(
        [sys.executable, "-c", wrapped],
        capture_output=True,
        text=True,
        check=False,
    )
    assert proc.returncode == 0, proc.stderr
    assert "Invalid specifier" not in proc.stderr


def _test_precommit_warm_soft_fails_install_hooks() -> None:
    import tempfile

    with tempfile.TemporaryDirectory() as tmp:
        root = Path(tmp)
        (root / ".pre-commit-config.yaml").write_text("repos: []\n", encoding="utf-8")
        body = _precommit_warm_script_body(root)
    assert "install-hooks" in body
    assert "continuing" in body
    assert "|| echo" in body


def _test_pythonpath_dockerfile_skips_synthetic_editable() -> None:
    import tempfile

    with tempfile.TemporaryDirectory() as tmp:
        root = Path(tmp)
        src = root / "src" / "demo_pkg"
        src.mkdir(parents=True)
        (src / "__init__.py").write_text("", encoding="utf-8")
        (root / "pyproject.toml").write_text(
            "[project]\nname = 'demo-pkg'\nversion = '0.1.0'\n"
            "[build-system]\nrequires = ['setuptools', 'cython']\n"
            "build-backend = 'setuptools.build_meta'\n",
            encoding="utf-8",
        )
        dockerfile = root / "Dockerfile"
        dockerfile.write_text(
            "FROM x\nENV PYTHONPATH=/app/src\n"
            "RUN pip install attrs numpy\n",
            encoding="utf-8",
        )
        declared = declared_python_dependencies(root, dockerfile)
        assert not any("-e" in s for s in declared.editable_segments), declared
        spec = discover_verifier_spec(root, tests_dir=None, dockerfile=dockerfile)
        assert not any("-e" in s for s in spec.editable_segments), spec
        assert any(str(root / "src") == p or p.endswith("/src") for p in spec.grade_pythonpath)
        env = verifier_grade_subprocess_env(spec, base_env={})
        assert "PYTHONPATH" in env
        assert "src" in env["PYTHONPATH"]


def _test_mandatory_probe_fails_on_invalid_version_string() -> None:
    import importlib.util
    import types
    from unittest.mock import patch

    fake_mod = types.ModuleType("badver")
    fake_mod.__version__ = "not-a-version"
    fake_mod.__spec__ = importlib.util.spec_from_loader("badver", loader=None)
    probe_body = _mandatory_probe_python(
        DeclaredDeps({}, {"badver": "==1.0.0"}, (), {})
    )
    with (
        patch.dict(sys.modules, {"badver": fake_mod}),
        patch("importlib.metadata.version", return_value="not-a-version"),
    ):
        try:
            exec(probe_body, {})  # noqa: S102
            raise AssertionError("expected probe to exit 1 on invalid version")
        except SystemExit as exc:
            assert exc.code == 1, f"expected exit 1, got {exc.code}"


def _test_mandatory_probe_prefers_metadata_over_stale_module_version() -> None:
    body = _mandatory_probe_python(
        DeclaredDeps({}, {"terminaltables": "==3.1.10"}, (), {})
    )
    assert "pkg_version(display_name)" in body
    assert body.index("pkg_version(display_name)") < body.index("__version__")


def _test_mandatory_probe_runtime_metadata_wins_over_stale_version() -> None:
    import importlib.util
    import types
    from unittest.mock import patch

    fake_mod = types.ModuleType("terminaltables")
    fake_mod.__version__ = "3.1.0"
    fake_mod.__spec__ = importlib.util.spec_from_loader("terminaltables", loader=None)
    probe_body = _mandatory_probe_python(
        DeclaredDeps({}, {"terminaltables": "==3.1.10"}, (), {})
    )
    with (
        patch.dict(sys.modules, {"terminaltables": fake_mod}),
        patch("importlib.metadata.version", return_value="3.1.10"),
    ):
        try:
            exec(probe_body, {})  # noqa: S102
        except SystemExit as exc:
            assert exc.code in (0, None), f"probe failed with exit {exc.code}"


def _test_effective_spec_prefers_pyproject_constraint_over_lockfile() -> None:
    declared = DeclaredDeps(
        {},
        {"pydantic": ">=2.0.0"},
        (),
        {"pydantic": "2.12.5"},
    )
    assert declared.effective_spec("pydantic") == ">=2.0.0"


def _test_effective_spec_exact_pyproject_beats_lockfile() -> None:
    declared = DeclaredDeps(
        {},
        {"pydantic": "==2.12.5"},
        (),
        {"pydantic": "2.13.4"},
    )
    assert declared.effective_spec("pydantic") == "==2.12.5"


def _test_mandatory_probe_fails_when_version_unknown() -> None:
    import importlib.util
    import types
    from unittest.mock import patch

    fake_mod = types.ModuleType("silentpkg")
    fake_mod.__spec__ = importlib.util.spec_from_loader("silentpkg", loader=None)
    probe_body = _mandatory_probe_python(
        DeclaredDeps({}, {"silentpkg": "==9.9.9"}, (), {})
    )
    with (
        patch.dict(sys.modules, {"silentpkg": fake_mod}),
        patch("importlib.metadata.version", side_effect=Exception("no metadata")),
    ):
        try:
            exec(probe_body, {})  # noqa: S102
            raise AssertionError("expected probe to exit 1 when version unknown")
        except SystemExit as exc:
            assert exc.code == 1, f"expected exit 1, got {exc.code}"


def _test_httpx_drift_probe_script_write_roundtrip() -> None:
    write_cmd = _httpx_drift_probe_script_write_command()
    payload = shlex.split(write_cmd.split("|")[0].removeprefix("echo ").strip())[0]
    assert base64.b64decode(payload).decode() == _httpx_drift_probe_python(), write_cmd


def _test_probe_import_name_phonenumberslite() -> None:
    assert _probe_import_name("phonenumberslite") == "phonenumbers"
    assert _probe_import_name("pydantic-core") == "pydantic_core"
    assert _probe_import_name("pyelftools") == "elftools"
    assert _probe_import_name("pyserial") == "serial"


def _test_mandatory_probe_uses_metadata_before_import() -> None:
    """Distribution metadata satisfies probes when the import root differs from the dist name."""
    declared = DeclaredDeps({}, {"pyelftools": ">=0.32"}, (), {})
    body = _mandatory_probe_python(declared)
    meta_at = body.index("pkg_version(display_name)")
    import_at = body.index("find_spec(import_name)")
    assert meta_at < import_at, body[:400]


def _test_registry_image_cache_bust_reconciles_twice_after_httpx_fix() -> None:
    import tempfile

    with tempfile.TemporaryDirectory() as tmp:
        root = Path(tmp)
        req_dir = root / "requirements"
        req_dir.mkdir()
        (req_dir / "dev.txt").write_text(
            "typing-extensions==4.12.2\nphonenumberslite==8.13.52\n",
            encoding="utf-8",
        )
        dockerfile = root / "Dockerfile"
        dockerfile.write_text("RUN pip install -r requirements/dev.txt\n", encoding="utf-8")
        cmds = registry_image_cache_bust_commands(dockerfile, workspace=root, registry_pull=True)
    reconcile = [cmd for cmd in cmds if "typing-extensions==4.12.2" in cmd]
    assert len(reconcile) == 2, cmds
    assert _httpx_drift_probe_script_write_command() in cmds, cmds


def _test_mandatory_probe_script_commands_builder_safe() -> None:
    declared = DeclaredDeps(
        {"pytest": "8.0.0"},
        {"pydantic": ">=2.0.0", "aioconsole": "==0.8.1"},
        (),
        {},
    )
    cmds = mandatory_probe_script_commands(declared)
    joined = " ".join(cmds)
    assert "checks = [" not in joined, joined
    assert "base64 -d" in joined, joined
    assert cmds[-1] == f"python3 {MANDATORY_PROBE_SCRIPT_PATH}", cmds


def _test_mandatory_probe_script_write_roundtrip() -> None:
    declared = DeclaredDeps({}, {"pydantic": ">=2.0.0", "aiomonitor": "==0.7.1"}, (), {})
    write_cmd = mandatory_probe_script_write_command(declared)
    expected = _mandatory_probe_python(declared)
    payload = shlex.split(write_cmd.split("|")[0].removeprefix("echo ").strip())[0]
    assert base64.b64decode(payload).decode() == expected, write_cmd


def _test_declared_deps_skip_marker_gated_backports() -> None:
    import tempfile

    with tempfile.TemporaryDirectory() as tmp:
        root = Path(tmp)
        (root / "pyproject.toml").write_text(
            '[project]\nname = "x"\ndependencies = [\n'
            '  "pydantic>=2.0.0",\n'
            '  "backports.strenum>=1.2.4; python_version<\'3.11\'",\n'
            "]\n",
            encoding="utf-8",
        )
        declared = declared_python_dependencies(root)
    assert "pydantic" in declared.constraints, declared
    if sys.version_info >= (3, 11):
        assert "backports.strenum" not in declared.constraints, declared
    else:
        assert "backports.strenum" in declared.constraints, declared


def _test_mandatory_probe_no_crash_on_dotted_import_name() -> None:
    import tempfile
    from unittest.mock import patch

    with tempfile.TemporaryDirectory() as tmp:
        workspace = Path(tmp)
        declared = DeclaredDeps(
            {},
            {"backports.strenum": ">=1.2.4", "pydantic": ">=2.0.0"},
            (),
            {},
        )
        mod = sys.modules[__name__]
        with patch.object(
            mod,
            "_run_shell",
            return_value=(1, "backports.strenum: import check failed (No module named 'backports')", False),
        ):
            errors = run_post_prep_probes(
                workspace, declared, task_id="dotted-test", phase="runtime probe"
            )
    assert len(errors) == 1, errors
    assert "dotted-test" in errors[0]
    assert "backports" in errors[0]


def _test_registry_image_cache_bust_adaptix_pydantic_pin() -> None:
    import tempfile

    tasks_root = malvin_repo_root().parent / "deep-swe" / "tasks"
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
        (req_dir / "lint.txt").write_text("pre-commit==4.0.1\n", encoding="utf-8")
        (workspace / ".pre-commit-config.yaml").write_text("repos: []\n", encoding="utf-8")
        cmds = registry_image_cache_bust_commands(dockerfile, workspace=workspace)
        precommit = precommit_warm_script_commands(workspace)
    assert any("pydantic==2.10.3" in c for c in cmds), cmds
    assert any("pydantic-core==2.27.1" in c for c in cmds), cmds
    assert not any("pydantic==2.13.4" in c for c in cmds), cmds
    assert precommit
    pinned_body = base64.b64decode(
        shlex.split(precommit[0].split("|")[0].removeprefix("echo ").strip())[0]
    ).decode()
    assert "pre-commit==4.0.1" in pinned_body


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
    tasks_root = malvin_repo_root().parent / "deep-swe" / "tasks"
    dockerfile = tasks_root / "fastapi-deprecation-response-headers" / "environment" / "Dockerfile"
    if not dockerfile.is_file():
        return
    bulk = dockerfile_bulk_pip_commands(dockerfile)
    assert bulk, bulk
    assert all("pip install" in cmd for cmd in bulk)
    assert all('-e "' not in cmd for cmd in bulk)


def _fixture_verifier_adaptix() -> Path:
    return malvin_repo_root() / "tests" / "fixtures" / "verifier_adaptix"


_VENV_CACHE_ROOT: Path | None = None
_VENV_CACHE: dict[tuple[str, ...], Path] = {}


def _venv_cache_root() -> Path:
    global _VENV_CACHE_ROOT
    if _VENV_CACHE_ROOT is None:
        _VENV_CACHE_ROOT = Path(tempfile.mkdtemp(prefix="malvin-venv-cache-"))
    return _VENV_CACHE_ROOT


def _clone_cached_venv(dest: Path, packages: tuple[str, ...] = ()) -> Path:
    """Copy a process-cached venv (optionally with pip packages) into ``dest``.

    Creating a venv + pip install is multi-second; copytree of a warm cache is
    ~0.2s and keeps unit tests under the 1.5s budget.
    """
    import shutil

    key = packages
    root = _venv_cache_root()
    if key not in _VENV_CACHE:
        base = root / f"base-{abs(hash(key)):x}"
        if not (base / "bin" / "python").is_file():
            subprocess.run(
                [sys.executable, "-m", "venv", str(base)],
                check=True,
                capture_output=True,
            )
            if packages:
                pip = str(base / "bin" / "pip")
                install = subprocess.run(
                    [pip, "install", "--no-cache-dir", *packages],
                    capture_output=True,
                    text=True,
                    check=False,
                )
                assert install.returncode == 0, install.stderr
        _VENV_CACHE[key] = base
    if dest.exists():
        shutil.rmtree(dest)
    shutil.copytree(_VENV_CACHE[key], dest, symlinks=True)
    return dest


def _minimal_venv_dir(dest: Path) -> Path:
    """Create a venv-shaped directory with ``bin/python`` → sys.executable (no real venv)."""
    dest.mkdir(parents=True, exist_ok=True)
    bin_dir = dest / "bin"
    bin_dir.mkdir(exist_ok=True)
    python = bin_dir / "python"
    if not python.exists():
        python.symlink_to(sys.executable)
    return dest


def _test_discover_verifier_spec_public_vs_grade() -> None:
    fixture = _fixture_verifier_adaptix()
    workspace = fixture / "workspace"
    tests_dir = fixture / "tests"
    dockerfile = fixture / "environment" / "Dockerfile"
    public = discover_verifier_spec(workspace, tests_dir=None, dockerfile=dockerfile)
    assert public.harbor_imports == ()
    assert public.grade_closure_install_specs == ()
    assert "typing-extensions==4.12.2" in public.public_install_specs or any(
        s.startswith("typing-extensions") for s in public.public_install_specs
    )
    # Public layer is DeclaredDeps only — never Harbor patch secret tokens.
    public_joined = " ".join(public.public_install_specs)
    assert "NoExtraItems" not in public_joined
    assert "test_aliases" not in public_joined
    assert "test.patch" not in public_joined
    # Declared pins include typeguard from requirements; public specs may list it.
    grade = discover_verifier_spec(workspace, tests_dir=tests_dir, dockerfile=dockerfile)
    assert "pytest" in grade.harbor_imports
    assert "typeguard" in grade.harbor_imports
    assert "typing_extensions" in grade.harbor_imports
    assert "os" not in grade.harbor_imports
    view = public.public_view()
    assert "harbor_imports" not in view
    assert "grade_closure_install_specs" not in view


def _test_verifier_venv_materialize_public_no_patch_only_names() -> None:
    fixture = _fixture_verifier_adaptix()
    workspace = fixture / "workspace"
    tests_dir = fixture / "tests"
    dockerfile = fixture / "environment" / "Dockerfile"
    # Invent a patch-only import absent from declared pins.
    grade = discover_verifier_spec(workspace, tests_dir=tests_dir, dockerfile=dockerfile)
    public = discover_verifier_spec(workspace, tests_dir=None, dockerfile=dockerfile)
    cmds = verifier_venv_materialize_public_commands(public)
    joined = "\n".join(cmds)
    assert VERIFIER_VENV_PATH in joined
    assert "venv" in joined
    # Public install line(s) may only mention public DeclaredDeps pins.
    public_names = {
        s.split("==", 1)[0].split("[", 1)[0].lower() for s in public.public_install_specs
    }
    for line in cmds:
        if "install" not in line or "--upgrade" in line or " -e " in f" {line} ":
            continue
        for token in line.split():
            if "==" not in token:
                continue
            pkg = token.split("==", 1)[0].split("[", 1)[0].lower()
            assert pkg in public_names, (pkg, public_names, line)
    for secret_name in ("NoExtraItems", "test_aliases", "test.patch"):
        assert secret_name not in joined
    # Unmapped names must not be invented as bare pip installs on public path.
    for name in grade.unmapped_imports:
        assert f" {name} " not in f" {joined} "
        assert f"/{name}" not in joined
        assert not any(
            tok == name or tok.startswith(f"{name}==") for tok in joined.split()
        )


def _test_verifier_grade_closure_commands_include_mapped() -> None:
    fixture = _fixture_verifier_adaptix()
    grade = discover_verifier_spec(
        fixture / "workspace",
        tests_dir=fixture / "tests",
        dockerfile=fixture / "environment" / "Dockerfile",
    )
    # Declared-mapped Harbor imports must appear in grade-only closure commands.
    assert grade.grade_closure_install_specs, grade.harbor_imports
    closure_cmds = verifier_venv_apply_grade_closure_commands(grade)
    assert closure_cmds, grade.grade_closure_install_specs
    joined = "\n".join(closure_cmds)
    assert "typeguard" in joined or any(
        "typeguard" in s for s in grade.grade_closure_install_specs
    )
    assert "typing-extensions" in joined or any(
        "typing-extensions" in s for s in grade.grade_closure_install_specs
    )
    public_cmds = "\n".join(verifier_venv_materialize_public_commands(grade))
    assert "typing-extensions" in public_cmds or any(
        "typing-extensions" in s for s in grade.public_install_specs
    )
    assert "harbor_imports" not in public_cmds
    assert "PLUGIN_CONFLICTS" not in public_cmds


def _test_probe_verifier_env_plugin_conflict_reports_verifier_prep() -> None:
    """Missing verifier venv fails closed (no system-Python fallback)."""
    import tempfile

    declared = DeclaredDeps(
        bulk_pins={"typing-extensions": "4.12.2"},
        constraints={},
        editable_segments=(),
        lockfile_pins={},
    )
    spec = VerifierSpec(
        declared=declared,
        public_install_specs=("typing-extensions==4.12.2",),
        editable_segments=(),
        harbor_imports=("typeguard",),
        test_sh_body="python -m pytest -q\n",
        venv_path="/tmp/malvin-verifier-does-not-exist",
    )
    with tempfile.TemporaryDirectory() as tmp:
        ok, err, policy = probe_verifier_env(
            spec,
            workspace=Path(tmp),
            task_id="fixture-adaptix",
            dry_run=False,
            run_collect=False,
        )
    assert ok is False
    assert err is not None
    assert "verifier prep" in err
    assert "missing" in err.lower() or "no system-python" in err.lower()
    assert policy is None


def _test_prepare_verifier_grade_materialize_when_missing() -> None:
    """Missing ``/opt/malvin-verifier``: materialize before probe; fail closed if absent."""
    import sys
    import tempfile
    from unittest.mock import patch

    fixture = _fixture_verifier_adaptix()
    workspace = fixture / "workspace"
    tests_dir = fixture / "tests"
    dockerfile = fixture / "environment" / "Dockerfile"
    mod = sys.modules[__name__]
    real_discover = discover_verifier_spec

    with tempfile.TemporaryDirectory() as tmp:
        venv_path = Path(tmp) / "malvin-verifier"
        calls: list[str] = []

        def discover_with_tmp_venv(
            ws: Path,
            tests_dir: Path | None = None,
            dockerfile: Path | None = None,
        ) -> VerifierSpec:
            spec = real_discover(ws, tests_dir=tests_dir, dockerfile=dockerfile)
            return VerifierSpec(
                declared=spec.declared,
                public_install_specs=spec.public_install_specs,
                editable_segments=spec.editable_segments,
                harbor_imports=spec.harbor_imports,
                grade_closure_install_specs=spec.grade_closure_install_specs,
                unmapped_imports=spec.unmapped_imports,
                test_sh_body=spec.test_sh_body,
                plugin_policy=spec.plugin_policy,
                venv_path=str(venv_path),
            )

        def shell_ok_no_create(
            command: str, _workspace: Path, timeout_sec: float | None = None
        ) -> tuple[int, str, bool]:
            del timeout_sec
            calls.append(command)
            return 0, "", False

        with (
            patch.object(mod, "discover_verifier_spec", side_effect=discover_with_tmp_venv),
            patch.object(mod, "_run_shell", side_effect=shell_ok_no_create),
            patch.object(mod, "probe_verifier_env") as probe,
        ):
            result = prepare_verifier_grade(
                workspace,
                tests_dir=tests_dir,
                dockerfile=dockerfile,
                task_id="mat-missing",
            )
            probe.assert_not_called()
        assert result.ok is False
        assert result.error is not None
        assert "verifier prep" in result.error
        assert "missing" in result.error.lower()
        assert calls, "expected public materialize commands when venv absent"
        assert any(str(venv_path) in c for c in calls)
        assert not (venv_path / "bin" / "python").is_file()

        calls.clear()

        def shell_fail(
            command: str, _workspace: Path, timeout_sec: float | None = None
        ) -> tuple[int, str, bool]:
            del timeout_sec
            calls.append(command)
            return 1, "venv create failed", False

        with (
            patch.object(mod, "discover_verifier_spec", side_effect=discover_with_tmp_venv),
            patch.object(mod, "_run_shell", side_effect=shell_fail),
            patch.object(mod, "probe_verifier_env") as probe,
        ):
            result = prepare_verifier_grade(
                workspace,
                tests_dir=tests_dir,
                dockerfile=dockerfile,
                task_id="mat-fail",
            )
            probe.assert_not_called()
        assert result.ok is False
        assert result.error is not None
        assert "verifier prep" in result.error
        assert calls, "expected materialize attempt before fail-closed"


def _test_probe_verifier_env_unmapped_imports_fail_closed() -> None:
    """Q7: unmapped Harbor imports abort at verifier prep (no invented PyPI pins)."""
    import tempfile

    declared = DeclaredDeps(
        bulk_pins={"pytest": "8.0.0"},
        constraints={},
        editable_segments=(),
        lockfile_pins={},
    )
    with tempfile.TemporaryDirectory() as tmp:
        root = Path(tmp)
        venv = root / "venv"
        _clone_cached_venv(venv)
        spec = VerifierSpec(
            declared=declared,
            public_install_specs=("pytest==8.0.0",),
            editable_segments=(),
            unmapped_imports=("only_in_patch",),
            test_sh_body="python -m pytest -q\n",
            venv_path=str(venv),
        )
        ok, err, _policy = probe_verifier_env(
            spec,
            workspace=root,
            task_id="unmapped",
            dry_run=False,
            run_collect=False,
        )
    assert ok is False
    assert err is not None
    assert "verifier prep" in err
    assert "only_in_patch" in err


def _test_prepare_task_sandbox_does_not_call_probe_verifier() -> None:
    import inspect

    source = inspect.getsource(prepare_task_sandbox)
    assert "probe_verifier_env" not in source
    assert "prepare_verifier_grade" not in source
    assert "discover_verifier_spec" not in source


def _test_probe_verifier_env_missing_collect_path_does_not_abort() -> None:
    """Collect-only against paths absent from disk *and* ``test.patch`` must not abort."""
    declared = DeclaredDeps(
        bulk_pins={"pytest": "8.0.0"},
        constraints={},
        editable_segments=(),
        lockfile_pins={},
    )
    with tempfile.TemporaryDirectory() as tmp:
        root = Path(tmp)
        venv = root / "venv"
        _clone_cached_venv(venv, ("pytest==8.0.0",))
        spec = VerifierSpec(
            declared=declared,
            public_install_specs=("pytest==8.0.0",),
            editable_segments=(),
            test_sh_body="python -m pytest tests/missing_hidden_from_patch.py -q\n",
            venv_path=str(venv),
        )
        ok, err, _policy = probe_verifier_env(
            spec,
            workspace=root,
            task_id="probe-missing-path",
            dry_run=False,
            run_collect=True,
            tests_dir=None,
        )
    assert ok is True, err
    assert err is None


def _test_probe_plugin_conflict_failed_collect_aborts() -> None:
    """PLUGIN_CONFLICTS must not soft-succeed when collect-only still fails."""
    from unittest.mock import MagicMock, patch

    declared = DeclaredDeps(
        bulk_pins={"pytest": "8.0.0"},
        constraints={},
        editable_segments=(),
        lockfile_pins={},
    )
    with tempfile.TemporaryDirectory() as tmp:
        root = Path(tmp)
        venv = root / "malvin-verifier"
        _clone_cached_venv(venv)
        spec = VerifierSpec(
            declared=declared,
            public_install_specs=("pytest==8.0.0",),
            editable_segments=(),
            test_sh_body="python -m pytest tests/test_aliases.py -q\n",
            venv_path=str(venv),
        )
        plugin_out = MagicMock(
            returncode=2,
            stdout=(
                "PLUGIN_OK:\n"
                "PLUGIN_CONFLICTS:typeguard: ImportError: NoExtraItems\n"
            ),
            stderr="",
        )
        collect_out = MagicMock(
            returncode=1,
            stdout="",
            stderr="INTERNALERROR> collection failed for unknown reason\n",
        )

        def fake_run(cmd, **kwargs):  # type: ignore[no-untyped-def]
            if isinstance(cmd, list) and len(cmd) >= 3 and cmd[1] == "-c":
                return plugin_out
            return collect_out

        with patch("subprocess.run", side_effect=fake_run):
            ok, err, policy = probe_verifier_env(
                spec,
                workspace=root,
                task_id="plugin-softpass",
                dry_run=False,
                run_collect=True,
                tests_dir=None,
            )
    assert ok is False, "plugin conflict + failed collect must fail closed"
    assert err is not None and "verifier prep" in err
    assert "INTERNALERROR" in err or "collection failed" in err
    assert policy is not None
    assert policy.disable_autoload is True


def _test_modified_hunk_context_imports_in_verifier_spec() -> None:
    """Modified test.patch hunks must surface context-line third-party imports."""
    with tempfile.TemporaryDirectory() as tmp:
        root = Path(tmp)
        workspace = root / "workspace"
        workspace.mkdir()
        (workspace / "pyproject.toml").write_text(
            "[project]\nname='x'\nversion='0'\n"
            "dependencies=['only-in-context-pkg==1.0.0']\n",
            encoding="utf-8",
        )
        tests = root / "tests"
        tests.mkdir()
        (tests / "test.patch").write_text(
            "diff --git a/tests/test_mod.py b/tests/test_mod.py\n"
            "--- a/tests/test_mod.py\n"
            "+++ b/tests/test_mod.py\n"
            "@@ -1,3 +1,5 @@\n"
            " import only_in_context_pkg\n"
            " def test_a():\n"
            "     assert True\n"
            "+def test_b():\n"
            "+    assert True\n",
            encoding="utf-8",
        )
        (tests / "test.sh").write_text(
            "#!/bin/bash\npython -m pytest tests/test_mod.py -q\n",
            encoding="utf-8",
        )
        grade = discover_verifier_spec(workspace, tests_dir=tests, dockerfile=None)
        public = discover_verifier_spec(workspace, tests_dir=None, dockerfile=None)
    assert "only_in_context_pkg" in grade.harbor_imports
    assert "only_in_context_pkg" not in public.harbor_imports
    assert any("only-in-context-pkg" in s for s in grade.grade_closure_install_specs)
    assert "only_in_context_pkg" in grade.grade_view()["harbor_imports"]
    assert "harbor_imports" not in grade.public_view()
    assert "unmapped_imports" not in grade.public_view()


def _test_adaptix_prepatch_materialize_catches_importerror() -> None:
    """Production Harbor timing: no test file on disk; patch hunks must fail prep."""
    fixture = _fixture_verifier_adaptix()
    dockerfile = fixture / "environment" / "Dockerfile"
    tests_dir = fixture / "tests"
    grade = discover_verifier_spec(
        fixture / "workspace",
        tests_dir=tests_dir,
        dockerfile=dockerfile,
    )
    with tempfile.TemporaryDirectory() as tmp:
        root = Path(tmp)
        venv = root / "malvin-verifier"
        workspace = root / "app"
        workspace.mkdir()
        # Intentionally do NOT write tests/test_aliases.py into workspace (pre-patch).
        _clone_cached_venv(
            venv,
            (
                "typing-extensions==4.12.2",
                "typeguard==4.4.1",
                "pytest==8.3.4",
            ),
        )
        conflict_spec = VerifierSpec(
            declared=grade.declared,
            public_install_specs=grade.public_install_specs,
            editable_segments=(),
            harbor_imports=grade.harbor_imports,
            grade_closure_install_specs=grade.grade_closure_install_specs,
            unmapped_imports=(),
            test_sh_body=grade.test_sh_body
            or "python -m pytest tests/test_aliases.py -q\n",
            venv_path=str(venv),
        )
        ok, err, policy = probe_verifier_env(
            conflict_spec,
            workspace=workspace,
            tests_dir=tests_dir,
            task_id="adaptix-prepatch",
            dry_run=False,
            run_collect=True,
        )
    # Must not soft-succeed: patch hunks are materialized, ImportError must fail prep
    # (or a plugin policy must be produced that grades would apply).
    if ok:
        assert policy is not None
        assert policy.disable_autoload is True
        grade_env = verifier_grade_subprocess_env(
            conflict_spec, plugin_policy=policy
        )
        assert grade_env.get("VIRTUAL_ENV") == str(venv)
    else:
        assert err is not None and "verifier prep" in err
        assert (
            "ImportError" in err
            or "ModuleNotFoundError" in err
            or "NoExtraItems" in err
        )


def _test_verifier_pip_honors_spec_venv_path() -> None:
    declared = DeclaredDeps(
        bulk_pins={"pytest": "8.0.0"},
        constraints={},
        editable_segments=(),
        lockfile_pins={},
    )
    spec = VerifierSpec(
        declared=declared,
        public_install_specs=("pytest==8.0.0",),
        editable_segments=(),
        venv_path="/tmp/custom-malvin-verifier",
    )
    cmds = "\n".join(verifier_venv_materialize_public_commands(spec))
    assert "/tmp/custom-malvin-verifier/bin/pip" in cmds
    assert "/opt/malvin-verifier/bin/pip" not in cmds
    closure = VerifierSpec(
        declared=declared,
        public_install_specs=("pytest==8.0.0",),
        editable_segments=(),
        grade_closure_install_specs=("pytest==8.0.0",),
        venv_path="/tmp/custom-malvin-verifier",
    )
    assert all(
        "/tmp/custom-malvin-verifier/bin/pip" in c
        for c in verifier_venv_apply_grade_closure_commands(closure)
    )


def _test_prepare_verifier_grade_materialize_creates_real_venv() -> None:
    """End-to-end: missing venv → materialize commands produce ``bin/python``.

    ``python -m venv`` / pip upgrade are multi-second; under unit tests those shell
    steps are served from the process venv cache while still driving
    ``prepare_verifier_grade`` through ``_run_shell``.
    """
    from unittest.mock import patch

    fixture = _fixture_verifier_adaptix()
    with tempfile.TemporaryDirectory() as tmp:
        venv_path = Path(tmp) / "malvin-verifier"
        workspace = fixture / "workspace"
        tests_dir = fixture / "tests"
        dockerfile = fixture / "environment" / "Dockerfile"
        mod = sys.modules[__name__]

        def discover_tmp(
            ws: Path,
            tests_dir: Path | None = None,
            dockerfile: Path | None = None,
        ) -> VerifierSpec:
            _ = (ws, tests_dir, dockerfile)
            # Empty public specs keep the e2e light (venv create + pip upgrade only).
            return VerifierSpec(
                declared=DeclaredDeps({}, {}, (), {}),
                public_install_specs=(),
                editable_segments=(),
                harbor_imports=(),
                grade_closure_install_specs=(),
                unmapped_imports=(),
                test_sh_body="python -m pytest -q\n",
                venv_path=str(venv_path),
            )

        def fast_run_shell(
            command: str,
            ws: Path,
            *,
            timeout_sec: float | None = None,
        ) -> tuple[int, str, bool]:
            _ = timeout_sec
            if " -m venv " in command or command.strip().startswith("python3 -m venv"):
                _clone_cached_venv(venv_path)
                return 0, "", False
            if "install --upgrade pip" in command:
                return 0, "", False
            return _run_shell(command, ws)

        with (
            patch.object(mod, "discover_verifier_spec", side_effect=discover_tmp),
            patch.object(
                mod,
                "probe_verifier_env",
                return_value=(True, None, None),
            ),
            patch.object(mod, "_run_shell", side_effect=fast_run_shell),
        ):
            result = prepare_verifier_grade(
                workspace,
                tests_dir=tests_dir,
                dockerfile=dockerfile,
                task_id="mat-e2e",
            )
        assert (venv_path / "bin" / "python").is_file(), result.error
        assert result.ok is True
        assert result.public_venv_present is True


def _test_discover_grade_closure_records_declared_harbor_imports() -> None:
    """Mapped Harbor imports fill grade_closure; unmapped stay out of install commands."""
    import tempfile

    with tempfile.TemporaryDirectory() as tmp:
        root = Path(tmp)
        workspace = root / "ws"
        workspace.mkdir()
        tests_dir = root / "tests"
        tests_dir.mkdir()
        (tests_dir / "test.patch").write_text(
            "diff --git a/t.py b/t.py\n"
            "--- /dev/null\n"
            "+++ b/t.py\n"
            "@@ -0,0 +1,2 @@\n"
            "+import requests\n"
            "+import only_in_patch\n",
            encoding="utf-8",
        )
        dockerfile = root / "Dockerfile"
        dockerfile.write_text(
            "FROM x\nRUN pip install --no-cache-dir requests==2.31.0\n",
            encoding="utf-8",
        )
        grade = discover_verifier_spec(
            workspace, tests_dir=tests_dir, dockerfile=dockerfile
        )
        public = discover_verifier_spec(workspace, tests_dir=None, dockerfile=dockerfile)
    assert any(s.startswith("requests==") for s in grade.grade_closure_install_specs)
    assert "only_in_patch" in grade.unmapped_imports
    assert "only_in_patch" not in "\n".join(
        verifier_venv_apply_grade_closure_commands(grade)
    )
    assert public.grade_closure_install_specs == ()
    assert public.harbor_imports == ()
    assert any(s.startswith("requests==") for s in grade.public_install_specs)


def _test_editable_project_satisfies_harbor_import() -> None:
    """Dockerfile ``pip install -e .`` provides Harbor imports without DeclaredDeps pins."""
    import tempfile

    with tempfile.TemporaryDirectory() as tmp:
        root = Path(tmp)
        workspace = root / "ws"
        workspace.mkdir()
        (workspace / "mypkg").mkdir()
        (workspace / "mypkg" / "__init__.py").write_text("", encoding="utf-8")
        (workspace / "pyproject.toml").write_text(
            '[project]\nname = "my-pkg"\nversion = "0.1.0"\n',
            encoding="utf-8",
        )
        tests_dir = root / "tests"
        tests_dir.mkdir()
        (tests_dir / "test.patch").write_text(
            "diff --git a/t.py b/t.py\n"
            "--- /dev/null\n"
            "+++ b/t.py\n"
            "@@ -0,0 +1,1 @@\n"
            "+import mypkg\n",
            encoding="utf-8",
        )
        dockerfile = root / "Dockerfile"
        dockerfile.write_text(
            "FROM x\nRUN pip install -e .\n",
            encoding="utf-8",
        )
        grade = discover_verifier_spec(
            workspace, tests_dir=tests_dir, dockerfile=dockerfile
        )
    assert "mypkg" in grade.harbor_imports
    assert grade.unmapped_imports == ()
    assert grade.editable_segments
    assert any("-e" in seg for seg in grade.editable_segments)


def _test_probe_editable_roots_prefers_harbor_import_case() -> None:
    """Dist name ``ipython`` must not override Harbor import spelling ``IPython``."""
    import tempfile

    with tempfile.TemporaryDirectory() as tmp:
        root = Path(tmp)
        (root / "IPython").mkdir()
        (root / "IPython" / "__init__.py").write_text("x = 1\n", encoding="utf-8")
        python = sys.executable
        err = _probe_editable_roots_importable(
            python,
            {"ipython", "IPython"},
            workspace=root,
            harbor_imports=("IPython",),
        )
        assert err is None, err


def _test_non_pytest_test_sh_skips_collect_probe() -> None:
    assert not test_sh_invokes_pytest("#!/bin/bash\nbash /app/test.sh base\n")
    assert test_sh_invokes_pytest("python -m pytest tests/ -q\n")


def _test_unpinned_dockerfile_package_declared() -> None:
    """Bare ``pip install pytest`` becomes an unpinned DeclaredDeps name."""
    import tempfile

    with tempfile.TemporaryDirectory() as tmp:
        root = Path(tmp)
        workspace = root / "ws"
        workspace.mkdir()
        (workspace / "pyproject.toml").write_text(
            '[project]\nname = "demo"\nversion = "0.1.0"\n',
            encoding="utf-8",
        )
        dockerfile = root / "Dockerfile"
        dockerfile.write_text(
            "FROM x\nRUN pip install --no-cache-dir pytest\nRUN pip install -e .\n",
            encoding="utf-8",
        )
        declared = declared_python_dependencies(workspace, dockerfile)
        tests_dir = root / "tests"
        tests_dir.mkdir()
        (tests_dir / "test.patch").write_text(
            "diff --git a/t.py b/t.py\n"
            "--- /dev/null\n"
            "+++ b/t.py\n"
            "@@ -0,0 +1,2 @@\n"
            "+import pytest\n"
            "+import demo\n",
            encoding="utf-8",
        )
        grade = discover_verifier_spec(
            workspace, tests_dir=tests_dir, dockerfile=dockerfile
        )
    assert "pytest" in declared.unpinned_names
    assert declared.pip_install_spec("pytest") == "pytest"
    assert "pytest" in grade.public_install_specs
    assert grade.unmapped_imports == ()


def _test_cargo_and_go_mod_skipped_in_offline_sync() -> None:
    """Network language package fetches are not replayed in offline sandbox sync."""
    cargo_runs = parse_dockerfile_run_commands("FROM x\nRUN cargo fetch\n")
    go_runs = parse_dockerfile_run_commands("FROM x\nRUN go mod download\n")
    assert _sync_commands_from_runs(cargo_runs, offline_editable=False) == []
    assert _sync_commands_from_runs(go_runs, offline_editable=False) == []
    assert _sync_commands_from_runs(cargo_runs, offline_editable=True) == []


def _test_collect_import_error_editable_feature_gap() -> None:
    provided = {"pwnlib", "pwn", "pwntools"}
    assert collect_import_error_is_editable_feature_gap(
        "ModuleNotFoundError: No module named 'pwnlib.tubes.mux'",
        provided,
    )
    assert not collect_import_error_is_editable_feature_gap(
        "ModuleNotFoundError: No module named 'pwnlib'",
        provided,
    )
    assert collect_import_error_is_editable_feature_gap(
        "tests/test_mux.py:17: in <module>\n    from pwnlib\n",
        provided,
    )
    # Third-party miss must not soft-succeed just because traceback mentions pwnlib.
    assert not collect_import_error_is_editable_feature_gap(
        "File \"/app/pwnlib/context/__init__.py\", line 21, in <module>\n"
        "    import socks\n"
        "ModuleNotFoundError: No module named 'socks'\n",
        provided,
    )


def _test_bare_pyproject_deps_become_unpinned() -> None:
    import tempfile

    with tempfile.TemporaryDirectory() as tmp:
        root = Path(tmp)
        (root / "pyproject.toml").write_text(
            '[project]\nname = "demo"\nversion = "0.1.0"\n'
            'dependencies = ["pysocks", "requests>=2.0"]\n',
            encoding="utf-8",
        )
        dockerfile = root / "Dockerfile"
        dockerfile.write_text("FROM x\nRUN pip install -e .\n", encoding="utf-8")
        declared = declared_python_dependencies(root, dockerfile)
    assert "pysocks" in declared.unpinned_names
    assert declared.constraints.get("requests") == ">=2.0"
    assert declared.pip_install_spec("pysocks") == "pysocks"



def _test_adaptix_conflict_fixture_yields_plugin_policy_or_verifier_prep() -> None:
    """Adaptix pin conflict: collect ImportError fails verifier prep (or plugin policy)."""
    import tempfile

    fixture = _fixture_verifier_adaptix()
    dockerfile = fixture / "environment" / "Dockerfile"
    grade = discover_verifier_spec(
        fixture / "workspace",
        tests_dir=fixture / "tests",
        dockerfile=dockerfile,
    )
    with tempfile.TemporaryDirectory() as tmp:
        root = Path(tmp)
        venv = root / "malvin-verifier"
        workspace = root / "app"
        tests = workspace / "tests"
        tests.mkdir(parents=True)
        # Harbor hidden test (from fixture patch) needs NoExtraItems; pin is 4.12.2.
        (tests / "test_aliases.py").write_text(
            "from typing_extensions import NoExtraItems\n"
            "import typeguard\n"
            "import pytest\n"
            "def test_smoke():\n"
            "    assert NoExtraItems is not None\n",
            encoding="utf-8",
        )
        _clone_cached_venv(
            venv,
            (
                "typing-extensions==4.12.2",
                "typeguard==4.4.1",
                "pytest==8.3.4",
            ),
        )
        conflict_spec = VerifierSpec(
            declared=grade.declared,
            public_install_specs=grade.public_install_specs,
            editable_segments=(),
            harbor_imports=grade.harbor_imports,
            grade_closure_install_specs=grade.grade_closure_install_specs,
            unmapped_imports=(),
            test_sh_body="python -m pytest tests/test_aliases.py -q\n",
            venv_path=str(venv),
        )
        ok, err, policy = probe_verifier_env(
            conflict_spec,
            workspace=workspace,
            tests_dir=fixture / "tests",
            task_id="adaptix-fixture",
            dry_run=False,
            run_collect=True,
        )
    if ok:
        assert policy is not None
        assert policy.disable_autoload is True
        assert policy.as_env().get("PYTEST_DISABLE_PLUGIN_AUTOLOAD") == "1"
        # Success path must use the verifier venv, never ambient system Python.
        assert conflict_spec.venv_path != ""
        assert Path(f"{conflict_spec.venv_path}/bin/python").is_file()
        grade_env = verifier_grade_subprocess_env(
            conflict_spec, plugin_policy=policy
        )
        assert grade_env.get("VIRTUAL_ENV") == conflict_spec.venv_path
        assert grade_env.get("VIRTUAL_ENV") != sys.prefix
    else:
        assert err is not None and "verifier prep" in err
        assert "ImportError" in err or "ModuleNotFoundError" in err or "NoExtraItems" in err


def _test_adaptix_import_error_never_soft_succeeds_on_system_python() -> None:
    """Adaptix-class ImportError: never ok=True when verifier venv is absent (system Python)."""
    import tempfile

    fixture = _fixture_verifier_adaptix()
    dockerfile = fixture / "environment" / "Dockerfile"
    grade = discover_verifier_spec(
        fixture / "workspace",
        tests_dir=fixture / "tests",
        dockerfile=dockerfile,
    )
    # Point at a missing venv — probe must fail closed, not silently use sys.executable.
    missing = VerifierSpec(
        declared=grade.declared,
        public_install_specs=grade.public_install_specs,
        editable_segments=(),
        harbor_imports=grade.harbor_imports,
        grade_closure_install_specs=grade.grade_closure_install_specs,
        unmapped_imports=(),
        test_sh_body="python -m pytest tests/test_aliases.py -q\n",
        venv_path="/tmp/malvin-verifier-adaptix-missing-venv",
    )
    with tempfile.TemporaryDirectory() as tmp:
        ok, err, policy = probe_verifier_env(
            missing,
            workspace=Path(tmp),
            task_id="adaptix-no-system",
            dry_run=False,
            run_collect=True,
        )
    assert ok is False
    assert err is not None and "verifier prep" in err
    assert "no system-python" in err.lower() or "missing" in err.lower()
    assert policy is None
    # prepare_verifier_grade must also fail closed (materialize cannot create /opt).
    with tempfile.TemporaryDirectory() as tmp:
        venv_path = Path(tmp) / "absent-verifier"
        calls: list[str] = []
        mod = sys.modules[__name__]
        real_discover = discover_verifier_spec

        def discover_missing(
            ws: Path,
            tests_dir: Path | None = None,
            dockerfile: Path | None = None,
        ) -> VerifierSpec:
            spec = real_discover(ws, tests_dir=tests_dir, dockerfile=dockerfile)
            return VerifierSpec(
                declared=spec.declared,
                public_install_specs=spec.public_install_specs,
                editable_segments=spec.editable_segments,
                harbor_imports=spec.harbor_imports,
                grade_closure_install_specs=spec.grade_closure_install_specs,
                unmapped_imports=(),
                test_sh_body=spec.test_sh_body,
                venv_path=str(venv_path),
            )

        def shell_noop(
            command: str, _workspace: Path, timeout_sec: float | None = None
        ) -> tuple[int, str, bool]:
            del timeout_sec
            calls.append(command)
            return 0, "", False

        from unittest.mock import patch

        with (
            patch.object(mod, "discover_verifier_spec", side_effect=discover_missing),
            patch.object(mod, "_run_shell", side_effect=shell_noop),
        ):
            prep = prepare_verifier_grade(
                fixture / "workspace",
                tests_dir=fixture / "tests",
                dockerfile=dockerfile,
                task_id="adaptix-grade-no-system",
            )
    assert prep.ok is False
    assert prep.error is not None and "verifier prep" in prep.error
    assert not (venv_path / "bin" / "python").is_file()
    # Must not claim success with ambient interpreter.
    assert prep.public_venv_present is False


def _test_plugin_policy_as_env_allowlist_wiring() -> None:
    policy = PluginPolicy(disable_autoload=True, allowlist=("xdist", "timeout"))
    env = policy.as_env()
    assert env["PYTEST_DISABLE_PLUGIN_AUTOLOAD"] == "1"
    assert "-p xdist" in env["PYTEST_ADDOPTS"]
    assert "-p timeout" in env["PYTEST_ADDOPTS"]
    assert env["MALVIN_VERIFIER_PLUGIN_ALLOWLIST"] == "xdist,timeout"
    declared = DeclaredDeps({}, {}, (), {})
    spec = VerifierSpec(
        declared=declared,
        public_install_specs=(),
        editable_segments=(),
        plugin_policy=policy,
    )
    grade_env = verifier_grade_subprocess_env(
        spec, base_env={"PATH": "/usr/bin", "PYTEST_ADDOPTS": "-q --maxfail=1"}
    )
    assert grade_env["PYTEST_DISABLE_PLUGIN_AUTOLOAD"] == "1"
    assert "VIRTUAL_ENV" in grade_env
    # Merge must preserve Harbor/base addopts and append allowlist -p tokens.
    assert "-q" in grade_env["PYTEST_ADDOPTS"]
    assert "--maxfail=1" in grade_env["PYTEST_ADDOPTS"]
    assert "-p xdist" in grade_env["PYTEST_ADDOPTS"]
    assert "-p timeout" in grade_env["PYTEST_ADDOPTS"]


def _test_plugin_disable_policy_lets_collect_boot() -> None:
    """Broken pytest11 entry point: disable-autoload policy → collect-only boots."""
    import os
    import tempfile

    with tempfile.TemporaryDirectory() as tmp:
        root = Path(tmp)
        venv = root / "malvin-verifier"
        workspace = root / "app"
        tests = workspace / "tests"
        tests.mkdir(parents=True)
        (tests / "test_smoke.py").write_text(
            "def test_ok():\n    assert True\n",
            encoding="utf-8",
        )
        _clone_cached_venv(venv, ("pytest==8.3.4",))
        # Register a broken pytest11 plugin without a full pip build (keeps call <1.5s).
        site = next((venv / "lib").glob("python*/site-packages"))
        pkg = site / "broken_plug"
        pkg.mkdir(parents=True)
        (pkg / "__init__.py").write_text("", encoding="utf-8")
        (pkg / "plugin.py").write_text(
            "raise ImportError('NoExtraItems is missing from typing_extensions')\n",
            encoding="utf-8",
        )
        dist = site / "broken_plug-0.0.1.dist-info"
        dist.mkdir()
        (dist / "METADATA").write_text(
            "Metadata-Version: 2.1\nName: broken-plug\nVersion: 0.0.1\n",
            encoding="utf-8",
        )
        (dist / "entry_points.txt").write_text(
            "[pytest11]\nbroken = broken_plug.plugin\n",
            encoding="utf-8",
        )
        python = str(venv / "bin" / "python")
        # Autoload must fail collect without policy (broken entry point).
        # Clear parent pytest env so ambient disable-autoload cannot mask the failure.
        bare_env = {
            k: v
            for k, v in os.environ.items()
            if k
            not in (
                "PYTEST_DISABLE_PLUGIN_AUTOLOAD",
                "PYTEST_ADDOPTS",
                "PYTEST_CURRENT_TEST",
            )
        }
        bare = subprocess.run(
            [python, "-m", "pytest", "--collect-only", "-q", "tests/test_smoke.py"],
            cwd=str(workspace),
            capture_output=True,
            text=True,
            check=False,
            env=bare_env,
        )
        assert bare.returncode != 0, bare.stdout + bare.stderr
        declared = DeclaredDeps(
            bulk_pins={"pytest": "8.3.4"},
            constraints={},
            editable_segments=(),
            lockfile_pins={},
        )
        spec = VerifierSpec(
            declared=declared,
            public_install_specs=("pytest==8.3.4",),
            editable_segments=(),
            test_sh_body="python -m pytest tests/test_smoke.py -q\n",
            venv_path=str(venv),
        )
        ok, err, policy = probe_verifier_env(
            spec,
            workspace=workspace,
            task_id="plugin-disable-boot",
            dry_run=False,
            run_collect=True,
        )
    assert ok is True, err
    assert err is None
    assert policy is not None
    assert policy.disable_autoload is True
    assert "broken" not in policy.allowlist
    grade_env = verifier_grade_subprocess_env(spec, plugin_policy=policy)
    assert grade_env.get("PYTEST_DISABLE_PLUGIN_AUTOLOAD") == "1"


def _test_verifier_prep_result_as_dict_excludes_secrets() -> None:
    """Behavioral spy: agent-safe as_dict never carries grade-only VerifierSpec fields."""
    declared = DeclaredDeps(
        bulk_pins={"pytest": "8.3.4"},
        constraints={},
        editable_segments=(),
        lockfile_pins={},
    )
    policy = PluginPolicy(disable_autoload=True, allowlist=("timeout",))
    spec = VerifierSpec(
        declared=declared,
        public_install_specs=("pytest==8.3.4",),
        editable_segments=(),
        harbor_imports=("typeguard", "secret_mod"),
        grade_closure_install_specs=("typeguard==4.4.1",),
        unmapped_imports=("secret_mod",),
        test_sh_body="python -m pytest -q\n",
        plugin_policy=policy,
    )
    result = VerifierPrepResult(
        ok=True, spec=spec, plugin_policy=policy, public_venv_present=True
    )
    payload = result.as_dict()
    dumped = str(payload)
    for forbidden in (
        "harbor_imports",
        "grade_closure",
        "unmapped",
        "plugin_policy",
        "test_sh_body",
        "typeguard",
        "secret_mod",
        "PYTEST_DISABLE",
        "NoExtraItems",
    ):
        assert forbidden not in dumped, dumped
    assert payload["ok"] is True
    assert payload["public_venv_present"] is True
    assert payload["venv_path"] == VERIFIER_VENV_PATH


def _test_leakage_public_view_excludes_patch_only_imports() -> None:
    fixture = _fixture_verifier_adaptix()
    public = discover_verifier_spec(
        fixture / "workspace",
        tests_dir=None,
        dockerfile=fixture / "environment" / "Dockerfile",
    )
    grade = discover_verifier_spec(
        fixture / "workspace",
        tests_dir=fixture / "tests",
        dockerfile=fixture / "environment" / "Dockerfile",
    )
    view = public.public_view()
    for key in (
        "harbor_imports",
        "grade_closure_install_specs",
        "unmapped_imports",
        "plugin_policy",
        "test_sh_body",
    ):
        assert key not in view
    assert "NoExtraItems" not in str(view)
    assert grade.harbor_imports
    assert public.harbor_imports == ()
    # Agent check discovery must not take tests_dir (Modal harbor_agent_image contract).
    modal_src = (Path(__file__).resolve().parent / "deepswe_modal.py").read_text(
        encoding="utf-8"
    )
    assert "discover_deepswe_checks(workspace)" in modal_src
    assert "discover_deepswe_checks(workspace, tests_dir" not in modal_src


def run_self_tests() -> None:
    _test_parse_dockerfile_run_commands_multiline()
    _test_workspace_sync_commands_bandit()
    _test_workspace_sync_commands_fastapi()
    _test_bash_lc_pip_intents_ignore_shell_noise()
    _test_requirement_inline_comments_stripped_for_pip()
    _test_pep508_extras_preserved_in_pip_install_spec()
    _test_requirements_editable_and_constraints_declared()
    _test_poetry_extra_and_runtime_deps_declared()
    _test_fixture_imports_not_unmapped_for_workspace_project()
    _test_editable_pip_segment_ignores_dirty_equals()
    _test_infra_abort_dockerfile_sync_is_offline()
    _test_dockerfile_image_build_commands_fastapi()
    _test_hybrid_poetry_runtime_sync_skipped()
    _test_hybrid_pnpm_runtime_sync_skipped()
    _test_tox_lint_check_commands()
    _test_just_and_tox_runner_install_commands()
    _test_workspace_lint_tool_install_command()
    _test_precommit_install_hooks_command()
    _test_precommit_pin_from_workspace_pyproject()
    _test_uv_sync_dev_command()
    _test_uv_pip_build_system_command()
    _test_uv_editable_install_command()
    _test_default_pip_editable_seed_for_offline_sync()
    _test_editable_seed_reads_monorepo_build_backends()
    _test_editable_target_project_deps_enter_declared()
    _test_uv_offline_smoke_commands()
    _test_setuptools_extra_requirement_files_not_extra_keys()
    _test_workspace_declared_repin_command()
    _test_workspace_image_warm_commands()
    _test_registry_image_cache_bust_commands()
    _test_registry_image_cache_bust_aiomonitor_shape()
    _test_registry_image_cache_bust_pydantic_v1_legitimate()
    _test_declared_deps_skip_marker_gated_backports()
    _test_mandatory_probe_no_crash_on_dotted_import_name()
    _test_run_post_prep_probes_structured_error()
    _test_run_post_prep_probes_multi_violation_errors()
    _test_run_post_prep_probes_mixed_import_and_violation_errors()
    _test_mandatory_probe_prefers_metadata_over_stale_module_version()
    _test_mandatory_probe_runtime_metadata_wins_over_stale_version()
    _test_mandatory_probe_fails_on_invalid_version_string()
    _test_mandatory_probe_accepts_single_char_version_ops()
    _test_mandatory_probe_strips_pep508_extras_before_specifier()
    _test_precommit_warm_soft_fails_install_hooks()
    _test_pythonpath_dockerfile_skips_synthetic_editable()
    _test_effective_spec_prefers_pyproject_constraint_over_lockfile()
    _test_effective_spec_exact_pyproject_beats_lockfile()
    _test_mandatory_probe_fails_when_version_unknown()
    _test_httpx_drift_probe_script_write_roundtrip()
    _test_probe_import_name_phonenumberslite()
    _test_mandatory_probe_uses_metadata_before_import()
    _test_registry_image_cache_bust_reconciles_twice_after_httpx_fix()
    _test_mandatory_probe_script_commands_builder_safe()
    _test_mandatory_probe_script_write_roundtrip()
    _test_registry_image_cache_bust_adaptix_pydantic_pin()
    _test_pydantic_pins_for_cache_bust_reads_requirements()
    _test_collect_pip_install_intents_bash_lc()
    _test_dockerfile_bulk_pip_commands_fastapi()
    _test_workspace_sync_commands_fastapi_task_dockerfile()
    _test_should_replay_skips_apt_and_git()
    _test_discover_verifier_spec_public_vs_grade()
    _test_verifier_venv_materialize_public_no_patch_only_names()
    _test_verifier_grade_closure_commands_include_mapped()
    _test_probe_verifier_env_plugin_conflict_reports_verifier_prep()
    _test_prepare_verifier_grade_materialize_when_missing()
    _test_probe_verifier_env_unmapped_imports_fail_closed()
    _test_prepare_task_sandbox_does_not_call_probe_verifier()
    _test_probe_verifier_env_missing_collect_path_does_not_abort()
    _test_probe_plugin_conflict_failed_collect_aborts()
    _test_modified_hunk_context_imports_in_verifier_spec()
    _test_adaptix_prepatch_materialize_catches_importerror()
    _test_verifier_pip_honors_spec_venv_path()
    _test_prepare_verifier_grade_materialize_creates_real_venv()
    _test_discover_grade_closure_records_declared_harbor_imports()
    _test_editable_project_satisfies_harbor_import()
    _test_probe_editable_roots_prefers_harbor_import_case()
    _test_non_pytest_test_sh_skips_collect_probe()
    _test_unpinned_dockerfile_package_declared()
    _test_cargo_and_go_mod_skipped_in_offline_sync()
    _test_collect_import_error_editable_feature_gap()
    _test_bare_pyproject_deps_become_unpinned()
    _test_adaptix_conflict_fixture_yields_plugin_policy_or_verifier_prep()
    _test_adaptix_import_error_never_soft_succeeds_on_system_python()
    _test_plugin_policy_as_env_allowlist_wiring()
    _test_plugin_disable_policy_lets_collect_boot()
    _test_verifier_prep_result_as_dict_excludes_secrets()
    _test_leakage_public_view_excludes_patch_only_imports()
    click.echo("sandbox_prep self-tests passed")


if __name__ == "__main__":
    run_self_tests()
