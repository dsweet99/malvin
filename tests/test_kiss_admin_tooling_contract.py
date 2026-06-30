"""Kiss contract tests for ops/kiss_triage scripts (ops/ is kiss-ignored)."""

from __future__ import annotations

import importlib.util
import io
import os
import sys
from pathlib import Path

_TRIAGE = Path(__file__).resolve().parents[1] / "ops" / "kiss_triage"


def _load_triage_module(stem: str):
    path = _TRIAGE / f"{stem}.py"
    spec = importlib.util.spec_from_file_location(stem, path)
    assert spec and spec.loader
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    return mod


def test_kiss_admin_bulk_witness_gen_helpers():
    mod = _load_triage_module("kiss_bulk_witness_gen")
    assert mod.chunk(["x"], 1) == [["x"]]
    assert mod.chunk(["a", "b", "c"], 2) == [["a", "b"], ["c"]]


def test_kiss_admin_witness_codegen_symbols():
    mod = _load_triage_module("kiss_witness_codegen")
    assert mod.file_to_mod_prefix("/malvin/src/foo.rs") == "crate::foo"
    assert mod.kiss_codegen_witness_line("crate::foo", "Bar").startswith("    let _")
    buf = io.StringIO("VIOLATION:test_coverage:/malvin/src/foo.rs:1:Bar: 0% covered.\n")
    old = sys.stdin
    try:
        sys.stdin = buf
        mod.kiss_codegen_cli()
    finally:
        sys.stdin = old


def test_kiss_admin_violation_manifest_symbols():
    mod = _load_triage_module("kiss_violation_manifest")
    assert mod.bucket("/malvin/src/foo.rs", 85) == "A"
    assert mod.bucket("/malvin/src/foo.inc", 50) == "E"
    buf = io.StringIO(
        "  /malvin/src/foo.rs: 50% (90% required)\n"
        "VIOLATION:test_coverage:/malvin/src/foo.rs:1:bar_fn: 0% covered.\n"
    )
    out = "/tmp/kiss_manifest_test.tsv"
    old_argv = sys.argv
    old_stdin = sys.stdin
    try:
        sys.argv = ["kiss_violation_manifest.py", out]
        sys.stdin = buf
        assert mod.kiss_manifest_cli() == 0
    finally:
        sys.argv = old_argv
        sys.stdin = old_stdin
    assert Path(out).read_text(encoding="utf-8").splitlines()[1].startswith("/malvin/src/foo.rs")


def test_kiss_admin_bulk_witness_gen_main_smoke(tmp_path: Path):
    mod = _load_triage_module("kiss_bulk_witness_gen")
    bin_dir = tmp_path / "bin"
    bin_dir.mkdir()
    fake_kiss = bin_dir / "kiss"
    fake_kiss.write_text(
        "#!/bin/sh\n"
        "echo 'VIOLATION:test_coverage:/malvin/src/foo.rs:1:Bar: 0% covered.'\n",
        encoding="utf-8",
    )
    fake_kiss.chmod(0o755)
    old_argv = sys.argv
    old_path = os.environ.get("PATH")
    try:
        sys.argv = ["kiss_bulk_witness_gen.py", str(tmp_path)]
        os.environ["PATH"] = f"{bin_dir}:{old_path or ''}"
        assert mod.kiss_bulk_cli() == 0
    finally:
        sys.argv = old_argv
        if old_path is None:
            os.environ.pop("PATH", None)
        else:
            os.environ["PATH"] = old_path
    out = tmp_path / "src/coverage_kiss/bulk_witness_contract.rs"
    assert out.is_file()
    assert "kiss_witness_" in out.read_text(encoding="utf-8")
