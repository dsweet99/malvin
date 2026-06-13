#!/usr/bin/env bash
# Quick smoke check that cargo-difftests prerequisites are installed.
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

check command -v cargo-difftests
if rustup component list --toolchain nightly | grep -q 'llvm-tools.*installed'; then
  echo "ok: llvm-tools-preview (nightly)"
else
  echo "FAIL: llvm-tools-preview not installed on nightly" >&2
  fail=1
fi
check command -v cargo-cov
check test -f .cargo/config.toml
check rg -q 'profile.difftests' .cargo/config.toml
check test -x admin/difftests_collect.sh
check test -x admin/difftests_rerun_dirty.sh
check test -x admin/malvin_rust_test_gate.sh

if [[ "$fail" -ne 0 ]]; then
  echo "Run ./admin/difftests_install.sh to fix missing pieces." >&2
  exit 1
fi

echo "cargo-difftests setup looks good."
