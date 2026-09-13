
from __future__ import annotations

import re
import shlex
from pathlib import Path

_TOX_GATE_ENV_RE = re.compile(
    r"^(?:pep8|format|lint|linters|ruff|style|check|tests?|unit|"
    r"py\d{2,3}|py3\d{1,2})$",
    re.I,
)
_TOX_ENVLIST_RE = re.compile(r"^envlist\s*=\s*(.+)$", re.I | re.M)
_TOX_TESTENV_HEADER_RE = re.compile(r"^\[testenv:([^\]]+)\]\s*$", re.M)
_TOX_SKIP_FLAGS = ("--skip-env-install", "--skip-pkg-install")
_TOX_SKIP_MISSING_INTERPRETERS = ("--skip-missing-interpreters", "true")

MIN_TOX_FOR_SKIP_ENV_INSTALL = "4.42.0"

def _tox_version_key(version: str) -> tuple[int, ...]:
    parts: list[int] = []
    for piece in re.split(r"[^\d]+", version.strip()):
        if piece.isdigit():
            parts.append(int(piece))
    return tuple(parts) if parts else (0,)

def clamp_tox_version(version: str | None = None) -> str:
    floor = MIN_TOX_FOR_SKIP_ENV_INSTALL
    if version is None or not str(version).strip():
        return floor
    pinned = str(version).strip()
    if _tox_version_key(pinned) >= _tox_version_key(floor):
        return pinned
    return floor

def image_build_pip_install_command(packages: str) -> str:
    pkgs = packages.strip()
    if not pkgs:
        raise ValueError("packages must be non-empty")
    return (
        "if [ -x /opt/venv/bin/python ]; then "
        f"/opt/venv/bin/python -m pip install --no-cache-dir {pkgs}; "
        "else "
        f"python3 -m pip install --no-cache-dir --break-system-packages {pkgs}; "
        "fi"
    )

def _expand_tox_envlist_token(token: str) -> list[str]:
    stripped = token.strip()
    if not stripped:
        return []
    match = re.match(r"^(.*)\{([^}]+)\}(.*)$", stripped)
    if not match:
        return [stripped]
    prefix, body, suffix = match.groups()
    return [f"{prefix}{part.strip()}{suffix}" for part in body.split(",") if part.strip()]

def _split_tox_envlist(raw: str) -> list[str]:
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
    if not is_tox_invocation(command):
        return command
    parts = command.split()
    if "--skip-missing-interpreters" not in parts and "-s" not in parts:
        parts.extend(_TOX_SKIP_MISSING_INTERPRETERS)
    return " ".join(parts)

def ensure_tox_offline_skip_flags(command: str) -> str:
    if not is_tox_invocation(command):
        return command
    parts = ensure_tox_skip_missing_interpreters(command).split()
    for flag in _TOX_SKIP_FLAGS:
        if flag not in parts:
            parts.append(flag)
    return " ".join(parts)

def tox_gate_env_names(workspace: Path) -> list[str]:
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
    match = re.fullmatch(r"py([23])(\d{1,2})", name.strip(), flags=re.IGNORECASE)
    if match is None:
        return None
    return f"python{match.group(1)}.{int(match.group(2))}"

def tox_gate_check_commands(workspace: Path) -> list[str]:
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
    names = tox_gate_env_names(workspace)
    if not names:
        return None
    return ensure_tox_skip_missing_interpreters(
        f"tox run -e {','.join(names)} --notest"
    )

def tox_gate_precommit_warm_command(workspace: Path) -> str | None:
    if not (workspace / ".pre-commit-config.yaml").is_file():
        return None
    names = tox_gate_env_names(workspace)
    if not names:
        return None
    soft = ' || echo "malvin: tox-env pre-commit install-hooks failed (continuing)" >&2'
    parts: list[str] = []
    for name in names:
        
        py = f".tox/{name}/bin/python"
        parts.append(
            f'if [ -x {shlex.quote(py)} ]; then '
            f'{shlex.quote(py)} -m pre_commit install-hooks{soft}; '
            f'else echo "malvin: skip pre-commit warm missing {name}" >&2; fi'
        )
    return "; ".join(parts)

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

def _test_clamp_tox_version_and_image_build_pip() -> None:
    assert clamp_tox_version(None) == MIN_TOX_FOR_SKIP_ENV_INSTALL
    assert clamp_tox_version("4.23.2") == MIN_TOX_FOR_SKIP_ENV_INSTALL
    assert clamp_tox_version("4.42.0") == "4.42.0"
    assert clamp_tox_version("4.50.3") == "4.50.3"
    cmd = image_build_pip_install_command("tox==4.42.0")
    assert "/opt/venv/bin/python -m pip install --no-cache-dir tox==4.42.0" in cmd
    assert "--break-system-packages tox==4.42.0" in cmd

def _test_tox_gate_precommit_warm_command() -> None:
    import tempfile

    with tempfile.TemporaryDirectory() as tmp:
        root = Path(tmp)
        assert tox_gate_precommit_warm_command(root) is None
        (root / ".pre-commit-config.yaml").write_text("repos: []\n", encoding="utf-8")
        assert tox_gate_precommit_warm_command(root) is None
        (root / "tox.ini").write_text(
            "[tox]\nenvlist = lint\n[testenv:lint]\ncommands = pre-commit run --all-files\n",
            encoding="utf-8",
        )
        cmd = tox_gate_precommit_warm_command(root)
        assert cmd is not None
        assert ".tox/lint/bin/python" in cmd
        assert "pre_commit install-hooks" in cmd
        assert ".tox/*/" not in cmd

def _test_tox_gate_env_names_omit_mypy_typecheck() -> None:
    import tempfile

    with tempfile.TemporaryDirectory() as tmp:
        root = Path(tmp)
        (root / "tox.ini").write_text(
            "[tox]\nenvlist = mypy,typecheck,lint\n"
            "[testenv:mypy]\ncommands = mypy gql\n"
            "[testenv:typecheck]\ncommands = mypy src\n"
            "[testenv:lint]\ncommands = ruff check .\n",
            encoding="utf-8",
        )
        assert tox_gate_env_names(root) == ["lint"]
        assert tox_gate_check_commands(root) == [
            "tox run -e lint --skip-missing-interpreters true "
            "--skip-env-install --skip-pkg-install"
        ]
    with tempfile.TemporaryDirectory() as tmp:
        root = Path(tmp)
        (root / "tox.ini").write_text(
            "[tox]\nenvlist = mypy\n[testenv:mypy]\ncommands = mypy gql tests\n",
            encoding="utf-8",
        )
        assert tox_gate_env_names(root) == []
        assert tox_gate_check_commands(root) == []

if __name__ == "__main__":
    _test_tox_gate_check_commands_offline_flags()
    _test_clamp_tox_version_and_image_build_pip()
    _test_tox_gate_precommit_warm_command()
    _test_tox_gate_env_names_omit_mypy_typecheck()
    print("tox_gates self-test: ok")
