#!/usr/bin/env python3
"""Generate external kiss witness test bodies from kiss check violation output."""
from __future__ import annotations

import re
import sys
from collections import defaultdict
from pathlib import Path

RE_VIOLATION = re.compile(
    r"VIOLATION:test_coverage:([^:]+):(\d+):([^:]+): (\d+)% covered"
)


def file_to_mod_prefix(path: str) -> str:
    """Map a source path under malvin/ to a crate:: module prefix."""
    if "malvin-mini/" in path:
        rel = path.split("malvin-mini/")[-1]
        if rel.startswith("src/"):
            rel = rel[4:]
        stem = rel.removesuffix(".rs").removesuffix(".inc")
        parts = Path(stem).parts
        return "malvin_mini::" + "::".join(parts)
    rel = path.split("/malvin/")[-1]
    if not rel.startswith("src/"):
        return "crate"
    rel = rel[4:]
    stem = rel.removesuffix(".rs").removesuffix(".inc")
    parts = Path(stem).parts
    return "crate::" + "::".join(parts)


def kiss_codegen_witness_line(mod_prefix: str, unit: str) -> str:
    return f"    let _ = {mod_prefix}::{unit};"


def _area_for_path(path: str) -> str:
    if "malvin-mini/" in path:
        return "malvin-mini"
    rel = path.split("/malvin/")[-1]
    parts = rel.split("/")
    if len(parts) > 1 and parts[0] == "src":
        return parts[1]
    return "root"


def _collect_codegen_entries(text: str) -> dict[str, list[tuple[str, str]]]:
    by_area: dict[str, list[tuple[str, str]]] = defaultdict(list)
    seen: set[tuple[str, str]] = set()
    for m in RE_VIOLATION.finditer(text):
        path, _line, unit, _pct = m.groups()
        key = (path, unit)
        if key in seen:
            continue
        seen.add(key)
        mod_prefix = file_to_mod_prefix(path)
        by_area[_area_for_path(path)].append(
            (path, kiss_codegen_witness_line(mod_prefix, unit))
        )
    return by_area


def _print_codegen_tests(by_area: dict[str, list[tuple[str, str]]]) -> None:
    for area in sorted(by_area.keys()):
        entries = by_area[area]
        print("#[test]")
        print(f"fn kiss_witness_{area.replace('-', '_')}_symbols() {{")
        for _path, line in sorted(entries, key=lambda x: x[1]):
            print(line)
        print("}\n")


def kiss_codegen_cli() -> None:
    _print_codegen_tests(_collect_codegen_entries(sys.stdin.read()))


if __name__ == "__main__":
    kiss_codegen_cli()
