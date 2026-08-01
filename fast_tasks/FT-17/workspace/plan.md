# FT-17: Offline experiment checklist under resource caps

Edit only files in this workspace.

## Task
Design a protocol checklist for a mock A/B measurement on a preloaded dataset (no web). Write `protocol.json` satisfying schema `schema/protocol.schema.json` (Draft-07).

Required constraints:
- max_runtime_minutes ≤ 30
- max_memory_mb ≤ 512
- n_seeds == 3
- primary_metric in {"auroc","f1"}
- steps[] ids unique and must include step id `blind_labels`
- forbid field `download_url` anywhere (including nested)

Fill `protocol.json` only. Schema is in the workspace. See `README_CONTEXT.md` for scientific context.

## Rules
- No network / no downloads.
- Do not rename the deliverable.

## Done when
`protocol.json` validates and meets the constraints above.
Stay inside this workspace directory. Do not read or edit parent directories (including any grader or golden files outside this folder).
