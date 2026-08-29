#!/usr/bin/env python3
"""Self-contained Harbor-style grader for FT-37. No malvin/repo imports."""
from __future__ import annotations

import argparse
import json
import os
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path


TASK_ID = "FT-37"
GOLD_FUNC = "apply_discount"
VALID_FUNCS = {"round_money", "apply_discount", "add_tax", "cart_total"}
MEMO_FUNC = "round_money"


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


def _run_hidden(workspace: Path) -> bool:
    hidden = Path(__file__).resolve().parent / "goldens" / "test_invoice_hidden.py"
    with tempfile.TemporaryDirectory() as td:
        td_path = Path(td)
        shutil.copytree(workspace / "src", td_path / "src")
        (td_path / "tests").mkdir()
        shutil.copy2(hidden, td_path / "tests" / "test_invoice_hidden.py")
        (td_path / "tests" / "__init__.py").write_text("", encoding="utf-8")
        (td_path / "src" / "__init__.py").write_text("", encoding="utf-8")
        proc = subprocess.run(
            [
                sys.executable,
                "-m",
                "pytest",
                "-q",
                "tests/test_invoice_hidden.py",
                "-p",
                "no:cacheprovider",
            ],
            cwd=td_path,
            capture_output=True,
            text=True,
            env={
                **os.environ,
                "PYTEST_DISABLE_PLUGIN_AUTOLOAD": "1",
                "PYTHONPATH": str(td_path),
            },
        )
        return proc.returncode == 0


def _answer_ok(workspace: Path) -> bool:
    path = workspace / "answer.json"
    if not path.is_file():
        return False
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
    except json.JSONDecodeError:
        return False
    if not isinstance(data, dict):
        return False
    if set(data.keys()) != {"faulty_function"}:
        return False
    fn = data.get("faulty_function")
    return fn in VALID_FUNCS and fn == GOLD_FUNC


def evaluate(workspace: Path) -> int:
    if not _answer_ok(workspace):
        return 0
    if not _run_hidden(workspace):
        return 0
    return 1


def _oracle_fix(workspace: Path) -> None:
    src = workspace / "src" / "invoice.py"
    text = src.read_text(encoding="utf-8")
    text = text.replace(
        "return int(x)\n",
        "return int(x + 0.5)\n",
    )
    text = text.replace(
        "    # BUG: treats percent as absolute cents, not a percentage\n"
        "    return subtotal_cents - percent\n",
        "    return subtotal_cents - (subtotal_cents * percent) // 100\n",
    )
    src.write_text(text, encoding="utf-8")
    (workspace / "answer.json").write_text(
        json.dumps({"faulty_function": GOLD_FUNC}, indent=2) + "\n",
        encoding="utf-8",
    )


def self_test() -> None:
    src = default_workspace()
    with tempfile.TemporaryDirectory() as td:
        fail_ws = Path(td) / "fail"
        shutil.copytree(src, fail_ws)
        assert evaluate(fail_ws) == 0, "starter must fail"

        pass_ws = Path(td) / "pass"
        shutil.copytree(src, pass_ws)
        _oracle_fix(pass_ws)
        assert evaluate(pass_ws) == 1, "oracle must pass"

        # Memo diagnosis + memo patch only
        memo_ws = Path(td) / "memo"
        shutil.copytree(src, memo_ws)
        inv = (memo_ws / "src" / "invoice.py").read_text(encoding="utf-8")
        inv = inv.replace("return int(x)\n", "return int(x + 0.5)\n")
        (memo_ws / "src" / "invoice.py").write_text(inv, encoding="utf-8")
        (memo_ws / "answer.json").write_text(
            json.dumps({"faulty_function": MEMO_FUNC}) + "\n", encoding="utf-8"
        )
        assert evaluate(memo_ws) == 0, "memo-only must fail"

        # Fixed code but memo answer
        wrong_ans = Path(td) / "wrong_ans"
        shutil.copytree(src, wrong_ans)
        _oracle_fix(wrong_ans)
        (wrong_ans / "answer.json").write_text(
            json.dumps({"faulty_function": MEMO_FUNC}) + "\n", encoding="utf-8"
        )
        assert evaluate(wrong_ans) == 0, "memo answer must fail even if code fixed"

        # Correct answer but code still broken
        broken = Path(td) / "broken"
        shutil.copytree(src, broken)
        (broken / "answer.json").write_text(
            json.dumps({"faulty_function": GOLD_FUNC}) + "\n", encoding="utf-8"
        )
        assert evaluate(broken) == 0, "answer alone must fail"

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
