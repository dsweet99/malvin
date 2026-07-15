#!/usr/bin/env python3
"""Self-contained Harbor-style grader for FT-13. No malvin/repo imports."""
from __future__ import annotations

import argparse
import os
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path


TASK_ID = "FT-13"


def write_reward(path: Path, value: int) -> None:
    assert value in (0, 1)
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(f"{value}\n", encoding="utf-8")


def default_workspace() -> Path:
    return Path(__file__).resolve().parent / "workspace"


def default_reward_out() -> Path:
    env = os.environ.get("MALVIN_REWARD_PATH") or os.environ.get("HARBOR_REWARD_PATH")
    if env:
        return Path(env)
    return Path(__file__).resolve().parent / "reward.txt"


def _pytest(ws: Path, impl_file: Path) -> int:
    with tempfile.TemporaryDirectory() as td:
        td_path = Path(td)
        (td_path / "impl").mkdir()
        (td_path / "tests").mkdir()
        shutil.copy2(impl_file, td_path / "impl" / "normalize.py")
        (td_path / "impl" / "__init__.py").write_text("", encoding="utf-8")
        shutil.copy2(ws / "tests" / "test_normalize.py", td_path / "tests" / "test_normalize.py")
        (td_path / "tests" / "__init__.py").write_text("", encoding="utf-8")
        # Make imports work: tests should import from impl.normalize
        proc = subprocess.run(
            [sys.executable, "-m", "pytest", "-q", "tests/test_normalize.py"],
            cwd=td_path,
            capture_output=True,
            text=True,
            env={**dict(**{k: v for k, v in __import__("os").environ.items()}), "PYTHONPATH": str(td_path)},
        )
        return proc.returncode


def evaluate(workspace: Path) -> int:
    test_path = workspace / "tests" / "test_normalize.py"
    if not test_path.is_file():
        return 0
    buggy = workspace / "impl" / "normalize.py"
    correct = Path(__file__).resolve().parent / "goldens" / "normalize_correct.py"
    buggy_code = _pytest(workspace, buggy)
    correct_code = _pytest(workspace, correct)
    # Need: correct PASS (0) and buggy FAIL (nonzero)
    if correct_code == 0 and buggy_code != 0:
        return 1
    return 0


ORACLE_TESTS = '''
from impl.normalize import normalize_ws

def test_spaces():
    assert normalize_ws("  a   b  ") == "a b"

def test_tabs_are_whitespace():
    assert normalize_ws("a\tb") == "a b"
'''


def _oracle_fix(workspace: Path) -> None:
    (workspace / "tests" / "test_normalize.py").write_text(ORACLE_TESTS, encoding="utf-8")


def self_test() -> None:
    src = default_workspace()
    with tempfile.TemporaryDirectory() as td:
        fail_ws = Path(td) / "fail"
        shutil.copytree(src, fail_ws)
        assert evaluate(fail_ws) == 0
        pass_ws = Path(td) / "pass"
        shutil.copytree(src, pass_ws)
        _oracle_fix(pass_ws)
        assert evaluate(pass_ws) == 1
    print(f"{TASK_ID} self-test OK")


def main(argv: list[str] | None = None) -> int:
    p = argparse.ArgumentParser(description=f"Grade {TASK_ID}")
    p.add_argument("--workspace", type=Path, default=None)
    p.add_argument("--reward-out", type=Path, default=None)
    p.add_argument("--self-test", action="store_true")
    args = p.parse_args(argv)
    if args.self_test:
        self_test()
        return 0
    ws = args.workspace or default_workspace()
    out = args.reward_out or default_reward_out()
    reward = evaluate(ws)
    write_reward(out, reward)
    print("PASS" if reward == 1 else "FAIL")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
