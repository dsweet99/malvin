#!/usr/bin/env bash
set -euo pipefail

if [[ "${MALVIN_DAG_SCHEDULER_HARNESS_ENABLE:-0}" != "1" ]]; then
  echo "MIGRATION_NOTICE: schedule CLI removed from this repo; this legacy harness is skipped."
  echo "Set MALVIN_DAG_SCHEDULER_HARNESS_ENABLE=1 to run the legacy schedule flow."
  exit 0
fi

WORKDIR=$(mktemp -d)
echo "Working in: $WORKDIR"
cd "$WORKDIR"
git init

malvin init rust

cat > grounding.md << 'EOF'
# Project grounding

## Objective

Build a deterministic DAG job scheduler CLI and library for offline plan generation.

## Constraints

- Code is written in Rust.
- `cargo run --release -- schedule --workers <N> <jobs.json>` emits a single JSON array on stdout.
- Each job is `{ "id": "string", "duration": 1..1000, "deps": ["id", ...] }` and input is a JSON array.
- Implement 3 subsystems in tandem:
  1. Input parser + validation (syntax, duplicates, unknown dependencies).
  2. DAG planner (dependency graph + cycle detection).
  3. Deterministic scheduler (ready-queue ordering and worker assignment).
- Ties must be deterministic: ready jobs ordered by (duration asc, id lexicographically), workers assigned lowest available index first.
- If a schedule is generated, stdout is JSON array of `{ "job": "...", "worker": 0-based int, "start": int, "end": int }`, sorted by `start`, then `worker`, then `job`.
- On validation or cycle errors, exit non-zero and print exactly one line `ERR:<message>` to stderr.
- Use deterministic, seeded behavior; same input must always produce identical output.
- Add unit/integration tests for at least one scheduler edge case and one cycle case.
- Public library surface lives under `src/lib.rs` and binary is a thin CLI wrapper.
EOF

malvin code "Implement the deterministic DAG scheduler per grounding.md with full tests and passing checks."

run_ok() {
  local input_json="$1"
  local expected="$2"
  local out_file err_file
  out_file="$(mktemp)"
  err_file="$(mktemp)"

  set +e
  cargo run --quiet --release -- schedule --workers 2 "$input_json" >"$out_file" 2>"$err_file"
  local code=$?
  set -e

  if [[ $code -ne 0 ]]; then
    echo "FAIL: command unexpectedly failed"
    echo "stdout:"; cat "$out_file"
    echo "stderr:"; cat "$err_file"
    exit 1
  fi
  if [[ -s "$err_file" ]]; then
    echo "FAIL: expected no stderr; got $(cat "$err_file")"
    exit 1
  fi
  if [[ "$(cat "$out_file")" != "$expected" ]]; then
    echo "FAIL: stdout mismatch"
    echo "got:    $(cat "$out_file")"
    echo "expect: $expected"
    exit 1
  fi
}

run_fail() {
  local input_json="$1"
  local out_file err_file
  out_file="$(mktemp)"
  err_file="$(mktemp)"

  set +e
  cargo run --quiet --release -- schedule --workers 2 "$input_json" >"$out_file" 2>"$err_file"
  local code=$?
  set -e

  if [[ $code -eq 0 ]]; then
    echo "FAIL: command unexpectedly succeeded"
    exit 1
  fi
  if [[ -s "$out_file" ]]; then
    echo "FAIL: expected no stdout"
    cat "$out_file"
    exit 1
  fi
  if ! grep -q '^ERR:.' "$err_file"; then
    echo "FAIL: expected ERR:<message> on stderr"
    cat "$err_file"
    exit 1
  fi
}

JOB_OK=$(mktemp)
cat > "$JOB_OK" <<'JSON'
[
  {"id":"A","duration":4,"deps":[]},
  {"id":"B","duration":2,"deps":["A"]},
  {"id":"C","duration":3,"deps":["A"]},
  {"id":"D","duration":1,"deps":["B","C"]}
]
JSON

assert_output='[{"job":"A","worker":0,"start":0,"end":4},{"job":"B","worker":0,"start":4,"end":6},{"job":"C","worker":1,"start":4,"end":7},{"job":"D","worker":0,"start":7,"end":8}]'
run_ok "$JOB_OK" "$assert_output"

JOB_CYCLE=$(mktemp)
cat > "$JOB_CYCLE" <<'JSON'
[
  {"id":"A","duration":1,"deps":["B"]},
  {"id":"B","duration":1,"deps":["A"]}
]
JSON
run_fail "$JOB_CYCLE"

JOB_BAD_DEP=$(mktemp)
cat > "$JOB_BAD_DEP" <<'JSON'
[
  {"id":"A","duration":1,"deps":["NOPE"]}
]
JSON
run_fail "$JOB_BAD_DEP"

OUT1=$(mktemp)
OUT2=$(mktemp)
cargo run --release -- schedule --workers 2 "$JOB_OK" >"$OUT1"
cargo run --release -- schedule --workers 2 "$JOB_OK" >"$OUT2"
if [[ "$(cat "$OUT1")" != "$(cat "$OUT2")" ]]; then
  echo "FAIL: scheduler output is non-deterministic"
  echo "first:  $(cat "$OUT1")"
  echo "second: $(cat "$OUT2")"
  exit 1
fi

echo "EVAL_PASS"
