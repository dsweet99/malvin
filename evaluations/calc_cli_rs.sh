#!/bin/bash
set -euo pipefail

TMP=$(mktemp -d)
echo "Working in: $TMP"

cd "$TMP"
git init

malvin init rust

cat > grounding.md << 'EOF'
# Project grounding

## Objective

Build a command-line calculator for arithmetic expressions.

## Constraints
- Code is written in Rust.
- `cargo run --release -- "<expr>"` evaluates integer expressions.
- Supported syntax includes whitespace, parentheses, binary `+ - * /`, and unary `-`.
- Division truncates toward zero.
- Invalid input must print `ERR:<message>` to stderr and exit non-zero.
- No parser generator crates.
EOF

malvin code "Implement a calculator CLI from the grounding with robust tests and make all checks pass."

run_expr() {
  cargo run --release -- "$1"
}

assert_stdout() {
  local expr="$1"
  local expected="$2"
  local out_file err_file out
  out_file="$(mktemp)"
  err_file="$(mktemp)"
  set +e
  cargo run --quiet --release -- "$expr" >"$out_file" 2>"$err_file"
  local code=$?
  set -e
  out="$(<"$out_file")"
  if [[ $code -ne 0 ]]; then
    echo "FAIL: expression '$expr' unexpectedly failed"
    rm -f "$out_file" "$err_file"
    exit 1
  fi
  if [[ -s "$err_file" ]]; then
    echo "FAIL: expression '$expr' unexpectedly wrote to stderr: '$(<"$err_file")'"
    rm -f "$out_file" "$err_file"
    exit 1
  fi
  if [[ "$out" != "$expected" ]]; then
    echo "FAIL: expression '$expr' expected '$expected' got '$out'"
    rm -f "$out_file" "$err_file"
    exit 1
  fi
  rm -f "$out_file" "$err_file"
}

assert_invalid() {
  local expr="$1"
  local out_file err_file err_line
  out_file="$(mktemp)"
  err_file="$(mktemp)"
  set +e
  cargo run --quiet --release -- "$expr" >"$out_file" 2>"$err_file"
  local code=$?
  set -e
  if [[ $code -eq 0 ]]; then
    echo "FAIL: expression '$expr' unexpectedly succeeded"
    rm -f "$out_file" "$err_file"
    exit 1
  fi
  if [[ -s "$out_file" ]]; then
    echo "FAIL: expression '$expr' unexpectedly wrote to stdout: '$(<"$out_file")'"
    rm -f "$out_file" "$err_file"
    exit 1
  fi
  if ! IFS= read -r err_line <"$err_file"; then
    echo "FAIL: expression '$expr' stderr must be exactly ERR:<message> with non-empty message"
    rm -f "$out_file" "$err_file"
    exit 1
  fi
  if IFS= read -r _ <&3 3<"$err_file"; then
    echo "FAIL: expression '$expr' stderr must be a single line"
    rm -f "$out_file" "$err_file"
    exit 1
  fi
  if [[ "$err_line" == *$'\r'* ]]; then
    echo "FAIL: expression '$expr' stderr must not contain carriage returns"
    rm -f "$out_file" "$err_file"
    exit 1
  fi
  if [[ ! "$err_line" =~ ^ERR:[^[:space:]].* ]]; then
    echo "FAIL: expression '$expr' stderr must be exactly ERR:<message> with non-empty message"
    rm -f "$out_file" "$err_file"
    exit 1
  fi
  rm -f "$out_file" "$err_file"
}

assert_stdout "1 + 2*3" "7"
assert_stdout "-(8-10)*4" "8"
assert_stdout "18 / 5" "3"
assert_stdout "-7 / 3" "-2"
assert_stdout "7 / -3" "-2"
assert_stdout "2*(3+(4*5))-7" "39"
assert_invalid "1+("
assert_invalid "2/0"

echo "EVAL_PASS"
