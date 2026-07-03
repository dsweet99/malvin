# Distributed concepts in malvin (supplement)

This glossary documents cross-cutting ideas that malvin implements across many modules but does not encapsulate in a single owning type. These concepts are not covered in `concepts.md` §1–§7. Section numbers §1–§4 are local to this file.

---

## §1 — Outer KPop gate-loop exit contract

### Problem it solves

The outer `KPopEngine` gate loop must decide when to stop iterating. Exit depends on workflow-specific rules: whether the mpc plan file (`_kpop/mpc_plan.md`) contains exactly `DONE`, and whether workspace quality gates must pass before the loop terminates. Operators and workflow authors need predictable stop conditions without each call site re-implementing the same predicates.

### Where it lives

- `src/kpop_engine/run_loop_exit.rs` — `GateLoopExitCtx`, `mpc_plan_early_exit`, `gates_pass_for_exit`
- `src/kpop_engine/behavior.rs` — `KPopHardConstraints`, `KPopHardConstraintsExit` presets per workflow (gate requirement, checks restore)
- `src/kpop_engine/run_loop.rs` — iteration driver, carry-forward dotfile restore before each snapshot
- `src/kpop_progression/mpc_plan.rs` — `mpc_plan_declares_done`, `strip_mpc_plan_done_on_disk`
- `src/kpop_progression/counters.rs` — `agent_declared_success` (delegates to mpc plan `DONE`)
- `src/kpop_log_protocol/mod.rs` — `ExperimentLog` step-heading parsing (`## Step N — KPop`)

When the mpc plan declares `DONE` and gates pass (where required), the outer gate loop exits early via `mpc_plan_early_exit`.

### Why there is no single type

Exit predicates, mpc plan `DONE` checks, and gate re-run at exit are separate functions wired through `GateLoopExitCtx`. `KPopHardConstraints` is a config bundle describing per-workflow gate requirements, not a state machine that owns iteration progress.

### Related typing aids

`GateLoopExitCtx` bundles references needed at exit time. `KPopHardConstraints` and `KPopHardConstraintsExit` label workflow presets. `ExperimentLog` parses hypothesis-step headings but does not drive loop exit. Each aid documents a slice of the contract; none coordinates the full exit lifecycle.

---

## §2 — Dual-contract observability

### Problem it solves

Malvin emits two parallel output channels with different trust contracts. Narrative output (`stdout` / `stdout.log`) is lossy and human-oriented, tagged with who-prefixes for skimming. Audit output (`trace.jsonl`) is machine-authoritative ACP-shaped JSONL for downstream tooling. Consumers must know which channel answers which question: exit codes, LLM usage, shrink/fork events → audit; human skimming and vocabulary parity → narrative.

### Where it lives

- `src/observability/mod.rs` — module docs, `ObservabilityChannel`, `NarrativeWhoTag`, `is_audit_only_session_update`
- `src/output/` — narrative emission to stdout and `stdout.log`
- `src/agent_backend/mini/trace.rs` — mini-specific audit emission helpers
- `src/agent_backend/mini/acp_trace_shim.rs` — synthetic ACP-shaped `session/update` lines for audit
- `VISION.md` — mini/non-mini log parity requirement

Adjacent artifact `prompts.log` holds outgoing prompt bodies; it is not part of the two-channel model.

### Why there is no single type

Emission is scattered across output, trace, and shim modules. Each call site writes to its channel directly. `ObservabilityChannel` labels targets but does not wrap, dispatch, or enforce write ordering across channels.

### Related typing aids

`ObservabilityChannel` enum (`Narrative`, `Audit`) names the two channels. `NarrativeWhoTag` labels who-prefixes on narrative lines. `AuditEventKind` (alias for `SyntheticAcpSessionUpdate`) catalogs audit record kinds. None form a unified observability facade.

---

## §3 — Workspace quality gate execution

### Problem it solves

Before and after agent work and at loop exit, malvin must discover gate commands from `.malvin/checks`, prepare the workspace (`kiss clamp`), run commands sequentially (one at a time in the sandbox), log results to `quality_gates.log`, and surface failures to agents via prompt markdown and `review.md`. Workflows share this pipeline but differ in when gates run and whether dotfiles are restored around them.

### Where it lives

- `src/repo_gates/mod.rs` — discovery, default checks, prompt markdown for gate failures
- `src/cli/repo_checks/gate_run.rs` — `run_repo_workspace_gates`, sequential command execution
- `src/cli/workflow_kpop_shared.rs` — `run_kpop_workspace_gates` with dotfile restore sandwich (pre-gate restore → repair → run → post-gate restore)
- `src/workflow_context.rs` — injects `quality_gates` output into agent prompts

Gate commands run one at a time to respect sandbox memory limits; overlapping heavy checks in a single shell invocation are unsafe.

### Why there is no single type

Discovery, workspace preparation, execution, logging, and prompt formatting are separate modules. `RepoGateOutput` and gate failure types are local to CLI check code. Nothing owns the full discover → prepare → execute → log → prompt pipeline as one coordinator.

### Related typing aids

Gate failure markdown builders live in `repo_gates`. These label subsets of the pipeline; runtime orchestration remains in `workflow_kpop_shared` and `gate_run`.

---

## §4 — Session dotfile snapshot-restore policy

### Problem it solves

Agent sessions mutate workspace dotfiles (`.malvin/checks`, `.kissconfig`, `VISION.md`, etc.). Malvin must snapshot before agent work, restore after gates or session end, repair kiss-clamp damage to backed-up content, and carry backups across gate-loop iterations without poisoning the next snapshot. Without this policy, agent edits to config files would persist into subsequent iterations or gate runs.

### Where it lives

- `src/session_dotfile_backup/mod.rs` — `SessionDotfileBackups`, `snapshot`, `restore`, `restore_excluding_malvin_checks`
- `src/session_dotfile_backup/gate_restore_merge.rs` — merge backed-up bytes with on-disk state for gate runs
- `src/session_dotfile_backup/gate_restore_repair.rs` — repair clamp-damaged dotfiles in bundles and on disk
- `src/session_dotfile_backup/gate_restore_checks.rs` — checks-specific restore helpers
- `src/kpop_engine/run_loop.rs` — `restore_carry_forward_before_iteration_snapshot` across iterations
- `src/kpop_engine/behavior.rs` — `restore_malvin_checks_after_session` per workflow

The restore sandwich enforced in `workflow_kpop_shared.rs` (pre-gate restore → repair → run gates → post-gate restore) is the operational heart of this concept.

### Why there is no single type

`SessionDotfileBackups` bundles per-file backup state but does not own the full restore sandwich or iteration carry-forward logic. Snapshot timing, merge rules, repair passes, and checks-specific exclusions are enforced at separate call sites in the KPop loop and gate runner.

### Related typing aids

`SessionDotfileBackups` struct with per-slot `DotfileBackupState`. `DotfileBackupPayload` captures bytes and backup path. `merge_for_gate_restore` and `repair_clamp_damaged_dotfiles_on_disk` handle subsets of restore policy. None coordinate the full snapshot → agent → gate → restore lifecycle.
