# Fast tasks

Twenty-two short-horizon, binary-graded micro-eval tasks.

## Layout

Each task id lives under `fast_tasks/<ID>/`:

| Path | Role |
|---|---|
| `workspace/plan.md` | Agent instruction. Run: `malvin fast_tasks/<ID>/workspace/plan.md` so `work_dir` is `workspace/`. |
| `workspace/**` | Prebaked unsolved starter (bugs/stubs only). |
| `grade.py` | Self-contained Harbor-style grader (outside agent work_dir). |
| `goldens/` | Grader-only expected values / hidden tests (not agent-visible when using plan path above). |

Task ids: `FT-01`, `FT-03`, `FT-13`, `FT-12`, `FT-05`, `FT-09`, `FT-17`, `FT-08`, `FT-20`, `FT-15`, `FT-24`, `FT-25`, `FT-26`, `FT-27`, `FT-28`, `FT-29`, `FT-30`, `FT-31`, `FT-32`, `FT-33`, `FT-34`, `FT-35`.

## Grader CLI

```text
python fast_tasks/<ID>/grade.py [--workspace PATH] [--reward-out PATH]
python fast_tasks/<ID>/grade.py --self-test
```

- Default `--workspace`: sibling `./workspace`.
- Default `--reward-out`: `./reward.txt` next to `grade.py`, or `$MALVIN_REWARD_PATH` / `$HARBOR_REWARD_PATH` if set.
- Reward file contents: exactly `0` or `1` plus newline.
- Graders use the active conda/Python env and stdlib (+ pytest where noted). They must not import the malvin package or repo modules.

## Self-tests (graders only; starters stay unsolved)

```text
python fast_tasks/run_selftests.py
```

Each grader’s `--self-test` builds **temporary** FAIL (starter copy) and PASS (oracle applied in-temp) trees. Committed `workspace/` trees remain unfixed.

## Isolation note

Agent invocation uses `workspace/plan.md` so malvin’s work directory is `workspace/`. Graders and `goldens/` sit outside that directory. Agents should not walk to `../` to read grade material.
