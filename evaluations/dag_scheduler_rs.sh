#!/bin/bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]:-$0}")" && pwd)"
MALVIN_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
MALVIN_EVAL_TIMEOUT_SECS="${MALVIN_EVAL_TIMEOUT_SECS:-300}"
(
  cd "$MALVIN_ROOT"
  cargo build --release -q
)
export PATH="$MALVIN_ROOT/target/release:$PATH"

WORKDIR=$(mktemp -d)
HOME_DIR=$(mktemp -d)
LOG="$WORKDIR/malvin_code.log"
echo "Working in: $WORKDIR (HOME=$HOME_DIR)"

export HOME="$HOME_DIR"

cleanup() {
  rm -rf "$WORKDIR" "$HOME_DIR"
}

fail() {
  local message="$1"
  echo "FAIL: $message"
  cleanup
  exit 1
}

run_ok() {
  local jobs_json="$1"
  local workers="$2"
  local expected="$3"
  local out_file err_file code output

  out_file="$(mktemp)"
  err_file="$(mktemp)"
  set +e
  timeout "$MALVIN_EVAL_TIMEOUT_SECS" cargo run --quiet --release -- schedule --workers "$workers" "$jobs_json" >"$out_file" 2>"$err_file"
  code=$?
  set -e

  if [[ "$code" -ne 0 ]]; then
    fail "expected success for $jobs_json"
  fi
  if [[ -s "$err_file" ]]; then
    fail "expected no stderr for $jobs_json"
  fi
  output="$(<"$out_file")"
  if [[ "$output" != "$expected" ]]; then
    fail "unexpected schedule output for $jobs_json. expected $expected got $output"
  fi

  rm -f "$out_file" "$err_file"
}

run_fail() {
  local jobs_json="$1"
  local out_file err_file err_line

  out_file="$(mktemp)"
  err_file="$(mktemp)"
  set +e
  timeout "$MALVIN_EVAL_TIMEOUT_SECS" cargo run --quiet --release -- schedule --workers 2 "$jobs_json" >"$out_file" 2>"$err_file"
  local code=$?
  set -e

  if [[ "$code" -eq 0 ]]; then
    fail "expected failure for $jobs_json"
  fi
  if [[ -s "$out_file" ]]; then
    fail "expected no stdout for failing input $jobs_json"
  fi
  if ! IFS= read -r err_line <"$err_file"; then
    fail "expected ERR: message for failing input $jobs_json"
  fi
  if [[ ! "$err_line" =~ ^ERR:.+ ]]; then
    fail "stderr must match ERR:<message> for $jobs_json"
  fi
  if [[ "$(awk 'END { print NR }' "$err_file")" -ne 1 ]]; then
    fail "expected exactly one-line stderr for $jobs_json"
  fi

  rm -f "$out_file" "$err_file"
}

cd "$WORKDIR"
git init
malvin init rust

cat > grounding.md << 'EOF'
# Project grounding

## Objective

Build a deterministic offline DAG scheduler.

## Constraints
- Code is written in Rust.
- `cargo run --release -- schedule --workers <N> <jobs.json>` writes a single JSON array to stdout and exits 0.
- `jobs.json` is an array of jobs with fields `id` (string), `duration_ms` (positive integer), and `deps` (array of job IDs).
- Validate input: duplicate ids, unknown dependencies, negative or zero duration, and dependency cycles are errors.
- On validation error, print one line `ERR:<message>` to stderr and exit non-zero.
- The scheduler is deterministic: ties are resolved by shortest duration then lexicographically smallest id, worker ids are assigned lowest free index first.
- Output is JSON objects sorted by `start` then `job` then `worker`, each object has fields `job`, `worker`, `start_ms`, `end_ms`.
- No external scheduling crates may be used.
- `src/lib.rs` contains core logic and `src/main.rs` is a thin CLI wrapper.
- Include unit tests for validation and scheduler behavior.
EOF

timeout "$MALVIN_EVAL_TIMEOUT_SECS" malvin code --trust-the-plan --no-learn "Implement the scheduler from grounding.md with robust tests, clippy-clean code, and passing checks."

cd "$WORKDIR"

cargo clippy --all-targets --all-features -- \
  -D warnings \
  -W clippy::pedantic \
  -W clippy::nursery \
  -W clippy::cargo \
  -A clippy::must_use_candidate \
  -A clippy::missing_errors_doc \
  -A clippy::missing_panics_doc >"$LOG" 2>&1 || {
  cat "$LOG"
  fail "cargo clippy failed"
}

cargo test >>"$LOG" 2>&1 || {
  cat "$LOG"
  fail "cargo test failed"
}

cat > "$WORKDIR/jobs_ok.json" << 'JSON'
[
  {"id":"ingest","duration_ms":4,"deps":[]},
  {"id":"render","duration_ms":2,"deps":["ingest"]},
  {"id":"notify","duration_ms":1,"deps":["ingest"]},
  {"id":"archive","duration_ms":1,"deps":["render","notify"]}
]
JSON

cat > "$WORKDIR/jobs_cycle.json" << 'JSON'
[
  {"id":"a","duration_ms":1,"deps":["c"]},
  {"id":"b","duration_ms":1,"deps":["a"]},
  {"id":"c","duration_ms":1,"deps":["b"]}
]
JSON

cat > "$WORKDIR/jobs_bad_dep.json" << 'JSON'
[
  {"id":"a","duration_ms":3,"deps":["missing"]}
]
JSON

run_ok "$WORKDIR/jobs_ok.json" 2 '[{"job":"ingest","worker":0,"start_ms":0,"end_ms":4},{"job":"notify","worker":1,"start_ms":4,"end_ms":5},{"job":"render","worker":0,"start_ms":4,"end_ms":6},{"job":"archive","worker":0,"start_ms":6,"end_ms":7}]'

run_fail "$WORKDIR/jobs_cycle.json"
run_fail "$WORKDIR/jobs_bad_dep.json"

RUN1=$(mktemp)
RUN2=$(mktemp)
timeout "$MALVIN_EVAL_TIMEOUT_SECS" cargo run --quiet --release -- schedule --workers 2 "$WORKDIR/jobs_ok.json" >"$RUN1"
timeout "$MALVIN_EVAL_TIMEOUT_SECS" cargo run --quiet --release -- schedule --workers 2 "$WORKDIR/jobs_ok.json" >"$RUN2"
if [[ "$(cat "$RUN1")" != "$(cat "$RUN2")" ]]; then
  rm -f "$RUN1" "$RUN2"
  fail "scheduler output is not deterministic"
fi
rm -f "$RUN1" "$RUN2"

echo "EVAL_PASS"
cleanup
