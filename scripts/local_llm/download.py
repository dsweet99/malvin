#!/usr/bin/env python3
"""Download a Hugging Face model into malvin's local model cache."""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path


def strip_moe_gate_quant_entries(model_dir: Path) -> int:
    """Remove MoEGate paths from config quantization (Nemotron / JANG)."""
    config_path = model_dir / "config.json"
    if not config_path.is_file():
        return 0
    config = json.loads(config_path.read_text())
    quant = config.get("quantization")
    if not isinstance(quant, dict):
        return 0
    gate_keys = [k for k in quant if isinstance(k, str) and "gate" in k.lower()]
    if not gate_keys:
        return 0
    bak = model_dir / "config.json.bak_pre_gate_strip"
    if not bak.exists():
        bak.write_text(config_path.read_text())
    for key in gate_keys:
        del quant[key]
    config_path.write_text(json.dumps(config, indent=2) + "\n")
    return len(gate_keys)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repo", required=True, help="Hugging Face repo id")
    parser.add_argument("--out", type=Path, required=True, help="Destination directory")
    parser.add_argument(
        "--loader",
        default="mlx_lm",
        choices=("mlx_lm", "jang"),
        help="Loader kind (jang triggers MoEGate strip after download)",
    )
    args = parser.parse_args()

    try:
        from huggingface_hub import snapshot_download
    except ImportError:
        print(
            "huggingface_hub is required. Install scripts/local_llm/requirements.txt",
            file=sys.stderr,
        )
        return 1

    out: Path = args.out.expanduser().resolve()
    out.mkdir(parents=True, exist_ok=True)
    print(f"downloading repo={args.repo} out={out}", flush=True)
    snapshot_download(repo_id=args.repo, local_dir=str(out))
    if args.loader == "jang":
        removed = strip_moe_gate_quant_entries(out)
        if removed:
            print(f"stripped {removed} MoEGate quantization entries", flush=True)
    if not (out / "config.json").is_file():
        print(f"download incomplete: missing {out / 'config.json'}", file=sys.stderr)
        return 1
    print("download complete", flush=True)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
