#!/usr/bin/env bash
# Initial (or full refresh) profiling-data collection for cargo-difftests.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

INDEX_ROOT="${CARGO_DIFFTESTS_INDEX_ROOT:-difftests-index-root}"
DIFFTESTS_ROOT="${CARGO_DIFFTESTS_ROOT:-target/tmp/difftests}"

if ! command -v cargo-difftests >/dev/null; then
  echo "cargo-difftests not found; install with:" >&2
  echo "  rustup component add llvm-tools-preview --toolchain nightly" >&2
  echo "  cargo +nightly install cargo-difftests --git https://github.com/dnbln/cargo-difftests --locked" >&2
  exit 1
fi

exec cargo +nightly difftests collect-profiling-data \
  --compile-index \
  --and-clean \
  --index-root="$INDEX_ROOT" \
  --index-strategy=always-and-clean \
  --root="$DIFFTESTS_ROOT" \
  "$@"
