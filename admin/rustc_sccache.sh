#!/bin/sh
# Cargo rustc-wrapper (set in .cargo/config.toml). Runs rustc through sccache
# when it is installed. Set MALVIN_SCCACHE=0 to bypass sccache.
if [ "${MALVIN_SCCACHE:-1}" != 0 ] && command -v sccache >/dev/null 2>&1; then
  exec sccache "$@"
fi
exec "$@"
