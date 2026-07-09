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

import re
import shlex
import subprocess
import sys
import time
from dataclasses import dataclass
from pathlib import Path
from typing import Any

import click

try:
    import tomllib
except ModuleNotFoundError:  # pragma: no cover - py310
    import tomli as tomllib  # type: ignore[no-redef]

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
    r"^([A-Za-z0-9][\w.-]*)\s*([^;]*?)(?:\s*;\s*.*)?$"
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

    def package_names(self) -> set[str]:
        keys = set(self.bulk_pins) | set(self.constraints) | set(self.lockfile_pins)
        return {name.lower() for name in keys}

    def effective_spec(self, name: str) -> str | None:
        key = name.lower()
        if key in self.bulk_pins:
            return f"=={self.bulk_pins[key]}"
        if key in self.lockfile_pins:
            return f"=={self.lockfile_pins[key]}"
        return self.constraints.get(key)

    def pip_install_spec(self, name: str) -> str | None:
        """Return a pip package argument for *name*, or None when not declared."""
        key = name.lower()
        spec = self.effective_spec(key)
        if spec is None:
            return None
        if spec.startswith(("==", ">=", "<=", "!=", ">", "<")):
            return f"{key}{spec}"
        return f"{key}=={spec}"


def _normalize_package_name(name: str) -> str:
    return name.lower().replace("_", "-")


def _parse_dependency_spec(raw: str) -> tuple[str, str] | None:
    line = raw.strip()
    if not line or line.startswith("#"):
        return None
    match = _DEP_SPEC_RE.match(line)
    if not match:
        return None
    name = _normalize_package_name(match.group(1))
    spec = match.group(2).strip()
    if not spec:
        return None
    return name, spec


def _split_pyproject_dependency(raw: str) -> tuple[str, str, str | None] | None:
    """Return ``(name, spec, marker)`` from one PEP 508 dependency string."""
    line = raw.strip()
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


def _read_pyproject_dependencies(pyproject: Path) -> dict[str, str]:
    if not pyproject.is_file():
        return {}
    raw = tomllib.loads(pyproject.read_text(encoding="utf-8"))
    constraints: dict[str, str] = {}
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
        constraints[name] = spec
    return constraints


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


def declared_python_dependencies(
    workspace: Path,
    dockerfile: Path | None = None,
) -> DeclaredDeps:
    """Collect declared Python deps from Dockerfile pins, pyproject, and uv.lock."""
    workspace = workspace.resolve()
    dockerfile_text = dockerfile.read_text(encoding="utf-8") if dockerfile and dockerfile.is_file() else ""
    intents = collect_pip_install_intents(dockerfile_text) if dockerfile_text else []
    bulk_pins = collect_pinned_packages(workspace, intents) if intents else {}
    constraints = _read_pyproject_dependencies(workspace / "pyproject.toml")
    for key in bulk_pins:
        constraints.pop(key, None)
    lockfile_pins = _read_uv_lock_pins(
        workspace / "uv.lock",
        {name.lower() for name in constraints} | set(bulk_pins),
    )
    editable_segments = _editable_segments_from_dockerfile(dockerfile_text) if dockerfile_text else ()
    return DeclaredDeps(
        bulk_pins=bulk_pins,
        constraints=constraints,
        editable_segments=editable_segments,
        lockfile_pins=lockfile_pins,
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
        import_name = name.replace("-", "_")
        checks.append((import_name, spec, name))
        seen.add(name)
    return checks


def _mandatory_probe_python(declared: DeclaredDeps) -> str:
    """Python source run by image-build and runtime verification probes."""
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
        "    try:\n"
        "        spec = importlib.util.find_spec(import_name)\n"
        "    except (ImportError, ModuleNotFoundError, ValueError) as exc:\n"
        "        errors.append(f'{display_name}: import check failed ({exc})')\n"
        "        continue\n"
        "    if spec is None:\n"
        "        errors.append(f'{display_name}: not installed (expected {spec_str})')\n"
        "        continue\n"
        "    mod = importlib.import_module(import_name)\n"
        "    version = getattr(mod, '__version__', None)\n"
        "    if version is None:\n"
        "        try:\n"
        "            from importlib.metadata import version as pkg_version\n"
        "            version = pkg_version(display_name)\n"
        "        except Exception:\n"
        "            version = None\n"
        "    if version is None:\n"
        "        continue\n"
        "    try:\n"
        "        from packaging.specifiers import SpecifierSet\n"
        "        from packaging.version import Version\n"
        "        normalized = spec_str if spec_str[:2] in ('==', '>=', '<=', '!=', '>', '<') else f'=={spec_str}'\n"
        "        if Version(version) not in SpecifierSet(normalized):\n"
        "            errors.append(f'{display_name} {version} violates {spec_str}')\n"
        "    except Exception:\n"
        "        if display_name == 'pydantic' and spec_str.startswith('>=2'):\n"
        "            if str(version).startswith('1.'):\n"
        "                errors.append(f'pydantic {version} violates {spec_str}')\n"
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


_HTTPX_DRIFT_FIX = "'starlette==1.0.0' 'click==8.3.1' 'typer==0.25.1'"


def _httpx_drift_probe_command() -> str:
    return (
        "python3 -c "
        "'import importlib.util, sys; "
        "s=importlib.util.find_spec(chr(104)+chr(116)+chr(116)+chr(112)+chr(120)); "
        "exec(\"import httpx\\nif httpx.__name__ != \\\"httpx\\\": sys.exit(1)\") if s else None'"
    )


_PROBE_VIOLATION_RE = re.compile(
    r"(?P<package>[a-zA-Z0-9][\w.-]*)\s+(?P<observed>[\d.]+)\s+violates\s+(?P<expected>.+)"
)


def _parse_probe_stderr(detail: str) -> tuple[str | None, str | None, str | None]:
    """Return ``(package, observed, expected)`` parsed from mandatory-probe stderr."""
    for fragment in detail.replace(";", "\n").splitlines():
        fragment = fragment.strip()
        if not fragment:
            continue
        match = _PROBE_VIOLATION_RE.search(fragment)
        if match:
            return match.group("package"), match.group("observed"), match.group("expected")
        if "namespace drift" in fragment:
            return "httpx", fragment, "httpx"
        if fragment.endswith(")") and "not installed" in fragment:
            pkg_match = re.match(r"(\S+):", fragment)
            if pkg_match:
                return pkg_match.group(1), "not installed", fragment
    return None, None, None


def _reconcile_declared_deps_commands(declared: DeclaredDeps) -> list[str]:
    """Force-reinstall declared pins and pyproject/lockfile packages at image build."""
    cmds: list[str] = []
    if declared.bulk_pins:
        pkg_args = [f"'{name}=={ver}'" for name, ver in sorted(declared.bulk_pins.items())]
        cmds.append("pip install --no-cache-dir --force-reinstall " + " ".join(pkg_args))
    extras: list[str] = []
    covered = {name.lower() for name in declared.bulk_pins}
    for name in sorted(set(declared.constraints) | set(declared.lockfile_pins)):
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
    package, observed, expected = _parse_probe_stderr(observed_text)
    return [
        format_prep_error(
            task_id,
            phase=phase,
            package=package,
            observed=observed,
            expected=expected,
            detail=observed_text if package is None else None,
            hint="check registry cache bust / pyproject.toml reconcile",
        )
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
        "import pydantic; "
        "sys.exit(1 if pydantic.__version__.startswith('1.') else 0)"
        "\" 2>/dev/null || "
        "pip install --no-cache-dir 'pydantic>=2,<3'"
    )


def registry_image_cache_bust_commands(
    dockerfile: Path | None = None,
    workspace: Path | None = None,
) -> list[str]:
    """Modal registry cache bust: reconcile declared deps, drift fixes, mandatory probe.

    When ``pyproject.toml`` declares packages omitted from Dockerfile bulk pins (e.g.
    aiomonitor ``pydantic>=2.0.0``), reconcile commands install them unconditionally
    after bulk pin replay — not only when bulk pins are absent.
    """
    declared = (
        declared_python_dependencies(workspace.resolve(), dockerfile)
        if workspace is not None and dockerfile is not None and dockerfile.is_file()
        else DeclaredDeps({}, {}, (), {})
    )
    cmds: list[str] = []
    cmds.extend(_reconcile_declared_deps_commands(declared))
    if not declared.package_names():
        cmds.append(_pydantic_v1_eviction_command())
    cmds.append(
        f"{_httpx_drift_probe_command()} || "
        f"pip install --no-cache-dir --force-reinstall {_HTTPX_DRIFT_FIX}"
    )
    cmds.append(_mandatory_probe_command(declared))
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
    tasks_root = Path(__file__).resolve().parent.parent.parent / "deep-swe" / "tasks"
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
    tasks_root = Path(__file__).resolve().parent.parent.parent / "deep-swe" / "tasks"
    dockerfile = tasks_root / "textual-kitty-key-phases" / "environment" / "Dockerfile"
    if not dockerfile.is_file():
        return
    sync = workspace_sync_commands_from_dockerfile(dockerfile)
    assert sync == [], sync


def _test_hybrid_pnpm_runtime_sync_skipped() -> None:
    tasks_root = Path(__file__).resolve().parent.parent.parent / "deep-swe" / "tasks"
    dockerfile = tasks_root / "koota-entity-snapshot-rollback" / "environment" / "Dockerfile"
    if not dockerfile.is_file():
        return
    sync = workspace_sync_commands_from_dockerfile(dockerfile)
    assert sync == [], sync


def _test_registry_image_cache_bust_commands() -> None:
    import tempfile

    text = """FROM base
RUN pip install --no-cache-dir -e ".[all]" && pip install --no-cache-dir pytest dirty-equals>=0.9.0
"""
    with tempfile.TemporaryDirectory() as tmp:
        dockerfile = Path(tmp) / "Dockerfile"
        dockerfile.write_text(text, encoding="utf-8")
        cmds = registry_image_cache_bust_commands(dockerfile)
    assert len(cmds) >= 3, cmds
    assert cmds[0].startswith("python3 -c") or cmds[0].startswith("pip install")
    joined = " ".join(cmds)
    assert "starlette==1.0.0" in joined
    assert "pydantic==2.13.4" not in joined
    assert cmds[-1].startswith("python3 -c")


def _test_registry_image_cache_bust_aiomonitor_shape() -> None:
    tasks_root = Path(__file__).resolve().parent.parent.parent / "deep-swe" / "tasks"
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
    cmds = registry_image_cache_bust_commands(dockerfile, workspace=workspace)
    joined = " ".join(cmds)
    assert "pydantic" in joined, cmds
    assert "pydantic==2.13.4" not in joined, cmds


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
    assert cmds[-1].startswith("python3 -c"), cmds


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
    _test_hybrid_poetry_runtime_sync_skipped()
    _test_hybrid_pnpm_runtime_sync_skipped()
    _test_registry_image_cache_bust_commands()
    _test_registry_image_cache_bust_aiomonitor_shape()
    _test_registry_image_cache_bust_pydantic_v1_legitimate()
    _test_declared_deps_skip_marker_gated_backports()
    _test_mandatory_probe_no_crash_on_dotted_import_name()
    _test_run_post_prep_probes_structured_error()
    _test_registry_image_cache_bust_adaptix_pydantic_pin()
    _test_pydantic_pins_for_cache_bust_reads_requirements()
    _test_collect_pip_install_intents_bash_lc()
    _test_dockerfile_bulk_pip_commands_fastapi()
    _test_workspace_sync_commands_fastapi_task_dockerfile()
    _test_should_replay_skips_apt_and_git()
    click.echo("sandbox_prep self-tests passed")


if __name__ == "__main__":
    run_self_tests()
