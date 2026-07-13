"""Tox gate discovery, warm commands, and offline flag rewriting for DeepSWE."""

from __future__ import annotations

import re
import shlex
from pathlib import Path

_TOX_GATE_ENV_RE = re.compile(
    r"^(?:pep8|format|lint|linters|typecheck|mypy|ruff|style|check|tests?|unit|"
    r"py\d{2,3}|py3\d{1,2})$",
    re.I,
)
_TOX_ENVLIST_RE = re.compile(r"^envlist\s*=\s*(.+)$", re.I | re.M)
_TOX_TESTENV_HEADER_RE = re.compile(r"^\[testenv:([^\]]+)\]\s*$", re.M)
_TOX_SKIP_FLAGS = ("--skip-env-install", "--skip-pkg-install")
_TOX_SKIP_MISSING_INTERPRETERS = ("--skip-missing-interpreters", "true")


def _expand_tox_envlist_token(token: str) -> list[str]:
    """Expand a single tox envlist factor, including simple ``{a,b}`` braces."""
    stripped = token.strip()
    if not stripped:
        return []
    match = re.match(r"^(.*)\{([^}]+)\}(.*)$", stripped)
    if not match:
        return [stripped]
    prefix, body, suffix = match.groups()
    return [f"{prefix}{part.strip()}{suffix}" for part in body.split(",") if part.strip()]


def _split_tox_envlist(raw: str) -> list[str]:
    """Split an envlist value on commas that are outside ``{…}`` factors."""
    tokens: list[str] = []
    buf: list[str] = []
    depth = 0
    for char in raw:
        if char == "{":
            depth += 1
            buf.append(char)
        elif char == "}":
            depth = max(0, depth - 1)
            buf.append(char)
        elif char == "," and depth == 0:
            token = "".join(buf).strip()
            if token:
                tokens.append(token)
            buf = []
        else:
            buf.append(char)
    token = "".join(buf).strip()
    if token:
        tokens.append(token)
    return tokens


def is_tox_invocation(command: str) -> bool:
    """True when *command* invokes the tox CLI (direct or ``python -m tox``)."""
    tokens = command.split()
    if not tokens:
        return False
    if tokens[0] == "tox":
        return True
    return (
        len(tokens) >= 3
        and tokens[0] in {"python", "python3"}
        and tokens[1] == "-m"
        and tokens[2] == "tox"
    )


def ensure_tox_skip_missing_interpreters(command: str) -> str:
    """Append ``--skip-missing-interpreters true`` when *command* invokes tox.

    Harbor images often ship a single Python while upstream ``tox.ini`` lists
    factor envs (``py310``, …). Failing closed on a missing interpreter aborts
    image warm and init-checks even when other gate envs are fine.
    """
    if not is_tox_invocation(command):
        return command
    parts = command.split()
    if "--skip-missing-interpreters" not in parts and "-s" not in parts:
        parts.extend(_TOX_SKIP_MISSING_INTERPRETERS)
    return " ".join(parts)


def ensure_tox_offline_skip_flags(command: str) -> str:
    """Append tox offline skip flags when *command* invokes tox."""
    if not is_tox_invocation(command):
        return command
    parts = ensure_tox_skip_missing_interpreters(command).split()
    for flag in _TOX_SKIP_FLAGS:
        if flag not in parts:
            parts.append(flag)
    return " ".join(parts)


def tox_gate_env_names(workspace: Path) -> list[str]:
    """Return gate-like tox env names from ``envlist`` and ``[testenv:…]`` headers."""
    tox_path = workspace / "tox.ini"
    if not tox_path.is_file():
        return []
    tox_text = tox_path.read_text(encoding="utf-8")
    names: list[str] = []
    seen: set[str] = set()

    def add(name: str) -> None:
        key = name.lower()
        if key in seen or not _TOX_GATE_ENV_RE.match(name):
            return
        seen.add(key)
        names.append(name)

    envlist_match = _TOX_ENVLIST_RE.search(tox_text)
    if envlist_match:
        for raw in _split_tox_envlist(envlist_match.group(1)):
            for expanded in _expand_tox_envlist_token(raw):
                add(expanded)
    for match in _TOX_TESTENV_HEADER_RE.finditer(tox_text):
        add(match.group(1).strip())
    return names


def tox_cpython_factor_executable(name: str) -> str | None:
    """Map tox factor env ``py310`` / ``py39`` / ``py27`` to ``python3.10`` / ``python3.9`` / ``python2.7``.

    Returns ``None`` for non-factor gate names (``pep8``, ``format``, ``pypy3``, …).
    """
    match = re.fullmatch(r"py([23])(\d{1,2})", name.strip(), flags=re.IGNORECASE)
    if match is None:
        return None
    return f"python{match.group(1)}.{int(match.group(2))}"


def tox_gate_check_commands(workspace: Path) -> list[str]:
    """Return offline-safe ``tox run -e …`` lines for gate environments.

    Factor envs (``py310``, …) are wrapped in an in-container interpreter probe so
    ``source .malvin/checks`` under ``set -e`` continues when Harbor lacks that
    Python. Tox's ``--skip-missing-interpreters`` alone still exits non-zero when
    the only selected env is skipped.
    """
    commands: list[str] = []
    for name in tox_gate_env_names(workspace):
        cmd = ensure_tox_offline_skip_flags(f"tox run -e {name}")
        exe = tox_cpython_factor_executable(name)
        if exe is None:
            commands.append(cmd)
            continue
        commands.append(
            f"if command -v {shlex.quote(exe)} >/dev/null 2>&1; then {cmd}; fi"
        )
    return commands


def tox_gate_env_warm_command(workspace: Path) -> str | None:
    """Pre-create gate tox envs at image build for offline ``--skip-env-install`` runs."""
    names = tox_gate_env_names(workspace)
    if not names:
        return None
    return ensure_tox_skip_missing_interpreters(
        f"tox run -e {','.join(names)} --notest"
    )


def _test_tox_gate_check_commands_offline_flags() -> None:
    import tempfile

    with tempfile.TemporaryDirectory() as tmp:
        root = Path(tmp)
        assert tox_gate_check_commands(root) == []
        (root / "tox.ini").write_text(
            "[tox]\nenvlist = py3{10,12},pep8,docs\n"
            "[testenv:format]\ncommands = pre-commit run --all-files\n"
            "[testenv:docs]\ncommands = sphinx-build doc doc/build\n",
            encoding="utf-8",
        )
        assert tox_gate_env_names(root) == ["py310", "py312", "pep8", "format"]
        assert tox_cpython_factor_executable("py310") == "python3.10"
        assert tox_cpython_factor_executable("py27") == "python2.7"
        assert tox_cpython_factor_executable("pep8") is None
        assert tox_cpython_factor_executable("pypy3") is None
        assert tox_gate_check_commands(root) == [
            (
                "if command -v python3.10 >/dev/null 2>&1; then "
                "tox run -e py310 --skip-missing-interpreters true "
                "--skip-env-install --skip-pkg-install; fi"
            ),
            (
                "if command -v python3.12 >/dev/null 2>&1; then "
                "tox run -e py312 --skip-missing-interpreters true "
                "--skip-env-install --skip-pkg-install; fi"
            ),
            (
                "tox run -e pep8 --skip-missing-interpreters true "
                "--skip-env-install --skip-pkg-install"
            ),
            (
                "tox run -e format --skip-missing-interpreters true "
                "--skip-env-install --skip-pkg-install"
            ),
        ]
        assert tox_gate_env_warm_command(root) == (
            "tox run -e py310,py312,pep8,format --notest "
            "--skip-missing-interpreters true"
        )
        assert ensure_tox_offline_skip_flags("tox run -e pep8") == (
            "tox run -e pep8 --skip-missing-interpreters true "
            "--skip-env-install --skip-pkg-install"
        )
        assert ensure_tox_offline_skip_flags(
            "tox run -e pep8 --skip-missing-interpreters true "
            "--skip-env-install --skip-pkg-install"
        ) == (
            "tox run -e pep8 --skip-missing-interpreters true "
            "--skip-env-install --skip-pkg-install"
        )
        assert ensure_tox_offline_skip_flags("ruff check .") == "ruff check ."
        assert ensure_tox_skip_missing_interpreters("tox run -e pep8 --notest") == (
            "tox run -e pep8 --notest --skip-missing-interpreters true"
        )
