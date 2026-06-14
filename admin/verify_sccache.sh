#!/usr/bin/env bash
# Quick smoke check that sccache is installed and wired into .cargo/config.toml.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

fail=0

check() {
  if "$@"; then
    echo "ok: $*"
  else
    echo "FAIL: $*" >&2
    fail=1
  fi
}

check command -v sccache
check test -f .cargo/config.toml
check rg -q 'rustc-wrapper.*sccache' .cargo/config.toml
check rg -q 'profile\.difftests' .cargo/config.toml

if [[ "$fail" -ne 0 ]]; then
  echo "Run ./admin/sccache_install.sh and ensure .cargo/config.toml exists." >&2
  exit 1
fi

# Start the sccache server if needed; confirm it responds.
sccache --start-server >/dev/null 2>&1 || true
check sccache --show-stats

echo "sccache setup looks good."
