#!/usr/bin/env python3
"""Self-contained Harbor-style grader for FT-23. No malvin/repo imports."""
from __future__ import annotations

import argparse
import hashlib
import json
import os
import shutil
import tempfile
from pathlib import Path


TASK_ID = "FT-23"
FORBIDDEN_KEYS = {"download_url", "api_key", "endpoint"}
REQUIRED_STEP_IDS = ["load", "mask", "fit", "report"]


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


def golden_path() -> Path:
    return Path(__file__).resolve().parent / "goldens" / "protocol_ok.json"


def _contains_key(obj, key: str) -> bool:
    if isinstance(obj, dict):
        if key in obj:
            return True
        return any(_contains_key(v, key) for v in obj.values())
    if isinstance(obj, list):
        return any(_contains_key(v, key) for v in obj)
    return False


def _contains_http_string(obj) -> bool:
    if isinstance(obj, str):
        return "http" in obj
    if isinstance(obj, dict):
        return any(_contains_http_string(v) for v in obj.values())
    if isinstance(obj, list):
        return any(_contains_http_string(v) for v in obj)
    return False


def _fingerprint(data: dict) -> str:
    payload = {k: v for k, v in data.items() if k != "fingerprint"}
    blob = json.dumps(payload, sort_keys=True, separators=(",", ":")).encode("utf-8")
    return hashlib.sha256(blob).hexdigest()


def _semantic_ok(data: object) -> bool:
    if not isinstance(data, dict):
        return False
    required = {
        "experiment_mode",
        "max_runtime_minutes",
        "max_memory_mb",
        "n_seeds",
        "primary_metric",
        "steps",
        "meta",
        "fingerprint",
    }
    if set(data.keys()) != required:
        return False
    if data.get("experiment_mode") != "offline_ab":
        return False
    if data.get("max_runtime_minutes") != 12:
        return False
    if data.get("max_memory_mb") != 256:
        return False
    if data.get("n_seeds") != 5:
        return False
    if data.get("primary_metric") != "f1":
        return False
    meta = data.get("meta")
    if not isinstance(meta, dict) or set(meta.keys()) != {"author", "schema_version"}:
        return False
    if meta.get("author") != "offline":
        return False
    if meta.get("schema_version") != "2026.07.15":
        return False
    steps = data.get("steps")
    if not isinstance(steps, list) or len(steps) != 4:
        return False
    ids = []
    budgets = []
    for step in steps:
        if not isinstance(step, dict) or set(step.keys()) != {"id", "budget_ms"}:
            return False
        if not isinstance(step["id"], str) or not isinstance(step["budget_ms"], int):
            return False
        ids.append(step["id"])
        budgets.append(step["budget_ms"])
    if ids != REQUIRED_STEP_IDS:
        return False
    if sum(budgets) != 1000:
        return False
    for key in FORBIDDEN_KEYS:
        if _contains_key(data, key):
            return False
    if _contains_http_string(data):
        return False
    fp = data.get("fingerprint")
    if not isinstance(fp, str) or fp != _fingerprint(data):
        return False
    return True


def evaluate(workspace: Path) -> int:
    path = workspace / "protocol.json"
    gold = golden_path()
    if not path.is_file() or not gold.is_file():
        return 0
    try:
        got = path.read_bytes()
        exp = gold.read_bytes()
    except OSError:
        return 0
    # Byte-exact canonical encoding required (pretty-printed JSON fails).
    if got != exp:
        return 0
    try:
        data = json.loads(got.decode("utf-8"))
    except (UnicodeDecodeError, json.JSONDecodeError):
        return 0
    return 1 if _semantic_ok(data) else 0


def _oracle_fix(workspace: Path) -> None:
    shutil.copy2(golden_path(), workspace / "protocol.json")


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
        # fail-closed: pretty-print of the same object must not pass
        data = json.loads(golden_path().read_text(encoding="utf-8"))
        (pass_ws / "protocol.json").write_text(json.dumps(data, indent=2) + "\n", encoding="utf-8")
        assert evaluate(pass_ws) == 0
        # fail-closed: wrong fingerprint on otherwise canonical shape
        bad = dict(data)
        bad["fingerprint"] = "0" * 64
        canon_bad = json.dumps(bad, sort_keys=True, separators=(",", ":")) + "\n"
        (pass_ws / "protocol.json").write_text(canon_bad, encoding="utf-8")
        assert evaluate(pass_ws) == 0
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
