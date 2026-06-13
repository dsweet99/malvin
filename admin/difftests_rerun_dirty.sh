#!/usr/bin/env bash
# Analyze indexes and rerun tests whose executed code changed since last collection.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

INDEX_ROOT="${CARGO_DIFFTESTS_INDEX_ROOT:-difftests-index-root}"
DIFFTESTS_ROOT="${CARGO_DIFFTESTS_ROOT:-target/tmp/difftests}"

if ! command -v cargo-difftests >/dev/null; then
  echo "cargo-difftests not found; see admin/difftests_collect.sh" >&2
  exit 1
fi

export CARGO_DIFFTESTS_EXTRA_ARGS="--compile-index,--and-clean,--index-root=${INDEX_ROOT},--root=${DIFFTESTS_ROOT}"

exec cargo +nightly difftests rerun-dirty-from-indexes --index-root="$INDEX_ROOT" "$@"
