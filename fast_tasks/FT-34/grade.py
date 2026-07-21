#!/usr/bin/env python3
"""Self-contained Harbor-style grader for FT-34. No malvin/repo imports."""
from __future__ import annotations

import argparse
import importlib.util
import math
import os
import shutil
import sys
import tempfile
from pathlib import Path


TASK_ID = "FT-34"


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


def _q_cont(r: float, length: float, dp: float, mu: float) -> float:
    return math.pi * r**4 * dp / (8.0 * mu * length)


def _q_slip(r: float, length: float, dp: float, mu: float, kn: float) -> float:
    # First-order Maxwell slip, σ=0.9, Kn=λ/R:
    # Q = Q_cont * (1 + 4 * ((2-σ)/σ) * Kn)
    sigma = 0.9
    return _q_cont(r, length, dp, mu) * (1.0 + 4.0 * ((2.0 - sigma) / sigma) * kn)


def _load(workspace: Path):
    source = workspace / "flow" / "poiseuille.py"
    with tempfile.TemporaryDirectory() as temp_dir:
        pkg = Path(temp_dir) / "flow"
        pkg.mkdir()
        (pkg / "__init__.py").write_text("", encoding="utf-8")
        dest = pkg / "poiseuille.py"
        dest.write_bytes(source.read_bytes())
        sys.path.insert(0, temp_dir)
        try:
            for name in ("flow.poiseuille", "flow"):
                if name in sys.modules:
                    del sys.modules[name]
            spec = importlib.util.spec_from_file_location("flow.poiseuille", dest)
            assert spec is not None and spec.loader is not None
            module = importlib.util.module_from_spec(spec)
            sys.modules["flow.poiseuille"] = module
            spec.loader.exec_module(module)
            return module.mass_flow_rate
        finally:
            sys.path.pop(0)


def evaluate(workspace: Path) -> int:
    sys.dont_write_bytecode = True
    try:
        fn = _load(workspace)
    except Exception:
        return 0

    # Continuum cases
    continuum_cases = [
        (1e-5, 1e-2, 1000.0, 1.8e-5, 0.0),
        (2e-6, 5e-3, 250.0, 2.2e-5, 0.0),
    ]
    for r, length, dp, mu, kn in continuum_cases:
        want = _q_cont(r, length, dp, mu)
        try:
            got = fn(r, length, dp, mu, kn)
        except Exception:
            return 0
        if not math.isfinite(got) or abs(got - want) > 1e-12 * max(1.0, abs(want)):
            return 0

    # Slip-regime cases (continuum-only implementations fail these).
    slip_cases = [
        (1e-5, 1e-2, 1000.0, 1.8e-5, 0.05),
        (5e-6, 2e-2, 500.0, 1.5e-5, 0.12),
        (1e-6, 1e-3, 80.0, 1.8e-5, 0.02),
    ]
    for r, length, dp, mu, kn in slip_cases:
        want = _q_slip(r, length, dp, mu, kn)
        try:
            got = fn(r, length, dp, mu, kn)
        except Exception:
            return 0
        if not math.isfinite(got) or abs(got - want) > 1e-9 * max(1.0, abs(want)):
            return 0
        # Mid-process / structural check: multiplier must be exactly (1+4Kn)
        # relative to the continuum branch on the same geometry.
        try:
            base = fn(r, length, dp, mu, 0.0)
        except Exception:
            return 0
        if base == 0.0:
            return 0
        multiplier = got / base
        sigma = 0.9
        want_mult = 1.0 + 4.0 * ((2.0 - sigma) / sigma) * kn
        if abs(multiplier - want_mult) > 1e-9:
            return 0

    # Near-miss: σ=1 shortcut (1+4Kn) and 1+6Kn must not match (relative).
    r, length, dp, mu, kn = slip_cases[0]
    got = fn(r, length, dp, mu, kn)
    base = _q_cont(r, length, dp, mu)
    correct = base * (1.0 + 4.0 * ((2.0 - 0.9) / 0.9) * kn)

    def rel_close(a: float, b: float) -> bool:
        return abs(a - b) <= 1e-9 * max(abs(a), abs(b), 1e-30)

    if not rel_close(got, correct):
        return 0
    if rel_close(got, base * (1.0 + 4.0 * kn)):
        return 0
    if rel_close(got, base * (1.0 + 6.0 * kn)):
        return 0

    # Validation
    for kwargs in (
        dict(radius_m=0.0, length_m=1.0, delta_p_pa=1.0, viscosity_pa_s=1.0, knudsen=0.0),
        dict(radius_m=1.0, length_m=1.0, delta_p_pa=1.0, viscosity_pa_s=1.0, knudsen=-0.1),
        dict(radius_m=1.0, length_m=1.0, delta_p_pa=1.0, viscosity_pa_s=float("nan"), knudsen=0.0),
    ):
        try:
            fn(**kwargs)
        except ValueError:
            pass
        except Exception:
            return 0
        else:
            return 0
    return 1


ORACLE = '''\
"""Hagen–Poiseuille microchannel flow."""
from __future__ import annotations

import math


def mass_flow_rate(
    radius_m: float,
    length_m: float,
    delta_p_pa: float,
    viscosity_pa_s: float,
    knudsen: float = 0.0,
) -> float:
    for name, value in (
        ("radius_m", radius_m),
        ("length_m", length_m),
        ("delta_p_pa", delta_p_pa),
        ("viscosity_pa_s", viscosity_pa_s),
    ):
        if not isinstance(value, (int, float)) or isinstance(value, bool):
            raise ValueError(f"{name} must be a real number")
        if not math.isfinite(float(value)) or float(value) <= 0.0:
            raise ValueError(f"{name} must be finite and positive")
    if not isinstance(knudsen, (int, float)) or isinstance(knudsen, bool):
        raise ValueError("knudsen must be a real number")
    if not math.isfinite(float(knudsen)) or float(knudsen) < 0.0:
        raise ValueError("knudsen must be finite and >= 0")

    q_cont = math.pi * float(radius_m) ** 4 * float(delta_p_pa) / (
        8.0 * float(viscosity_pa_s) * float(length_m)
    )
    kn = float(knudsen)
    if kn == 0.0:
        return q_cont
    sigma = 0.9
    return q_cont * (1.0 + 4.0 * ((2.0 - sigma) / sigma) * kn)
'''


def _oracle_fix(workspace: Path) -> None:
    (workspace / "flow" / "poiseuille.py").write_text(ORACLE, encoding="utf-8")


def self_test() -> None:
    source = default_workspace()
    with tempfile.TemporaryDirectory() as temp_dir:
        fail_workspace = Path(temp_dir) / "fail"
        shutil.copytree(source, fail_workspace)
        assert evaluate(fail_workspace) == 0, "starter must fail"

        pass_workspace = Path(temp_dir) / "pass"
        shutil.copytree(source, pass_workspace)
        _oracle_fix(pass_workspace)
        assert evaluate(pass_workspace) == 1, "oracle must pass"

        # Near-miss: σ=1 shortcut.
        wrong = ORACLE.replace(
            "sigma = 0.9\n    return q_cont * (1.0 + 4.0 * ((2.0 - sigma) / sigma) * kn)",
            "return q_cont * (1.0 + 4.0 * kn)",
        )
        (pass_workspace / "flow" / "poiseuille.py").write_text(wrong, encoding="utf-8")
        assert evaluate(pass_workspace) == 0, "sigma=1 shortcut must fail"
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
