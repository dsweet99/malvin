#!/usr/bin/env bash
# Fail when any Rust or Python unit test exceeds the per-test duration budget.
# When invoked from `.malvin/checks`, docker must be available; the checks line sets
# DEEPSWE_SKIP_DOCKER_SELFTESTS=0 so docker-marked pytest tests run under budget.
set -euo pipefail

THRESHOLD="${MALVIN_TEST_MAX_SECS:-1.500}"
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

export NEXTEST_EXPERIMENTAL_LIBTEST_JSON=1
export DEEPSWE_SKIP_DOCKER_SELFTESTS="${DEEPSWE_SKIP_DOCKER_SELFTESTS:-1}"

fail=0

echo "=== timing gate: Rust (threshold ${THRESHOLD}s) ==="
rust_json="$(mktemp)"
NEXTEST_EXPERIMENTAL_LIBTEST_JSON=1 cargo nextest run --message-format libtest-json --test-threads 1 >"${rust_json}" 2>&1 || {
  cat "${rust_json}"
  rm -f "${rust_json}"
  exit 1
}
rust_report="$(python3 -c "
import json, sys
threshold = float('${THRESHOLD}')
slow = []
for line in open('${rust_json}'):
    line = line.strip()
    if not line:
        continue
    try:
        obj = json.loads(line)
    except json.JSONDecodeError:
        continue
    if obj.get('type') != 'test' or obj.get('event') not in ('ok', 'failed'):
        continue
    duration = obj.get('exec_time')
    if duration is not None and duration >= threshold:
        slow.append((duration, obj.get('name', '?')))
for duration, name in sorted(slow, reverse=True):
    print(f'  {duration:.3f}s  {name}')
print(len(slow))
" | tail -1)"
rust_count="${rust_report}"
rust_lines="$(python3 -c "
import json
threshold = float('${THRESHOLD}')
slow = []
for line in open('${rust_json}'):
    line = line.strip()
    if not line:
        continue
    try:
        obj = json.loads(line)
    except json.JSONDecodeError:
        continue
    if obj.get('type') != 'test' or obj.get('event') not in ('ok', 'failed'):
        continue
    duration = obj.get('exec_time')
    if duration is not None and duration >= threshold:
        slow.append((duration, obj.get('name', '?')))
for duration, name in sorted(slow, reverse=True):
    print(f'  {duration:.3f}s  {name}')
")"
rm -f "${rust_json}"
if [[ "${rust_count}" =~ ^[0-9]+$ ]] && [[ "${rust_count}" -gt 0 ]]; then
  echo "Rust tests over budget:"
  echo "${rust_lines}"
  fail=1
else
  echo "Rust: all tests under ${THRESHOLD}s"
fi

echo "=== timing gate: Python (threshold ${THRESHOLD}s) ==="
py_out="$(mktemp)"
if ! pytest tests --durations=0 -q >"${py_out}" 2>&1; then
  cat "${py_out}"
  rm -f "${py_out}"
  exit 1
fi
py_slow="$(python3 -c "
import re
threshold = float('${THRESHOLD}')
text = open('${py_out}').read()
slow = []
for duration, nodeid in re.findall(r'([\d.]+)s call\s+(\S+)', text):
    if float(duration) >= threshold:
        slow.append((float(duration), nodeid))
for duration, nodeid in sorted(slow, reverse=True):
    print(f'  {duration:.3f}s  {nodeid}')
print(len(slow))
" | tail -1)"
py_lines="$(python3 -c "
import re
threshold = float('${THRESHOLD}')
text = open('${py_out}').read()
slow = []
for duration, nodeid in re.findall(r'([\d.]+)s call\s+(\S+)', text):
    if float(duration) >= threshold:
        slow.append((float(duration), nodeid))
for duration, nodeid in sorted(slow, reverse=True):
    print(f'  {duration:.3f}s  {nodeid}')
")"
rm -f "${py_out}"
if [[ "${py_slow}" =~ ^[0-9]+$ ]] && [[ "${py_slow}" -gt 0 ]]; then
  echo "Python tests over budget:"
  echo "${py_lines}"
  fail=1
else
  echo "Python: all tests under ${THRESHOLD}s"
fi

if [[ "${fail}" -ne 0 ]]; then
  echo "timing gate FAILED (threshold ${THRESHOLD}s)"
  exit 1
fi
echo "timing gate passed"
