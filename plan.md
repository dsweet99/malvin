# Plan: Enforce Harbor per-phase timeouts in DeepSWE solve

## User Request

Enforce DeepSWE/Harbor per-task timeouts correctly (separate agent and verifier budgets from `task.toml`), instead of only sizing a combined Modal sandbox ceiling.

## Current State

### Harbor / DeepSWE task config

Each task under `../deep-swe/tasks/<id>/task.toml` declares phase timeouts (Harbor schema):

| Section | Field | Typical value |
|---|---|---|
| `[agent]` | `timeout_sec` | 5400 (90 min) |
| `[verifier]` | `timeout_sec` | 1800 (30 min) |
| `[environment]` | `build_timeout_sec` | 1800 (30 min) |

All 113 tasks in the current dataset use the same numbers; the schema allows per-task variation.

Harbor's native harness treats these as **independent caps**: the agent gets up to `agent.timeout_sec`, then the verifier gets up to `verifier.timeout_sec` on whatever workspace state remains.

### What malvin reads today

`parse_task_dir()` in `ops/deepswe_run.py` loads agent and verifier timeouts into `TaskSpec`:

```186:187:ops/deepswe_run.py
        agent_timeout_sec=float(agent.get("timeout_sec", 5400.0)),
        verifier_timeout_sec=float(verifier.get("timeout_sec", 1800.0)),
```

`build_timeout_sec` is **not** parsed or used anywhere in malvin.

### Solve call path (default vs inner exec)

Default `solve TASK` does **not** call host-side `run_task()`. It goes:

`solve` → `run_modal_solve` → `run_modal_eval` → `run_deepswe_run_in_sandbox` → **`deepswe_run.py run --runtime in-sandbox`** (two execs: `--skip-grade`, then `--grade-only`).

Local Docker and Modal therefore enforce timeouts inside **`run_task()` in-sandbox**, not on the host orchestrator.

### What each phase actually includes (enforcement boundary)

`run_task()` in `ops/deepswe_run.py` (~1772–1821) runs work in this order:

1. `prepare_task_sandbox()` — **always** (can take minutes; pip replay)
2. If `not grade_only`: `write_plan_and_checks`, `ensure_deepswe_malvin_config`, `run_malvin()`
3. If grading: `grade_workspace_native()` (in-sandbox) or `grade_workspace()` (host)

Modal/local Docker split this into two containers/exec:

| Exec | Flags | Work included |
|---|---|---|
| Agent | `--skip-grade` | prep + plan + config + malvin |
| Grade | `--grade-only` | prep + verifier |

Harbor's agent budget must cover **all of row 1**; verifier budget must cover **all of row 2** — not just `run_malvin()` or `bash test.sh`.

### Where phases run today (no per-phase enforcement)

| Path | Agent | Verifier | Timeout today |
|---|---|---|---|
| **Default `solve`** → Modal | `sandbox.exec` → in-sandbox `run --skip-grade` | second exec → `--grade-only` | Single `modal.Sandbox.create(timeout=…)` from `agent_sandbox_timeout_sec()` (sum + 900s headroom); inner execs unbounded |
| **`solve --local`** | Docker → in-sandbox `run --skip-grade` | second container → `--grade-only` | None on host `subprocess.run(docker …)` |
| **`run --runtime in-sandbox`** | full agent block in `run_task()` | grade block in `run_task()` | None |
| **`run --runtime host`** | `run_malvin()` on host | `grade_workspace()` (Docker) | None |

Subprocess call sites with no `timeout=` today: `run_malvin()` (~1423), `grade_workspace_native()` (~1253), `grade_workspace()` (~1314), `_relay_subprocess_stdout()` for hello (~1347–1366).

### Modal sandbox timeout helper

```1694:1700:ops/deepswe_modal.py
def agent_sandbox_timeout_sec(spec: Any, *, skip_grade: bool) -> int:
    agent = float(getattr(spec, "agent_timeout_sec", 5400.0))
    if skip_grade:
        return int(agent + 900)
    verifier = float(getattr(spec, "verifier_timeout_sec", 1800.0))
    return int(agent + verifier + 900)
```

Passed to `modal.Sandbox.create(timeout=…)`. Host-side `proc.wait()` on inner execs has no deadline.

**Bug:** `run_modal_eval(grade_only=True)` calls this with `skip_grade=False`, so grade-only runs get `agent + verifier + 900` even though only the verifier exec runs.

### Metadata and exit behavior

- `_build_run_metadata()` has no `timed_out` field today.
- `_exit_from_evaluation()` (`deepswe_run.py` ~1606) and `finalize_modal_eval()` (`deepswe_modal.py` ~2374) both:
  - exit 1 when `grade.pass is False`
  - **exit with agent `exit_code` when nonzero**, even if grade passed

So agent timeout (e.g. exit 124) would fail the CLI after a passing grade unless exit helpers are updated.

### Tests encoding current behavior

- `ops/deepswe_modal.py`: `_test_agent_sandbox_timeout_sec()` asserts 8100 / 6300.
- No test asserts per-phase kill at configured limits.

### Adjacent / out of scope

- `hello --host` calls `run_hello_probe_on_host()` without loading `TaskSpec` — connectivity probe, not a scored solve.
- `build_timeout_sec` unused; `build_local_agent_image()` / Modal `harbor_image()` builds uncapped.

## Requested Changes

1. Enforce `[agent] timeout_sec` on the **full agent exec** (prep + plan + config + malvin), not malvin alone.
2. Enforce `[verifier] timeout_sec` on the **full verifier exec** (prep + grade), not `test.sh` alone.
3. Keep Modal/local-Docker orchestration working: outer sandbox/container lifetime is a **backstop** with enough headroom; inner phase limits match Harbor (agent budget does not consume verifier budget).
4. Record timeout outcomes in metadata (`timed_out`, exit code, configured limits).
5. After agent timeout, **still run grading**; **CLI exit status follows grade** (see Q3).
6. Fix Modal sandbox timeout formulas for graded, skip-grade, and **grade-only** paths.
7. Update self-tests; no live agent runs required.

**Out of scope:** `hello --host`, `build_timeout_sec`, Harbor CLI timeout multipliers.

## Q&A

### Q1. Should `environment.build_timeout_sec` be in scope?

**Answer:** Defer. Agent and verifier phase caps are the solve-path gap. Build timeout applies to `docker build` / Modal image build — follow-up if needed.

### Q2. Where should enforcement live?

**Answer:** Primary enforcement in `ops/deepswe_run.py` inside `run_task()`, using a monotonic **phase deadline** (`agent_deadline` / `verifier_deadline`) that covers prep and subprocess work for that exec. Modal and local Docker already invoke `run_task()` in-sandbox per exec; no duplicate timers in `deepswe_modal.py` except the outer sandbox backstop (Phase 2).

### Q3. On agent timeout, should grading still run? What exit code?

**Answer:** Yes — grade the partial workspace. Mark `agent.timed_out: true` and `exit_code: 124`. **CLI exit follows grade:** if `grade.pass is True`, exit 0; if grade fails or verifier times out, exit 1. Update `_exit_from_evaluation()` and `finalize_modal_eval()` to skip the agent exit-code check when `agent.timed_out` is true (grade already ran). This matches “score the submission, note agent overrun in metadata.”

### Q4. On verifier timeout, what is the outcome?

**Answer:** `pass: false`, `reward: 0`, `timed_out: true`, non-zero verifier exit code (124). Write `verifier.log` with whatever was captured before kill.

### Q5. How to stop malvin + cursor-agent on timeout?

**Answer:** Process-group kill: `start_new_session=True`, on timeout `SIGTERM` then `SIGKILL` to the group. Each subprocess call within a phase gets `timeout_sec=min(remaining_deadline, …)`.

### Q6. Combined `run_task` (host runtime, agent + grade in one invocation)?

**Answer:** Prep at the top counts toward the **agent** deadline only (runs once). Start `agent_deadline` before prep when `not grade_only`; start a fresh `verifier_deadline` before the grade block when `not skip_grade`. Do not double-charge prep to the verifier budget in this path.

## Plan

### Phase 1 — Phase deadlines in `run_task()` + subprocess helper

- [x] Add `_run_with_timeout(cmd, *, cwd, timeout_sec, stream=False)` in `ops/deepswe_run.py`:
  - returns `exit_code`, `timed_out`, `elapsed_sec`, optional output;
  - process-group termination on timeout;
  - maps timeout to exit code 124.
- [x] Add `_remaining_sec(deadline: float) -> float` (floor at 0).
- [x] Refactor `run_task()` phase structure:
  - **`grade_only` exec:** set `verifier_deadline` **before** `prepare_task_sandbox`; each step (prep, grade) uses remaining budget.
  - **Agent exec** (`not grade_only`): set `agent_deadline` **before** prep; prep, plan, config, and `run_malvin()` use remaining budget.
  - **Combined path** (`not grade_only` and `not skip_grade`, e.g. host runtime): agent deadline before prep; after agent completes, new verifier deadline before grade (prep not repeated).
  - On deadline exhausted before a step: skip remaining steps in that phase, set `timed_out: true`, exit code 124.
- [x] Thread remaining budget into `prepare_task_sandbox` (add optional `timeout_sec` or deadline param to its slow subprocess calls in `sandbox_prep.py` — minimal surface: pass deadline into prep and cap `_run_shell` calls).
- [x] Update `run_malvin()`, `grade_workspace_native()`, and `grade_workspace()` to accept `timeout_sec` from caller (already computed from phase deadline).
- [x] On verifier timeout: force `pass: false`, `reward: 0` when no valid reward file.
- [x] Extend `_build_run_metadata()` with configured limits and `timed_out` on agent/grade dicts.
- [x] Update `_exit_from_evaluation()` and `finalize_modal_eval()`: when `agent.get("timed_out")`, do not raise on agent `exit_code`; still exit 1 on `grade.pass is False`.
- [x] Add `_print_evaluation_summary()` lines for `timed_out`.

**Validation:**

- `python ops/deepswe_run.py self-test` passes.
- `_test_run_with_timeout_kills_slow_command()`: `sleep 999` with 1s cap → `timed_out`, elapsed ≈ 1s.
- `_test_run_task_agent_phase_includes_prep()`: mock prep + malvin; assert total wall time capped at `agent_timeout_sec`.
- `_test_exit_after_agent_timeout_grade_pass()`: mock agent `{timed_out: true, exit_code: 124}`, grade `{pass: true}` → `_exit_from_evaluation` does not raise.
- Dry-run unchanged: `python ops/deepswe_run.py run … --dry-run` still prints commands only.

### Phase 2 — Modal outer sandbox backstop

- [x] Replace `agent_sandbox_timeout_sec()` with explicit modes (or add `grade_only: bool`):
  - **Graded solve:** `agent + verifier + INJECT_SLACK + SANDBOX_HEADROOM` (keep **900s headroom** until prep is reliably under phase deadlines; do **not** shrink to 300s).
  - **Skip-grade:** `agent + SANDBOX_HEADROOM`
  - **Grade-only:** `verifier + SANDBOX_HEADROOM` (fix current bug using full sum)
- [x] Pass correct mode from `run_modal_eval()` (`grade_only`, `skip_grade` flags).
- [x] Update `_test_agent_sandbox_timeout_sec()` for new grade-only case; graded/skip-grade values stay 8100/6300 with 900 headroom (+ small inject slack constant if added, e.g. 120s between execs → 8220 graded — document constant in code).
- [x] Do **not** add host-side `proc.wait(timeout=…)` unless Phase 1 inner enforcement proves insufficient.

**Validation:**

- `python ops/deepswe_modal.py --self-test` passes.
- `pytest tests/test_ops_selftest.py -k deepswe_modal` passes.
- `_test_agent_sandbox_timeout_sec()` covers all three modes.

### Phase 3 — Local Docker outer safety (optional)

- [x] Host-side timeouts on `subprocess.run(agent_cmd)` / `subprocess.run(grade_cmd)` in `run_local_eval_in_docker()` (~1201, ~1222): `spec.agent_timeout_sec + slack` and `spec.verifier_timeout_sec + slack` as backstop only.

**Validation:**

- `python ops/deepswe_run.py self-test` passes.
- Docker tests pass when Docker available (`DEEPSWE_SKIP_DOCKER_SELFTESTS=0`).

### Phase 4 — Docs

- [x] Update `ops/deepswe_run.py` module docstring: per-phase enforcement from `task.toml`; note Modal/default path uses in-sandbox `run_task()`.
- [x] One-line comment on `agent_sandbox_timeout_sec` modes in `deepswe_modal.py`.

**Validation:**

- Grep confirms docstring mentions per-phase enforcement and in-sandbox path.
