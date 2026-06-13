#!/usr/bin/env bash
# Install prerequisites for cargo-difftests (nightly + llvm-tools + cargo-difftests from git).
set -euo pipefail

if ! command -v rustup >/dev/null; then
  echo "rustup not found; install from https://rustup.rs" >&2
  exit 1
fi

if ! rustup toolchain list | grep -q '^nightly'; then
  echo "Installing nightly toolchain..."
  rustup toolchain install nightly
fi

echo "Adding llvm-tools-preview to nightly..."
rustup component add llvm-tools-preview --toolchain nightly

if ! command -v cargo-cov >/dev/null; then
  echo "Installing cargo-binutils (provides cargo-cov)..."
  cargo install cargo-binutils
fi

if ! command -v cargo-difftests >/dev/null; then
  echo "Installing cargo-difftests from git (requires nightly)..."
  cargo +nightly install cargo-difftests \
    --git https://github.com/dnbln/cargo-difftests \
    --locked
fi

echo "cargo-difftests ready. Verify with: ./admin/difftests_verify.sh"
