#!/usr/bin/env bash
# Rust test gate for malvin: selective difftests when indexes are warm, else full nextest/cargo test.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

INDEX_ROOT="${CARGO_DIFFTESTS_INDEX_ROOT:-difftests-index-root}"

if [[ "${MALVIN_FORCE_FULL_RUST_TESTS:-}" == "1" ]]; then
  :
elif command -v cargo-difftests >/dev/null && [[ -d "$INDEX_ROOT" ]]; then
  exec ./admin/difftests_rerun_dirty.sh "$@"
fi

if command -v cargo-nextest >/dev/null || cargo nextest --version >/dev/null 2>&1; then
  cargo nextest run --partition hash:1/2 "$@"
  exec cargo nextest run --partition hash:2/2 "$@"
fi

exec cargo test "$@"
