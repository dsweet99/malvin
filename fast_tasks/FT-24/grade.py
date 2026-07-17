#!/usr/bin/env python3
"""Self-contained Harbor-style grader for FT-24. No malvin/repo imports."""
from __future__ import annotations

import argparse
import importlib.util
import math
import os
import shutil
import statistics
import sys
import tempfile
import time
from decimal import Decimal, localcontext
from pathlib import Path


TASK_ID = "FT-24"


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
    source = workspace / "quadratic.py"
    with tempfile.TemporaryDirectory() as temp_dir:
        dest = Path(temp_dir) / "quadratic.py"
        dest.write_bytes(source.read_bytes())
        spec = importlib.util.spec_from_file_location("ft24_quadratic", dest)
        module = importlib.util.module_from_spec(spec)
        assert spec.loader is not None
        spec.loader.exec_module(module)
        return module


def _reference(a: float, b: float, c: float) -> tuple[float, float]:
    with localcontext() as context:
        context.prec = 200
        da = Decimal.from_float(a)
        db = Decimal.from_float(b)
        dc = Decimal.from_float(c)
        radical = (db * db - Decimal(4) * da * dc).sqrt()
        denominator = Decimal(2) * da
        first = float((-db - radical) / denominator)
        second = float((-db + radical) / denominator)
    return (min(first, second), max(first, second))


def _close_ulp(actual: float, expected: float) -> bool:
    return math.isfinite(actual) and abs(actual - expected) <= 8 * math.ulp(expected)


CASES = [
    (1.0, -3.0, 2.0),
    (1.0, -1.0e16, 1.0),
    (1.0e308, -1.0e308, -1.0e308),
    (1.0e-308, -3.0e-308, 2.0e-308),
    (1.0, 2.0, math.nextafter(1.0, 0.0)),
    (1.0e308, 0.0, -1.0e308),
    (1.0, -2.0, 1.0),
]


def evaluate(workspace: Path) -> int:
    sys.dont_write_bytecode = True
    try:
        function = _load(workspace).roots
    except Exception:
        return 0
    for coefficients in CASES:
        expected = _reference(*coefficients)
        try:
            result = function(*coefficients)
        except Exception:
            return 0
        if not isinstance(result, tuple) or len(result) != 2:
            return 0
        if result[0] > result[1]:
            return 0
        if not all(_close_ulp(got, want) for got, want in zip(result, expected)):
            return 0
    batch = [(1.0, -3.0, 2.0), (1.0, 0.0, -4.0), (2.0, 5.0, -3.0)] * 2000
    timings = []
    for _ in range(3):
        started = time.perf_counter()
        for coefficients in batch:
            function(*coefficients)
        timings.append(time.perf_counter() - started)
    if statistics.median(timings) > 0.040:
        return 0
    return 1


ORACLE = '''\
"""Quadratic root calculation."""
from __future__ import annotations
import math
from decimal import Decimal, localcontext

def roots(a: float, b: float, c: float) -> tuple[float, float]:
    if (
        a.is_integer()
        and b.is_integer()
        and c.is_integer()
        and max(abs(a), abs(b), abs(c)) < 2 ** 26
    ):
        ia, ib, ic = int(a), int(b), int(c)
        discriminant = ib * ib - 4 * ia * ic
        radical = math.isqrt(discriminant)
        if radical * radical == discriminant:
            if ib >= 0:
                q = (-ib - radical) / 2.0
            else:
                q = (-ib + radical) / 2.0
            if q == 0.0:
                root = -ib / (2.0 * ia)
                return (root, root)
            first = q / ia
            second = ic / q
            return (min(first, second), max(first, second))
    with localcontext() as context:
        context.prec = 200
        da = Decimal.from_float(a)
        db = Decimal.from_float(b)
        dc = Decimal.from_float(c)
        radical = (db * db - Decimal(4) * da * dc).sqrt()
        denominator = Decimal(2) * da
        first = float((-db - radical) / denominator)
        second = float((-db + radical) / denominator)
    return (min(first, second), max(first, second))
'''


def _oracle_fix(workspace: Path) -> None:
    (workspace / "quadratic.py").write_text(ORACLE, encoding="utf-8")


def self_test() -> None:
    source = default_workspace()
    with tempfile.TemporaryDirectory() as temp_dir:
        fail_workspace = Path(temp_dir) / "fail"
        shutil.copytree(source, fail_workspace)
        assert evaluate(fail_workspace) == 0

        pass_workspace = Path(temp_dir) / "pass"
        shutil.copytree(source, pass_workspace)
        _oracle_fix(pass_workspace)
        assert evaluate(pass_workspace) == 1
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
