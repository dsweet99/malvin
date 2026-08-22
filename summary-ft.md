# Fast-task results

Requested model: `codex:gpt-5.6-terra`.
Command: `./ops/fast_task.py solve --model=codex:gpt-5.6-terra TASK_NAME`.
Results root: `/home/dsweet/.malvin_home/fast_task_results`.

| Task | Reward | malvin exit | agent_seconds | Result |
|---|---:|---:|---:|---|
| FT-01 | 1 | 0 | 110.8 | PASS |
| FT-03 | 1 | 0 | 177.9 | PASS |
| FT-05 | 1 | 0 | 189.3 | PASS |
| FT-08 | 1 | 0 | 381.8 | PASS |
| FT-09 | 1 | 0 | 159.0 | PASS |
| FT-12 | 1 | 0 | 147.8 | PASS |
| FT-13 | 1 | 0 | 141.9 | PASS |
| FT-15 | 1 | 0 | 111.9 | PASS |
| FT-17 | 1 | 0 | 157.0 | PASS |
| FT-20 | 1 | 0 | 156.7 | PASS |
| FT-24 | 1 | 0 | 417.7 | PASS |
| FT-25 | 1 | 0 | 276.5 | PASS |
| FT-26 | 1 | 0 | 175.4 | PASS |
| FT-27 | 1 | 0 | 323.5 | PASS |
| FT-28 | 0 | 0 | 309.8 | FAIL |
| FT-29 | 1 | 0 | 298.8 | PASS |
| FT-30 | 1 | 0 | 446.3 | PASS |
| FT-31 | 1 | 0 | 189.1 | PASS |
| FT-32 | 1 | 0 | 517.9 | PASS |
| FT-33 | 1 | 0 | 229.6 | PASS |
| FT-34 | 1 | 0 | 286.5 | PASS |
| FT-35 | 0 | 124 | 607.1 | FAIL (timed out) |
| FT-36 | 1 | 0 | 179.2 | PASS |

Totals: 21/23 passed. Mean agent_seconds: 260.5. Sum agent_seconds: 5991.5.

Notes:

- FT-28 finished with malvin exit 0 but failed the host grader.
- FT-35 hit the 600s agent timeout (`timed_out: true`, exit 124).
- Artifacts are under `/home/dsweet/.malvin_home/fast_task_results/<TASK>/<timestamp>/`.
