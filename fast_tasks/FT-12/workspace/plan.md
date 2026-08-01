# FT-12: Remove broken env override

Edit only files in this workspace.

## Task
Service stub `svc/start.sh` sources `svc/env.sh` then writes `var/port` with the listen port. Expected port is `8080`. Currently it writes `18080`. An override under `svc/env.d/` forces PORT.

Make `./svc/start.sh` result in `var/port` content `8080` by editing/removing files under `svc/env.d/` and/or fixing `svc/env.sh`.

## Rules
- Do **not** change `svc/start.sh`.
- No network.

## Done when
After running `./svc/start.sh`, `var/port` contains exactly `8080`.

Keep the service bootstrap intact. Prefer deleting or neutralizing the bad override rather than rewriting the starter script.
Stay inside this workspace directory. Do not read or edit parent directories (including any grader or golden files outside this folder).
