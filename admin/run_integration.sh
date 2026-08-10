#!/usr/bin/env bash
# Run malvin integration tests (fast contract + opt-in live suites).
#
# Does not set API keys. Export OPENROUTER_API_KEY (and cursor-agent auth)
# in your environment before running live OpenRouter / ACP cases.
#
# Usage:
#   ./admin/run_integration.sh           # fast contract + network live gates
#   ./admin/run_integration.sh --local   # also set MALVIN_LIVE_LOCAL=1 (Metal/GPU)
#
# Live suites use cargo-nextest --ignored. This script exports live gates; when a
# gate is set, missing keys/prereqs fail the test (fail-closed). Unset gates may
# still early-return inside ignored bodies.

set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

run_local=0
for arg in "$@"; do
  case "$arg" in
    --local) run_local=1 ;;
    -h|--help)
      sed -n '2,14p' "$0" | sed 's/^# \?//'
      exit 0
      ;;
    *)
      echo "unknown argument: $arg (try --help)" >&2
      exit 2
      ;;
  esac
done

if ! command -v cargo-nextest >/dev/null 2>&1 && ! cargo nextest --version >/dev/null 2>&1; then
  echo "error: cargo-nextest is required" >&2
  exit 1
fi

echo "=== fast: agent_backend_contract ==="
cargo nextest run -E 'binary(agent_backend_contract)'

# Live gates only — never assign OPENROUTER_API_KEY / cursor keys here.
export MALVIN_LIVE_TRANSPORT=1
export MALVIN_LIVE_MINI=1
export MALVIN_LIVE_DEFER_ENRICH=1
if [[ "$run_local" -eq 1 ]]; then
  export MALVIN_LIVE_LOCAL=1
  echo "=== MALVIN_LIVE_LOCAL=1 (Metal/GPU local: suites) ==="
else
  unset MALVIN_LIVE_LOCAL || true
fi

echo "=== live: transport_live (ignored) ==="
cargo nextest run -E 'test(transport_live)' -- --ignored

echo "=== live: agent_backend_live (ignored) ==="
cargo nextest run -E 'test(agent_backend_live)' -- --ignored

echo "=== live: mini_live (ignored) ==="
cargo nextest run mini_live -- --ignored

echo "=== live: defer_enrich_live (ignored) ==="
cargo nextest run defer_enrich_live -- --ignored

echo "=== integration runs finished ==="
