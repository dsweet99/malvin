#!/bin/bash
set -euo pipefail

TMP=$(mktemp -d)
echo "Working in: $TMP"

cd "$TMP"
git init

malvin init python

cat > grounding.md << 'EOF'
# Project grounding

## Objective

Say Hello to the world.

## Constraints
- Code is written in Python.
- `./ops/hello.py` prints "Hello, world!"
EOF

malvin code "Write this app"
