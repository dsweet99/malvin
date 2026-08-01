#!/usr/bin/env python3
"""Self-contained Harbor-style grader for FT-17. No malvin/repo imports."""
from __future__ import annotations

import argparse
import json
import os
import shutil
import tempfile
from pathlib import Path


TASK_ID = "FT-17"


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


def _contains_key(obj, key: str) -> bool:
    if isinstance(obj, dict):
        if key in obj:
            return True
        return any(_contains_key(v, key) for v in obj.values())
    if isinstance(obj, list):
        return any(_contains_key(v, key) for v in obj)
    return False


def evaluate(workspace: Path) -> int:
    path = workspace / "protocol.json"
    schema_path = workspace / "schema" / "protocol.schema.json"
    if not path.is_file() or not schema_path.is_file():
        return 0
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
        schema = json.loads(schema_path.read_text(encoding="utf-8"))
    except json.JSONDecodeError:
        return 0
    # Minimal draft-07 checks without external deps
    if not isinstance(data, dict):
        return 0
    for req in schema.get("required", []):
        if req not in data:
            return 0
    try:
        if float(data["max_runtime_minutes"]) > 30:
            return 0
        if float(data["max_memory_mb"]) > 512:
            return 0
        if int(data["n_seeds"]) != 3:
            return 0
    except (TypeError, ValueError):
        return 0
    if data.get("primary_metric") not in {"auroc", "f1"}:
        return 0
    steps = data.get("steps")
    if not isinstance(steps, list) or not steps:
        return 0
    ids = []
    for step in steps:
        if not isinstance(step, dict) or "id" not in step:
            return 0
        ids.append(step["id"])
    if len(ids) != len(set(ids)):
        return 0
    if "blind_labels" not in ids:
        return 0
    if _contains_key(data, "download_url"):
        return 0
    return 1


ORACLE = {
    "max_runtime_minutes": 30,
    "max_memory_mb": 512,
    "n_seeds": 3,
    "primary_metric": "auroc",
    "steps": [
        {"id": "load_data"},
        {"id": "blind_labels"},
        {"id": "score"},
    ],
}


def _oracle_fix(workspace: Path) -> None:
    (workspace / "protocol.json").write_text(json.dumps(ORACLE, indent=2) + "\n", encoding="utf-8")


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
        # fail-closed: download_url forbidden
        bad = dict(ORACLE)
        bad["download_url"] = "http://example"
        (pass_ws / "protocol.json").write_text(json.dumps(bad), encoding="utf-8")
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
