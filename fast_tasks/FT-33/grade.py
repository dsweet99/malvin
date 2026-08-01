#!/usr/bin/env python3
"""Self-contained Harbor-style grader for FT-33. No malvin/repo imports."""
from __future__ import annotations

import argparse
import json
import os
import shutil
import tempfile
from pathlib import Path


TASK_ID = "FT-33"

GOLD = {
    "coap_documentation_id_low": 64998,
    "coap_documentation_id_high": 64999,
    "new_top_level_media_type": "haptics",
    "nuts_level1_regions": 92,
    "nuts_level2_regions": 244,
    "nuts_level3_regions": 1165,
    "time_ordered_uuid_example": "017f22e2-79b0-7cc3-98c4-dc0c0c07398f",
    "answer": 184,
}

INT_KEYS = (
    "coap_documentation_id_low",
    "coap_documentation_id_high",
    "nuts_level1_regions",
    "nuts_level2_regions",
    "nuts_level3_regions",
    "answer",
)
STR_KEYS = (
    "new_top_level_media_type",
    "time_ordered_uuid_example",
)


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


def evaluate(workspace: Path) -> int:
    path = workspace / "answer.json"
    if not path.is_file():
        return 0
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
    except json.JSONDecodeError:
        return 0
    if not isinstance(data, dict):
        return 0
    if set(data.keys()) != set(GOLD.keys()):
        return 0

    for key in INT_KEYS:
        value = data.get(key)
        if not isinstance(value, int) or isinstance(value, bool):
            return 0
        if value != GOLD[key]:
            return 0

    for key in STR_KEYS:
        value = data.get(key)
        if not isinstance(value, str):
            return 0
        if value != GOLD[key]:
            return 0

    # Metamorphic check: derived answer must match the CoAP×NUTS formula.
    lo = data["coap_documentation_id_low"]
    hi = data["coap_documentation_id_high"]
    n1 = data["nuts_level1_regions"]
    if data["answer"] != (hi - lo + 1) * n1:
        return 0
    return 1


def _oracle_fix(workspace: Path) -> None:
    (workspace / "answer.json").write_text(
        json.dumps(GOLD, indent=2) + "\n",
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

        # Reject internally inconsistent payloads.
        bad = dict(GOLD)
        bad["answer"] = GOLD["answer"] + 1
        (pass_ws / "answer.json").write_text(json.dumps(bad) + "\n", encoding="utf-8")
        assert evaluate(pass_ws) == 0, "inconsistent answer must fail"

        # Near-miss: Experimental Use CoAP IDs instead of documentation pair.
        near = dict(GOLD)
        near["coap_documentation_id_low"] = 65000
        near["coap_documentation_id_high"] = 65001
        near["answer"] = (65001 - 65000 + 1) * GOLD["nuts_level1_regions"]
        (pass_ws / "answer.json").write_text(json.dumps(near) + "\n", encoding="utf-8")
        assert evaluate(pass_ws) == 0, "experimental CoAP IDs must fail"

        # Near-miss: NUTS 2027 counts.
        near2 = dict(GOLD)
        near2["nuts_level1_regions"] = 91
        near2["nuts_level2_regions"] = 242
        near2["nuts_level3_regions"] = 1170
        near2["answer"] = (GOLD["coap_documentation_id_high"] - GOLD["coap_documentation_id_low"] + 1) * 91
        (pass_ws / "answer.json").write_text(json.dumps(near2) + "\n", encoding="utf-8")
        assert evaluate(pass_ws) == 0, "NUTS 2027 counts must fail"

        # Near-miss: wrong top-level token.
        near3 = dict(GOLD)
        near3["new_top_level_media_type"] = "haptic"
        (pass_ws / "answer.json").write_text(json.dumps(near3) + "\n", encoding="utf-8")
        assert evaluate(pass_ws) == 0, "haptic typo must fail"

        # Per-key mutation: every field must be load-bearing against GOLD.
        for key, wrong in (
            ("time_ordered_uuid_example", "017f22e2-79b0-6cc3-98c4-dc0c0c07398f"),
            ("nuts_level2_regions", GOLD["nuts_level2_regions"] + 1),
            ("nuts_level3_regions", GOLD["nuts_level3_regions"] + 1),
            ("coap_documentation_id_low", GOLD["coap_documentation_id_low"] - 1),
            ("coap_documentation_id_high", GOLD["coap_documentation_id_high"] + 1),
            ("nuts_level1_regions", GOLD["nuts_level1_regions"] + 1),
            ("new_top_level_media_type", "application"),
            ("answer", GOLD["answer"] + 7),
        ):
            flipped = dict(GOLD)
            flipped[key] = wrong
            if key in ("coap_documentation_id_low", "coap_documentation_id_high", "nuts_level1_regions"):
                flipped["answer"] = (
                    flipped["coap_documentation_id_high"]
                    - flipped["coap_documentation_id_low"]
                    + 1
                ) * flipped["nuts_level1_regions"]
            (pass_ws / "answer.json").write_text(
                json.dumps(flipped) + "\n", encoding="utf-8"
            )
            assert evaluate(pass_ws) == 0, f"single-key flip of {key} must fail"

        # Reject float-smuggled integers and uppercase UUID.
        for bad_payload in (
            {**GOLD, "answer": 184.0},
            {**GOLD, "nuts_level1_regions": "92"},
            {**GOLD, "time_ordered_uuid_example": GOLD["time_ordered_uuid_example"].upper()},
        ):
            (pass_ws / "answer.json").write_text(
                json.dumps(bad_payload) + "\n", encoding="utf-8"
            )
            assert evaluate(pass_ws) == 0, f"type/case smuggle must fail: {bad_payload!r}"

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
