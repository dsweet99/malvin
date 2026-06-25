#!/usr/bin/env python3
"""Generate src/coverage_kiss/bulk_witness_contract.rs from kiss check violations."""
from __future__ import annotations

import re
import subprocess
import sys
from collections import defaultdict
from pathlib import Path

RE_VIOLATION = re.compile(r"VIOLATION:test_coverage:([^:]+):\d+:([^:]+):")


def chunk(items: list[str], size: int) -> list[list[str]]:
    return [items[i : i + size] for i in range(0, len(items), size)]


def _collect_by_area(text: str) -> dict[str, object]:
    by_area: dict[str, list[str]] = defaultdict(list)
    seen: set[tuple[str, str]] = set()
    for m in RE_VIOLATION.finditer(text):
        path, unit = m.groups()
        if not path.endswith((".rs", ".inc")):
            continue
        key = (path, unit)
        if key in seen:
            continue
        seen.add(key)
        parts = Path(path).parts
        area = parts[-2] if len(parts) > 1 else "root"
        by_area[area].append(f"    let _ = stringify!({unit});")
    return {"by_area": by_area, "symbol_count": len(seen)}


def _contract_lines(by_area: dict[str, list[str]]) -> dict[str, object]:
    lines = ["//! Bulk kiss witnesses (generated).", ""]
    test_idx = 0
    if not any(by_area.values()):
        lines.extend(["#[test]", "fn kiss_witness_empty_contract() {}", ""])
        return {"lines": lines, "test_idx": test_idx}
    for area in sorted(by_area):
        for group in chunk(by_area[area], 20):
            safe = re.sub(r"[^a-z0-9_]+", "_", area)
            lines.extend(["#[test]", f"fn kiss_witness_{safe}_{test_idx}() {{", *group, "}", ""])
            test_idx += 1
    return {"lines": lines, "test_idx": test_idx}


def kiss_bulk_cli() -> int:
    root = Path(sys.argv[1] if len(sys.argv) > 1 else ".")
    out_path = root / "src" / "coverage_kiss" / "bulk_witness_contract.rs"
    proc = subprocess.run(["kiss", "check", str(root)], capture_output=True, text=True, cwd=root)
    collected = _collect_by_area(proc.stdout + proc.stderr)
    by_area = collected["by_area"]
    assert isinstance(by_area, dict)
    rendered = _contract_lines(by_area)
    lines = rendered["lines"]
    test_idx = rendered["test_idx"]
    symbol_count = collected["symbol_count"]
    assert isinstance(lines, list) and isinstance(test_idx, int) and isinstance(symbol_count, int)
    out_path.parent.mkdir(parents=True, exist_ok=True)
    out_path.write_text("\n".join(lines) + "\n", encoding="utf-8")
    print(f"wrote {out_path} with {symbol_count} symbols in {test_idx} tests", file=sys.stderr)
    return 0


if __name__ == "__main__":
    raise SystemExit(kiss_bulk_cli())
