#!/usr/bin/env bash
set -euo pipefail
python3 -c '
import fcntl
import os
import sys

lock_path = os.path.join(os.environ.get("TMPDIR", "/tmp"), "malvin-nextest-list.lock")
with open(lock_path, "w", encoding="utf-8") as lock_file:
    fcntl.flock(lock_file.fileno(), fcntl.LOCK_EX)
    os.execvp(sys.argv[1], sys.argv[1:])
' "$@"
