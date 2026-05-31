#!/bin/bash
# Pre-commit hook: fail if any .rs or .py files are untracked, or if
# default_prompts/do_header.md exists but is not tracked (this malvin monorepo
# embeds it via include_str!; the default_repo/ template omits this check for
# generic consumer projects).
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

req_md="default_prompts/do_header.md"
req_do_header="default_prompts/header_do.md"
if [ -f "$req_md" ] && [ -z "$(git ls-files -- "$req_md" 2>/dev/null || true)" ]; then
    echo "Error: $req_md exists on disk but is not tracked. It is required for the build (include_str! in src/prompts/defaults.rs). Run: git add $req_md"
    exit 1
fi
if [ -f "$req_do_header" ] && [ -z "$(git ls-files -- "$req_do_header" 2>/dev/null || true)" ]; then
    echo "Error: $req_do_header exists on disk but is not tracked. It is required for the build (include_str! in src/prompts/defaults.rs). Run: git add $req_do_header"
    exit 1
fi

exit 0
