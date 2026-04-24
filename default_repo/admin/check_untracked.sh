#!/bin/bash
# Pre-commit hook: fail if any .rs or .py source files are untracked.
#
# Untracked files that match .gitignore (and other standard Git excludes) are
# ignored; we use --exclude-standard so behavior matches `git status`.
#
# Usage: ./admin/check_untracked.sh

set -euo pipefail

have_untracked=0
while IFS= read -r -d '' f; do
    case "$f" in
        *.rs|*.py)
            if [ "$have_untracked" -eq 0 ]; then
                echo "Error: The following source files are not tracked by git:"
                echo
                have_untracked=1
            fi
            printf '%s\n' "$f"
            ;;
    esac
done < <(git ls-files --others --exclude-standard -z)

if [ "$have_untracked" -eq 1 ]; then
    echo
    echo "Please add them with 'git add' or add to .gitignore"
    exit 1
fi

exit 0
