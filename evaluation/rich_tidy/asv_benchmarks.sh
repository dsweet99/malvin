#!/usr/bin/env bash
set -euo pipefail

if [[ ! -f "asv.conf.json" ]]; then
    echo "error: run this script from the rich repository root" >&2
    exit 1
fi

if [[ -n "${PYTHON:-}" ]]; then
    python_bin="$PYTHON"
elif command -v python3 >/dev/null 2>&1; then
    python_bin="python3"
elif command -v python >/dev/null 2>&1; then
    python_bin="python"
else
    echo "error: could not find python3 or python" >&2
    exit 1
fi

if ! "$python_bin" -m asv --version >/dev/null 2>&1; then
    "$python_bin" -m pip install asv
fi

"$python_bin" -m asv machine --yes
"$python_bin" -m asv run -E existing --set-commit-hash HEAD
"$python_bin" -m asv show --details -E existing HEAD
