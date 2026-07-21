#!/usr/bin/env python3
"""Self-contained Harbor-style grader for FT-35. No malvin/repo imports."""
from __future__ import annotations

import argparse
import importlib.util
import json
import os
import shutil
import sys
import tempfile
from pathlib import Path


TASK_ID = "FT-35"

GOLD = {
    "alpha_inv_times_1e12": 137035999177,
    "rydberg_frequency_hz": 3289841960250000,
}


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


def _load(workspace: Path):
    source = workspace / "meta" / "codata.py"
    with tempfile.TemporaryDirectory() as temp_dir:
        pkg = Path(temp_dir) / "meta"
        pkg.mkdir()
        (pkg / "__init__.py").write_text("", encoding="utf-8")
        dest = pkg / "codata.py"
        dest.write_bytes(source.read_bytes())
        sys.path.insert(0, temp_dir)
        try:
            for name in ("meta.codata", "meta"):
                if name in sys.modules:
                    del sys.modules[name]
            spec = importlib.util.spec_from_file_location("meta.codata", dest)
            assert spec is not None and spec.loader is not None
            module = importlib.util.module_from_spec(spec)
            sys.modules["meta.codata"] = module
            spec.loader.exec_module(module)
            return module.alpha_inv_times_1e12, module.rydberg_frequency_hz
        finally:
            sys.path.pop(0)


def _is_plain_int(value: object) -> bool:
    return isinstance(value, int) and not isinstance(value, bool)


def evaluate(workspace: Path) -> int:
    sys.dont_write_bytecode = True
    try:
        alpha_fn, rydberg_fn = _load(workspace)
    except Exception:
        return 0

    try:
        alpha = alpha_fn()
        rydberg = rydberg_fn()
    except Exception:
        return 0

    if not _is_plain_int(alpha) or not _is_plain_int(rydberg):
        return 0
    if alpha != GOLD["alpha_inv_times_1e12"]:
        return 0
    if rydberg != GOLD["rydberg_frequency_hz"]:
        return 0

    # Near-miss structural guards (edition / scale).
    if alpha == 137035999084:
        return 0
    if alpha == 137035999177000:
        return 0
    if rydberg == 3289841960355000:
        return 0
    return 1


ORACLE = '''\
"""CODATA 2022 accessors."""


def alpha_inv_times_1e12() -> int:
    return 137035999177


def rydberg_frequency_hz() -> int:
    return 3289841960250000
'''


def _oracle_fix(workspace: Path) -> None:
    (workspace / "meta" / "codata.py").write_text(ORACLE, encoding="utf-8")


def self_test() -> None:
    gold_path = Path(__file__).resolve().parent / "goldens" / "values.json"
    assert json.loads(gold_path.read_text(encoding="utf-8")) == GOLD

    source = default_workspace()
    with tempfile.TemporaryDirectory() as temp_dir:
        fail_workspace = Path(temp_dir) / "fail"
        shutil.copytree(source, fail_workspace)
        assert evaluate(fail_workspace) == 0, "starter must fail"

        pass_workspace = Path(temp_dir) / "pass"
        shutil.copytree(source, pass_workspace)
        _oracle_fix(pass_workspace)
        assert evaluate(pass_workspace) == 1, "oracle must pass"

        # Near-miss: CODATA 2018 alpha.
        wrong = ORACLE.replace("137035999177", "137035999084")
        (pass_workspace / "meta" / "codata.py").write_text(wrong, encoding="utf-8")
        assert evaluate(pass_workspace) == 0, "2018 alpha must fail"

        # Near-miss: ×1000 scale error (observed agent failure mode).
        wrong2 = ORACLE.replace(
            "return 137035999177",
            "return 137035999177000",
        )
        (pass_workspace / "meta" / "codata.py").write_text(wrong2, encoding="utf-8")
        assert evaluate(pass_workspace) == 0, "alpha scale ×1000 must fail"

        # Near-miss: 2018 Rydberg with correct alpha.
        wrong3 = ORACLE.replace(
            "return 3289841960250000",
            "return 3289841960355000",
        )
        (pass_workspace / "meta" / "codata.py").write_text(wrong3, encoding="utf-8")
        assert evaluate(pass_workspace) == 0, "2018 Rydberg must fail"

        # Type smuggling: float return.
        wrong4 = ORACLE.replace(
            "return 137035999177",
            "return 137035999177.0",
        )
        (pass_workspace / "meta" / "codata.py").write_text(wrong4, encoding="utf-8")
        assert evaluate(pass_workspace) == 0, "float alpha must fail"
    print(f"{TASK_ID} self-test OK")


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=f"Grade {TASK_ID}")
    parser.add_argument("--workspace", type=Path, default=None)
    parser.add_argument("--reward-out", type=Path, default=None)
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args(argv)
    if args.self_test:
        self_test()
        return 0
    workspace = args.workspace or default_workspace()
    reward_out = args.reward_out or default_reward_out()
    reward = evaluate(workspace)
    write_reward(reward_out, reward)
    print("PASS" if reward == 1 else "FAIL")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
