#!/usr/bin/env python3
"""Shared Harbor ``tests/test.patch`` / ``test.sh`` parsers for DeepSWE ops.

Contract (verifier dependency discovery):
- **Inputs:** task workspace, optional ``tests/`` dir, Dockerfile path.
- **Public layer:** ``DeclaredDeps`` from workspace manifests (agent-readable).
- **Grade-only layer:** imports / closure / plugin policy derived from ``test.patch``
  or Harbor ``test.sh`` — never bake these into agent-phase metadata or agent image
  materialize commands.
- **Verifier venv:** ``/opt/malvin-verifier`` (outside ``/app``; remounts cannot wipe it).
- **Non-leakage:** agent must not observe ``tests/``, ``test.patch``, solution patches,
  ``test.patch``-derived install deltas, or rich grade-only ``VerifierSpec`` fields.
- **Out of scope:** Docker host ``grade_workspace`` (fresh Harbor image); non-Python tasks.

Parse-added-hunks is preferred over apply-to-tree. When apply is required, use a temp
directory outside ``/app`` and never leave hidden tests on disk for the agent phase.
"""

from __future__ import annotations

import ast
import re
import sys
from pathlib import Path

_DIFF_GIT_RE = re.compile(r"^diff --git a/(.+?) b/(.+?)\s*$")
_PYTEST_INVOCATION_RE = re.compile(
    r"(?:python3?|\$PYTHON|\$\{PYTHON\}|\"\$PYTHON_BIN\"|\$PYTHON_BIN)\s+-m\s+pytest\b(.*)$"
    r"|^\s*pytest\b(.*)$",
    re.I,
)


def embedded_file_body_from_patch(patch_path: Path, relative_path: str) -> str | None:
    """Return added body for ``relative_path`` from a unified diff patch, if present."""
    if not patch_path.is_file():
        return None
    target_suffixes = {
        f"+++ b/{relative_path}",
        f"+++ b/{relative_path.lstrip('./')}",
    }
    text = patch_path.read_text(encoding="utf-8")
    added: list[str] = []
    in_target = False
    for line in text.splitlines():
        if any(line.startswith(suffix) or line == suffix for suffix in target_suffixes):
            in_target = True
            added = []
            continue
        if not in_target:
            continue
        if line.startswith("diff --git"):
            break
        if line.startswith("+++ b/") and not any(
            line.startswith(suffix) or line == suffix for suffix in target_suffixes
        ):
            break
        if line.startswith("+") and not line.startswith("+++"):
            added.append(line[1:])
    if not added:
        return None
    return "\n".join(added)


def embedded_test_sh_from_patch(patch_path: Path) -> str | None:
    """Return added ``test.sh`` body from a Harbor ``test.patch``, if present."""
    return embedded_file_body_from_patch(patch_path, "test.sh")


def embedded_test_py_from_patch(patch_path: Path) -> str | None:
    """Return added ``test.py`` body from a Harbor ``test.patch``, if present."""
    return embedded_file_body_from_patch(patch_path, "test.py")


def added_python_sources_from_patch(patch_path: Path) -> dict[str, str]:
    """Map relative paths → reconstructed NEW-side bodies for ``.py`` hunks.

    Prefer this over applying ``test.patch`` into a workspace tree. Paths are as
    written in ``+++ b/...`` headers (no leading ``b/``).

    For modified files, include unified-diff context lines (leading space) as well
    as ``+`` lines so third-party imports that appear only as unchanged context are
    not dropped from discovery / probe materialize. Deleted (``-``) lines are omitted.
    """
    if not patch_path.is_file():
        return {}
    text = patch_path.read_text(encoding="utf-8")
    sources: dict[str, str] = {}
    current_path: str | None = None
    new_lines: list[str] = []

    def _flush() -> None:
        nonlocal current_path, new_lines
        if current_path and current_path.endswith(".py") and new_lines:
            sources[current_path] = "\n".join(new_lines)
        current_path = None
        new_lines = []

    for line in text.splitlines():
        if _DIFF_GIT_RE.match(line):
            _flush()
            continue
        if line.startswith("+++ b/"):
            _flush()
            path = line[len("+++ b/") :].strip()
            current_path = path if path.endswith(".py") else None
            new_lines = []
            continue
        if current_path is None:
            continue
        if line.startswith("@@"):
            continue
        if line.startswith("+") and not line.startswith("+++"):
            new_lines.append(line[1:])
        elif line.startswith("-") and not line.startswith("---"):
            continue
        elif line.startswith("\\"):  # "\ No newline at end of file"
            continue
        elif line.startswith(" "):
            # Context line: present on the NEW side of a modified hunk.
            new_lines.append(line[1:])
    _flush()
    return sources


def resolve_harbor_test_sh_body(tests_dir: Path | None) -> str | None:
    """Return Harbor ``test.sh`` body from ``tests/test.sh`` or embedded patch content."""
    if tests_dir is None:
        return None
    direct = tests_dir / "test.sh"
    if direct.is_file():
        return direct.read_text(encoding="utf-8")
    return embedded_test_sh_from_patch(tests_dir / "test.patch")


def is_stdlib_module(name: str) -> bool:
    """Return True when *name* is a top-level stdlib (or built-in) module."""
    root = name.split(".", 1)[0]
    stdlib = getattr(sys, "stdlib_module_names", None)
    if stdlib is not None:
        return root in stdlib
    # Fallback for older interpreters: treat common stdlib roots as excluded.
    return root in {
        "abc",
        "ast",
        "asyncio",
        "collections",
        "contextlib",
        "copy",
        "dataclasses",
        "datetime",
        "enum",
        "functools",
        "importlib",
        "io",
        "json",
        "logging",
        "math",
        "os",
        "pathlib",
        "pickle",
        "re",
        "sys",
        "tempfile",
        "threading",
        "time",
        "typing",
        "unittest",
        "uuid",
        "warnings",
    }


# Import-root → distribution name when they differ (extend as tasks require).
_IMPORT_TO_DISTRIBUTION: dict[str, str] = {
    "attr": "attrs",
    "bs4": "beautifulsoup4",
    "cv2": "opencv-python",
    "dateutil": "python-dateutil",
    "graphql": "graphql-core",
    "PIL": "pillow",
    "yaml": "pyyaml",
    "skimage": "scikit-image",
    "sklearn": "scikit-learn",
}


def distribution_name_for_import(import_name: str) -> str:
    """Map a top-level import name to a likely PyPI / DeclaredDeps key."""
    root = import_name.split(".", 1)[0]
    if root in _IMPORT_TO_DISTRIBUTION:
        return _IMPORT_TO_DISTRIBUTION[root]
    return root.replace("_", "-").lower()


def top_level_imports_from_source(source: str) -> set[str]:
    """Return top-level imported module roots from Python *source* via AST."""
    try:
        tree = ast.parse(source)
    except SyntaxError:
        return set()
    names: set[str] = set()
    for node in ast.walk(tree):
        if isinstance(node, ast.Import):
            for alias in node.names:
                root = alias.name.split(".", 1)[0]
                if root:
                    names.add(root)
        elif isinstance(node, ast.ImportFrom):
            if node.level and node.level > 0:
                continue
            if not node.module:
                continue
            root = node.module.split(".", 1)[0]
            if root:
                names.add(root)
    return names


# Path segments whose Python sources are analysis samples / fixtures, not test code
# that the verifier imports at collection time (e.g. bandit challenge fixtures).
_ANALYSIS_SAMPLE_SEGMENTS = frozenset(
    {
        "fixtures",
        "fixture",
        "examples",
        "example",
        "samples",
        "sample",
        "testdata",
        "test_data",
        "data_files",
    }
)

# Import roots that are almost always local packages, never Harbor third-party deps.
_LOCAL_IMPORT_ROOTS = frozenset({"tests", "test", "conftest", "challenge"})


def is_analysis_sample_path(path: str | Path) -> bool:
    """True when *path* looks like fixture/sample code rather than a test module."""
    parts = Path(str(path)).parts
    return any(part.lower() in _ANALYSIS_SAMPLE_SEGMENTS for part in parts)


def harbor_imports_from_tests_dir(tests_dir: Path | None) -> tuple[str, ...]:
    """Third-party top-level imports discovered from Harbor patch / test sources.

    Skips analysis-sample paths (``fixtures/``, ``examples/``, …) whose imports are
    code under test, not verifier runtime dependencies.
    """
    if tests_dir is None:
        return ()
    found: set[str] = set()
    patch_path = tests_dir / "test.patch"
    for rel_path, body in added_python_sources_from_patch(patch_path).items():
        if is_analysis_sample_path(rel_path):
            continue
        found |= top_level_imports_from_source(body)
    for py_path in sorted(tests_dir.rglob("*.py")):
        try:
            rel = py_path.relative_to(tests_dir)
        except ValueError:
            rel = py_path
        if is_analysis_sample_path(rel):
            continue
        try:
            body = py_path.read_text(encoding="utf-8")
        except OSError:
            continue
        found |= top_level_imports_from_source(body)
    third_party = sorted(
        name
        for name in found
        if name
        and name not in _LOCAL_IMPORT_ROOTS
        and not is_stdlib_module(name)
    )
    return tuple(third_party)


def pytest_args_from_test_sh(script: str | None) -> tuple[str, ...]:
    """Extract pytest argument tokens from the first pytest invocation in *script*."""
    if not script:
        return ()
    for raw in script.splitlines():
        line = raw.strip()
        if not line or line.startswith("#"):
            continue
        match = _PYTEST_INVOCATION_RE.search(line)
        if not match:
            continue
        rest = (match.group(1) if match.lastindex and match.group(1) is not None else None) or (
            match.group(2) if match.lastindex and match.lastindex >= 2 else ""
        )
        if rest is None:
            rest = ""
        tokens = [tok for tok in rest.split() if tok and not tok.startswith("$")]
        return tuple(tokens)
    return ()


def collect_only_pytest_command(
    python_bin: str,
    script: str | None,
    *,
    extra_args: tuple[str, ...] = (),
) -> str:
    """Build a ``pytest --collect-only`` command mirroring Harbor ``test.sh`` args."""
    args = list(pytest_args_from_test_sh(script))
    if "--collect-only" not in args and "--co" not in args:
        args.insert(0, "--collect-only")
    args.extend(extra_args)
    quoted = " ".join(_shell_quote(tok) for tok in args)
    return f"{python_bin} -m pytest {quoted}".rstrip()


def _shell_quote(token: str) -> str:
    if re.fullmatch(r"[-A-Za-z0-9_./=,:]+", token):
        return token
    return "'" + token.replace("'", "'\"'\"'") + "'"


def run_self_tests() -> None:
    import tempfile

    with tempfile.TemporaryDirectory() as tmp:
        root = Path(tmp)
        patch = root / "test.patch"
        patch.write_text(
            "diff --git a/test.sh b/test.sh\n"
            "--- /dev/null\n"
            "+++ b/test.sh\n"
            "@@ -0,0 +1,3 @@\n"
            "+#!/bin/bash\n"
            "+python -m pytest tests/test_foo.py -q\n"
            "+\n"
            "diff --git a/tests/test_foo.py b/tests/test_foo.py\n"
            "--- /dev/null\n"
            "+++ b/tests/test_foo.py\n"
            "@@ -0,0 +1,5 @@\n"
            "+import os\n"
            "+import pytest\n"
            "+from typing_extensions import NoExtraItems\n"
            "+import typeguard\n"
            "+from adaptix import Retort\n",
            encoding="utf-8",
        )
        body = embedded_test_sh_from_patch(patch)
        assert body is not None
        assert "pytest" in body
        sources = added_python_sources_from_patch(patch)
        assert "tests/test_foo.py" in sources
        imports = top_level_imports_from_source(sources["tests/test_foo.py"])
        assert "pytest" in imports
        assert "typeguard" in imports
        assert "adaptix" in imports
        assert "typing_extensions" in imports
        assert "os" in imports
        harbor = harbor_imports_from_tests_dir(root)
        assert "os" not in harbor
        assert "pytest" in harbor
        assert "typeguard" in harbor
        assert distribution_name_for_import("typing_extensions") == "typing-extensions"
        args = pytest_args_from_test_sh(body)
        assert "tests/test_foo.py" in args
        cmd = collect_only_pytest_command("/opt/malvin-verifier/bin/python", body)
        assert "--collect-only" in cmd
        assert cmd.startswith("/opt/malvin-verifier/bin/python -m pytest")

        # Modified hunk: context-line imports must appear in reconstructed NEW body.
        mod_patch = root / "modified.patch"
        mod_patch.write_text(
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
        mod_sources = added_python_sources_from_patch(mod_patch)
        assert "tests/test_mod.py" in mod_sources
        mod_body = mod_sources["tests/test_mod.py"]
        assert "import only_in_context_pkg" in mod_body
        assert "def test_b():" in mod_body
        mod_imports = top_level_imports_from_source(mod_body)
        assert "only_in_context_pkg" in mod_imports
        (root / "test.patch").write_text(mod_patch.read_text(encoding="utf-8"), encoding="utf-8")
        mod_harbor = harbor_imports_from_tests_dir(root)
        assert "only_in_context_pkg" in mod_harbor

        # Analysis fixture paths must not contribute third-party Harbor imports.
        fixture_patch = root / "fixture.patch"
        fixture_patch.write_text(
            "diff --git a/challenge/fixtures/sample.py b/challenge/fixtures/sample.py\n"
            "--- /dev/null\n"
            "+++ b/challenge/fixtures/sample.py\n"
            "@@ -0,0 +1,2 @@\n"
            "+import flask\n"
            "+import django\n"
            "diff --git a/tests/test_real.py b/tests/test_real.py\n"
            "--- /dev/null\n"
            "+++ b/tests/test_real.py\n"
            "@@ -0,0 +1,2 @@\n"
            "+import pytest\n"
            "+import bandit\n",
            encoding="utf-8",
        )
        (root / "test.patch").write_text(
            fixture_patch.read_text(encoding="utf-8"), encoding="utf-8"
        )
        fixture_harbor = harbor_imports_from_tests_dir(root)
        assert "flask" not in fixture_harbor
        assert "django" not in fixture_harbor
        assert "pytest" in fixture_harbor
        assert "bandit" in fixture_harbor
        assert "tests" not in fixture_harbor
        assert is_analysis_sample_path("challenge/fixtures/x.py")
        assert distribution_name_for_import("graphql") == "graphql-core"
    print("harbor_tests self-tests passed")


if __name__ == "__main__":
    run_self_tests()
