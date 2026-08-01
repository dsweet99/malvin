#!/usr/bin/env bash
PORT=8080
ENV_D="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/env.d"
if [[ -d "$ENV_D" ]]; then
  shopt -s nullglob
  for f in "$ENV_D"/*.sh; do
    # shellcheck disable=SC1090
    source "$f"
  done
  shopt -u nullglob
fi
