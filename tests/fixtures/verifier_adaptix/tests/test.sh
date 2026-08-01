#!/bin/bash
set -euo pipefail
cd /app
if [ "${1:-}" = "base" ]; then
  python -m pytest tests/test_aliases.py -q
elif [ "${1:-}" = "new" ]; then
  python -m pytest tests/test_aliases.py -q
fi
