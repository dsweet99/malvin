#!/usr/bin/env bash
# Run fast_task.py solve for every task listed by `python ops/fast_task.py tasks`.
# Logs (stdout + stderr) go to _logs/log-<task_name>
# (or _logs-cursor/ with --agent=cursor, or _logs-main/ with --main).

set -uo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$ROOT"

agent="malvin"
use_main=0
while [[ $# -gt 0 ]]; do
  case "$1" in
    --agent=*)
      agent="${1#--agent=}"
      shift
      ;;
    --agent)
      if [[ $# -lt 2 ]]; then
        echo "--agent requires a value (malvin|cursor)" >&2
        exit 2
      fi
      agent="$2"
      shift 2
      ;;
    --main)
      use_main=1
      shift
      ;;
    *)
      echo "Unknown option: $1" >&2
      exit 2
      ;;
  esac
done

agent="$(printf '%s' "$agent" | tr '[:upper:]' '[:lower:]')"
case "$agent" in
  malvin|cursor) ;;
  *)
    echo "Unknown --agent=$agent (expected malvin|cursor)" >&2
    exit 2
    ;;
esac

if [[ $use_main -eq 1 && "$agent" != "malvin" ]]; then
  echo "--agent=$agent and --main are mutually exclusive" >&2
  exit 2
fi

if [[ $use_main -eq 1 ]]; then
  log_dir="_logs-main"
  mode_flag=(--main)
elif [[ "$agent" == "cursor" ]]; then
  log_dir="_logs-cursor"
  mode_flag=(--agent=cursor)
else
  log_dir="_logs"
  mode_flag=()
fi

mkdir -p "$log_dir"

status=0
while IFS= read -r task || [[ -n "$task" ]]; do
  [[ -z "$task" || "$task" =~ ^[[:space:]]*# ]] && continue
  log="${log_dir}/log-${task}"
  if [[ -f "$log" ]]; then
    echo "Skipping ${task}: ${log} already exists"
    continue
  fi
  echo "Running solve for ${task} -> ${log}"
  python ops/fast_task.py solve "${mode_flag[@]}" "$task" >"$log" 2>&1
  rc=$?
  if [[ $rc -ne 0 ]]; then
    echo "FAILED: ${task} (exit ${rc}; see ${log})" >&2
    status=1
  fi
done < <(python ops/fast_task.py tasks)

exit "$status"
