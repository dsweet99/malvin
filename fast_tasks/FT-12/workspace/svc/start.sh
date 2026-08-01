#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
# shellcheck disable=SC1091
source "$ROOT/svc/env.sh"
mkdir -p "$ROOT/var"
printf '%s' "$PORT" > "$ROOT/var/port"
