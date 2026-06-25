#!/usr/bin/env bash
# Install sccache for rustc compilation caching.
set -euo pipefail

if command -v sccache >/dev/null; then
  echo "sccache already installed: $(command -v sccache)"
  sccache --version
  exit 0
fi

echo "Installing sccache via cargo install..."
cargo install sccache --locked

echo "sccache ready. Verify with: ./admin/verify_sccache.sh"
