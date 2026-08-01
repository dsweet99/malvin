#!/usr/bin/env python3
"""Self-contained Harbor-style grader for FT-32. No malvin/repo imports."""
from __future__ import annotations

import argparse
import ast
import json
import os
import shutil
import subprocess
import sys
import tempfile
import textwrap
from pathlib import Path


TASK_ID = "FT-32"
FORBIDDEN_IMPORT_ROOTS = {"sqlite3", "dbm", "shelve", "malvin", "ops", "src"}
MEM_BUDGET = 8 * 1024 * 1024
# Absolute peak RSS in KiB for a clean workload subprocess. Sized so a Python
# dict holding the full working set (~200k × 512B values) exceeds the cap,
# while a memtable+SST design stays under it.
MAX_RSS_KB = 100 * 1024
WALL_TIME_S = 120.0
N_KEYS = 200_000
VALUE_LEN = 512


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


def _check_forbidden_imports(diskmap_py: Path) -> bool:
    tree = ast.parse(diskmap_py.read_text(encoding="utf-8"), filename=str(diskmap_py))
    for node in ast.walk(tree):
        if isinstance(node, ast.Import):
            for alias in node.names:
                if alias.name.split(".")[0] in FORBIDDEN_IMPORT_ROOTS:
                    return False
        elif isinstance(node, ast.ImportFrom) and node.module:
            if node.module.split(".")[0] in FORBIDDEN_IMPORT_ROOTS:
                return False
    return True


def _workload_script() -> str:
    # Embedded so grade.py stays single-file for Harbor; constants injected below.
    return textwrap.dedent(
        """
        from __future__ import annotations
        import json, resource, struct, sys, time, hashlib
        from pathlib import Path

        MEM_BUDGET = int(sys.argv[1])
        N_KEYS = int(sys.argv[2])
        VALUE_LEN = int(sys.argv[3])
        WS = Path(sys.argv[4])
        DB = Path(sys.argv[5])
        KEY_PREFIX = b"k"

        def key(i: int) -> bytes:
            return KEY_PREFIX + struct.pack(">I", i)

        def val(i: int, salt: int = 0) -> bytes:
            digest = hashlib.sha256(struct.pack(">II", i, salt)).digest()
            return (digest * ((VALUE_LEN // 32) + 1))[:VALUE_LEN]

        def expected_get(i: int):
            # Final logical value after put-all, update every 5th, delete every 10th.
            if i % 10 == 0:
                return None
            if i % 5 == 0:
                return val(i, 1)
            return val(i, 0)

        sys.path.insert(0, str(WS))
        from kvstore.diskmap import DiskMap

        t0 = time.perf_counter()
        store = DiskMap(DB, mem_budget_bytes=MEM_BUDGET)
        n = N_KEYS
        for i in range(n):
            store.put(key(i), val(i, 0))
        for i in range(0, n, 5):
            store.put(key(i), val(i, 1))
        for i in range(0, n, 10):
            store.delete(key(i))
        store.drop_cache()

        for i in range(0, n, max(1, n // 4000)):
            got = store.get(key(i))
            want = expected_get(i)
            if got != want:
                raise SystemExit(json.dumps({"ok": False, "err": f"get:{i}"}))

        for lo_i, hi_i in ((0, 250), (n // 3, n // 3 + 180), (n - 400, n)):
            lo, hi = key(lo_i), key(hi_i)
            got = store.range(lo, hi)
            want = [(key(i), expected_get(i)) for i in range(lo_i, hi_i) if expected_get(i) is not None]
            if got != want:
                raise SystemExit(json.dumps({"ok": False, "err": f"range:{lo_i}"}))

        store.close()
        store2 = DiskMap(DB, mem_budget_bytes=MEM_BUDGET)
        store2.drop_cache()
        for i in (1, 11, 21, n // 4, n - 3):
            if store2.get(key(i)) != expected_get(i):
                raise SystemExit(json.dumps({"ok": False, "err": f"reopen:{i}"}))
        store2.close()

        wall = time.perf_counter() - t0
        rss = resource.getrusage(resource.RUSAGE_SELF).ru_maxrss
        # Linux reports KiB; macOS/BSD report bytes.
        if sys.platform != "linux":
            rss //= 1024
        elif rss > 512 * 1024 * 1024:
            rss //= 1024
        print(json.dumps({"ok": True, "wall": wall, "rss_kb": int(rss)}))
        """
    )


def _run_workload(workspace: Path) -> tuple[bool, float, int, str]:
    with tempfile.TemporaryDirectory() as td:
        db = Path(td) / "db"
        script = Path(td) / "workload.py"
        script.write_text(_workload_script(), encoding="utf-8")
        proc = subprocess.run(
            [
                sys.executable,
                str(script),
                str(MEM_BUDGET),
                str(N_KEYS),
                str(VALUE_LEN),
                str(workspace.resolve()),
                str(db),
            ],
            cwd=td,
            capture_output=True,
            text=True,
            check=False,
            env={**os.environ, "PYTHONDONTWRITEBYTECODE": "1"},
        )
        if proc.returncode != 0:
            err = (proc.stderr or proc.stdout or "workload failed").strip()
            # Try parse JSON error
            for line in (proc.stdout or "").splitlines()[::-1]:
                line = line.strip()
                if line.startswith("{"):
                    try:
                        data = json.loads(line)
                        return False, 0.0, 0, str(data.get("err", err))
                    except json.JSONDecodeError:
                        pass
            return False, 0.0, 0, err[:500]
        line = (proc.stdout or "").strip().splitlines()[-1]
        data = json.loads(line)
        if not data.get("ok"):
            return False, 0.0, 0, str(data.get("err", "not ok"))
        return True, float(data["wall"]), int(data["rss_kb"]), ""


def _public_tests_ok(workspace: Path) -> bool:
    proc = subprocess.run(
        [sys.executable, "-m", "pytest", "-q", "tests", "-p", "no:cacheprovider"],
        cwd=workspace,
        capture_output=True,
        text=True,
        check=False,
        env={**os.environ, "PYTEST_DISABLE_PLUGIN_AUTOLOAD": "1"},
    )
    return proc.returncode == 0


def evaluate(workspace: Path) -> int:
    workspace = workspace.resolve()
    diskmap = workspace / "kvstore" / "diskmap.py"
    if not diskmap.is_file():
        return 0
    if not _check_forbidden_imports(diskmap):
        return 0
    if not _public_tests_ok(workspace):
        return 0
    ok, wall, rss_kb, _err = _run_workload(workspace)
    if not ok:
        return 0
    if wall > WALL_TIME_S:
        return 0
    if rss_kb > MAX_RSS_KB:
        return 0
    return 1


def _oracle_fix(workspace: Path) -> None:
    src = Path(__file__).resolve().parent / "goldens" / "oracle_diskmap.py"
    dst = workspace / "kvstore" / "diskmap.py"
    dst.write_text(src.read_text(encoding="utf-8"), encoding="utf-8")


def self_test() -> None:
    """Contract checks; shrink N_KEYS so each unit-test invocation stays under 1.5s."""
    global N_KEYS
    src = default_workspace()
    old_keys = N_KEYS
    # Keep oracle workload tiny in unit tests; Harbor evaluate still uses N_KEYS.
    N_KEYS = 2_000
    try:
        with tempfile.TemporaryDirectory() as td:
            fail_ws = Path(td) / "fail"
            shutil.copytree(src, fail_ws)
            assert evaluate(fail_ws) == 0, "starter must fail"

            pass_ws = Path(td) / "pass"
            shutil.copytree(src, pass_ws)
            _oracle_fix(pass_ws)
            assert evaluate(pass_ws) == 1, "oracle must pass"

            bad = Path(td) / "bad"
            shutil.copytree(pass_ws, bad)
            text = (bad / "kvstore" / "diskmap.py").read_text(encoding="utf-8")
            (bad / "kvstore" / "diskmap.py").write_text(
                "import sqlite3\n" + text, encoding="utf-8"
            )
            assert evaluate(bad) == 0, "sqlite3 import must fail"
    finally:
        N_KEYS = old_keys
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
