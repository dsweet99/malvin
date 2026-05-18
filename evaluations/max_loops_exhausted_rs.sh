#!/bin/bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$ROOT/target}"
export CARGO_BIN_EXE_malvin="${CARGO_BIN_EXE_malvin:-$CARGO_TARGET_DIR/debug/malvin}"

cd "$ROOT"
cargo build -q --bin malvin
cargo test -q --test cli_parity_code default_max_loops_exhausts_fanout_review_without_lgtm
