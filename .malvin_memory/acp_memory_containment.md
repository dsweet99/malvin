# ACP memory containment and spawn warnings

TRIGGER: containment unavailable, memory containment warn, cgroup memory limit
ADVICE: Constant `CONTAINMENT_UNAVAILABLE_WARN` and `emit_containment_unavailable_warn()` live in `src/acp_memory_containment/mod.rs` (stderr via `print_log_warning`). At spawn, call `emit_containment_unavailable_warn_after_spawn(ContainmentUnavailableWarnAtSpawn { … })` from `src/acp/session_spawn.inc` after `complete_containment_after_spawn`, before `take_stdio_pipes`.
CONFIDENCE: 0

TRIGGER: verbose containment, log_full_outgoing_prompts, acp_verbose
ADVICE: CLI `--verbose` maps to `SharedOpts.verbose` → `AgentIoOptions.log_full_outgoing_prompts` → `AcpSpawnArgs` (`src/cli/code_flow_a.inc`, `src/acp/ops_body_spawn.inc`). Gate the containment warn on `log_full_outgoing_prompts`, not `acp_verbose` (RPC/trace coalescing; often `false` at spawn).
CONFIDENCE: 0

TRIGGER: kiss boolean_parameters, two bool parameters, spawn warn gate
ADVICE: Kiss allows at most one `bool` parameter per function. Do not add `fn gate(verbose: bool, active: bool)` for spawn warn policy; use a small struct (`ContainmentUnavailableWarnAtSpawn`) or inline the `if` in `session_spawn.inc`.
CONFIDENCE: 0

TRIGGER: session_spawn.inc, spawn_acp_session, where spawn warn
ADVICE: Live ACP spawn and containment warn wiring are in `src/acp/session_spawn.inc` (pulled in via `session_types.rs`), not only in `acp_memory_containment/mod.rs`.
CONFIDENCE: 1
