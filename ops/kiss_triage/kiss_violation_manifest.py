#!/usr/bin/env python3
"""Parse kiss check stdin into a TSV manifest."""
from __future__ import annotations

import re
import sys
from collections import defaultdict


def bucket(path: str, pct: int) -> str:
    if pct >= 80:
        return "A"
    checks = (
        (path.endswith(".inc"), "E"),
        ("malvin-mini/" in path, "F"),
        ("_tests" in path or "_helpers" in path, "D"),
    )
    return next((label for ok, label in checks if ok), "B/C/G")


def _parse_manifest(text: str) -> dict[str, object]:
    file_pct: dict[str, int] = {}
    units: dict[str, list[tuple[str, int, int]]] = defaultdict(list)
    unit_re = re.compile(
        r"VIOLATION:test_coverage:([^:]+):(\d+):([^:]+): (\d+)% covered"
    )
    pct_re = re.compile(r"^\s+([^:]+): (\d+)% \(90% required\)")
    for line in text.splitlines():
        unit_match = unit_re.match(line)
        if unit_match:
            units[unit_match.group(1)].append(
                (unit_match.group(3), int(unit_match.group(2)), int(unit_match.group(4)))
            )
            continue
        pct_match = pct_re.match(line)
        if pct_match:
            file_pct[pct_match.group(1)] = int(pct_match.group(2))
    return {"file_pct": file_pct, "units": units}


def _manifest_rows(
    file_pct: dict[str, int],
    units: dict[str, list[tuple[str, int, int]]],
) -> list[str]:
    rows: list[str] = []
    for path in sorted(units):
        pct = file_pct.get(path, min(u[2] for u in units[path]))
        for unit, line_no, _ in units[path]:
            rows.append(f"{path}\t{pct}\t{unit}\t{line_no}\t{bucket(path, pct)}")
    return rows


def kiss_manifest_cli() -> int:
    out_path = sys.argv[1] if len(sys.argv) > 1 else "/tmp/kiss_manifest.tsv"
    parsed = _parse_manifest(sys.stdin.read())
    file_pct = parsed["file_pct"]
    units = parsed["units"]
    assert isinstance(file_pct, dict) and isinstance(units, dict)
    rows = _manifest_rows(file_pct, units)
    with open(out_path, "w", encoding="utf-8") as f:
        f.write("file\tfile_pct\tunit\tunit_line\tbucket\n")
        f.write("\n".join(rows))
        if rows:
            f.write("\n")
    print(f"wrote {len(rows)} rows to {out_path}", file=sys.stderr)
    return 0


if __name__ == "__main__":
    raise SystemExit(kiss_manifest_cli())
