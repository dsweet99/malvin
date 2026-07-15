#!/usr/bin/env python3
"""Self-contained Harbor-style grader for FT-03. No malvin/repo imports."""
from __future__ import annotations

import argparse
import os
import shutil
import subprocess
import tempfile
from pathlib import Path


TASK_ID = "FT-03"


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


def _run(bin_path: Path, args: list[str], cwd: Path):
    return subprocess.run(
        [str(bin_path), *args],
        cwd=cwd,
        capture_output=True,
        text=True,
    )


def evaluate(workspace: Path) -> int:
    bin_path = workspace / "bin" / "csvcut"
    if not bin_path.is_file():
        return 0
    gold = Path(__file__).resolve().parent / "goldens"
    cases = [
        (["-f", "b,a", "data/input.csv"], 0, (gold / "out_ba.csv").read_text(encoding="utf-8"), None),
        (["-f", "c", "data/input.csv"], 0, (gold / "out_c.csv").read_text(encoding="utf-8"), None),
        (["-f", "a,z", "data/input.csv"], 2, None, "missing: z\n"),
        (["-f", "nope", "data/input.csv"], 2, None, "missing: nope\n"),
        (["-f", "c,b,a", "data/input.csv"], 0, "c,b,a\n3,2,1\n6,5,4\n", None),
    ]
    for argv, exp_code, exp_out, exp_err in cases:
        proc = _run(bin_path, argv, workspace)
        if proc.returncode != exp_code:
            return 0
        if exp_out is not None and proc.stdout != exp_out:
            return 0
        if exp_err is not None and proc.stderr != exp_err:
            return 0
    return 1


ORACLE = """#!/usr/bin/env python3
import csv, sys
args = sys.argv[1:]
if len(args) < 3 or args[0] != "-f":
    print("usage: csvcut -f cols file", file=sys.stderr)
    sys.exit(1)
fields = args[1].split(",")
path = args[2]
with open(path, newline="") as f:
    reader = csv.DictReader(f)
    for name in fields:
        if name not in reader.fieldnames:
            sys.stderr.write("missing: " + name + chr(10))
            sys.exit(2)
    w = csv.DictWriter(sys.stdout, fieldnames=fields, lineterminator=chr(10))
    w.writeheader()
    for row in reader:
        w.writerow({k: row[k] for k in fields})
"""


def _oracle_fix(workspace: Path) -> None:
    p = workspace / "bin" / "csvcut"
    p.write_text(ORACLE, encoding="utf-8")
    p.chmod(0o755)


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
