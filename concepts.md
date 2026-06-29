# Distributed concepts in malvin

This glossary documents cross-cutting ideas that malvin implements across many modules but does not encapsulate in a single owning type. Section numbers match the `see concepts.md §N` references in `src/` module doc comments. All seven sections §1–§7 are covered.

---

## §1 — Nested budget scopes

### Problem it solves

Agent work in malvin can fail or stall at many independent layers: HTTP transport blips, per-turn LLM calls, bash subprocess limits, whole-loop gate retries, context-shrink passes, outer KPop gate iterations, and ACP spawn retries. Each layer needs its own retry budget and CLI knobs so operators can tune resilience without one global counter swallowing all failures.

### Where it lives

- `src/nested_budget_scopes/mod.rs` — names each layer and documents CLI flags
- `src/cli/shared_opts.rs` — defines `--max-loops`, `--mini-max-http-turns`, `--mini-max-gate-retries`, `--max-acp-retries`, and related flags
- `src/cli/workflow_kpop_shared.rs` — outer KPop loop iteration count via `BudgetScopeLayer::effective_outer_loop_iterations`
- `src/agent_backend/mini/loop_http_retry.rs` — transport retry budget for OpenRouter HTTP
- `src/agent_backend/mini/loop_inner_phases.rs` — per-prompt HTTP turn counting against `--mini-max-http-turns`
- `src/agent_backend/mini/client_gate_retry.rs` — whole-loop gate retry after workspace gate failure

### Why there is no single type

There is no unified budget tree or coordinator object. Each enforcement site owns its counter locally, reads its limit from CLI or config, and applies `single_attempt` or billing-immediate rules independently. Layers stack conceptually (transport inside HTTP turn inside gate iteration inside outer loop) but nothing aggregates remaining budget across them.

### Related typing aids

`BudgetScopeLayer` enum labels the seven layers in stable concept order and maps each to its primary CLI flag when one exists. It documents enforcement semantics (`respects_single_attempt`, `billing_fails_immediately`) but is not invoked as a runtime coordinator.

---

## §2 — Transcript–workspace fork state

### Problem it solves

Gate-iteration retries must checkpoint transcript length and workspace manifest hash at each attempt boundary. Operators and audit consumers need to distinguish cumulative-transcript retries (append a divergence observation) from workspace-snapshot retries (truncate back to the checkpoint). The `miniRetryFork` trace event records each fork attempt for post-hoc review.

### Where it lives

- `src/fork_state/mod.rs` — `ForkState` capture and divergence checks
- `src/agent_backend/mini/retry_fork.rs` — `RetryForkLedger`, `MiniRetryStrategy`, `ForkOutcome`
- `src/agent_backend/mini/client_gate_retry_attempt.rs` — checkpoint capture at attempt start, ledger build on outcome
- `src/agent_backend/mini/client_gate_retry.rs` — strategy application across gate retries
- `src/agent_backend/mini/trace_audit.rs` — `emit_retry_fork` writes `miniRetryFork` audit lines

`ForkState::capture` records message count and workspace manifest hash. On retry, `MiniRetryStrategy::WorkspaceSnapshot` truncates the transcript to the checkpoint length; `MiniRetryStrategy::CumulativeTranscript` appends a divergence observation with the manifest hash instead.

### Why there is no single type

Checkpoint capture, ledger serialization, strategy selection, and transcript mutation live in separate gate-retry modules. Nothing owns the full fork lifecycle as one object; each site reads or writes the paired checkpoint fields independently.

### Related typing aids

`ForkState` struct with `capture`, `transcript_matches`, `workspace_matches`, and `is_diverged`. `RetryForkLedger` bundles per-attempt metadata for audit emission. `MiniRetryStrategy` and `ForkOutcome` label retry branches. `is_diverged` is tested API surface for documentation; gate retry applies strategies explicitly rather than calling it at runtime.

---

## §3 — ACP trace impersonation

### Problem it solves

Malvin supports two agent backends: a real ACP subprocess and an in-process `--mini` OpenRouter loop. Users and downstream tools expect the same audit artifacts regardless of backend. VISION.md requires that `--mini` logs look basically the same as non-mini logs. The mini backend therefore writes synthetic ACP-shaped JSON-RPC `session/update` lines into `trace.jsonl` so audit consumers see familiar envelopes.

### Where it lives

- `src/acp_trace_impersonation/mod.rs` — names each synthetic update kind
- `src/agent_backend/mini/acp_trace_shim.rs` — builds and appends ACP-shaped `session/update` messages (message chunks, tool calls, mini extensions)
- `src/agent_backend/mini/trace.rs` — mini-specific audit emission helpers
- `src/observability/mod.rs` — dual-channel model: narrative (`stdout.log`) vs audit (`trace.jsonl`); audit is machine-authoritative
- `VISION.md` — mini/non-mini log parity requirement

Standard ACP updates (`agent_message_chunk`, `agent_thought_chunk`, `tool_call`, `tool_call_update`) share wire shape with real ACP runs. Mini-only extensions (`miniUsage`, `miniTerminal`, `miniHttpExchange`, `miniPromptShrink`, `miniRetryFork`, etc.) ride on `agent_message_chunk` envelopes so existing parsers can ignore or specialize on them.

### Why there is no single type

Emission is scattered across the trace shim and mini loop modules. Each call site constructs JSON inline and appends to `AcpJsonlTrace`. There is no impersonation session object that owns the full trace lifecycle or enforces update ordering.

### Related typing aids

`SyntheticAcpSessionUpdate` enum catalogs every emitted update kind and maps standard kinds to ACP `sessionUpdate` wire keys. It labels emission sites for documentation; it does not wrap or dispatch trace writes.

---

## §4 — Prompt stratification

### Problem it solves

Malvin assembles agent prompts from many sources: embedded markdown templates, workflow-specific headers, the user's request file, gate-loop blocks, mini constraint snippets, and placeholder-filled context (log paths, review artifacts, gate output). Workflows differ in which layers they include and in what order, but all must produce a single plain-text prompt string for the agent.

### Where it lives

- `src/prompt_stratification/mod.rs` — `join_strata`, `join_labeled_strata`, `WorkflowRenderContext`
- `src/workflow_context.rs` — populates placeholder keys (artifact paths, gate logs) into a context map
- `src/cli/do_flow_prompt.rs`, `src/kpop_turn_prompts.rs`, `src/kpop_engine/mpc_planner.rs` — per-workflow prompt recipes using labeled strata
- `src/cli/*/prep.rs` — workflow-specific context preparation (code, tidy, delight, revise, explain flows)
- `default_prompts/` — embedded markdown templates and constraint blocks loaded by `PromptStore`

Each recipe calls `join_labeled_strata` with an ordered list of `(PromptStratum, text)` pairs. Non-empty parts are trimmed and joined with blank lines. There is no intermediate representation beyond strings.

### Why there is no single type

Prompts are flat concatenated strings, not an AST or template tree. Layer order and inclusion are explicit at each workflow's recipe site. Adding a new stratum means updating individual recipes, not extending a central prompt builder.

### Related typing aids

`PromptStratum` enum names the conceptual layers (`EmbeddedTemplate`, `PlaceholderContext`, `WorkflowHeader`, `UserRequest`, `GateLoopBlock`, `MiniConstraints`). `WorkflowRenderContext` is a typed `HashMap<String, String>` for placeholder substitution. Both aid typing and documentation; neither enforces recipe structure at compile time.

---

## §5 — Session-scoped sandbox spawn policy

### Problem it solves

While a coder session is active, every subprocess malvin starts must respect workspace isolation and memory limits: children run in isolated process groups, glibc arena use is capped, prior session PIDs must be dead before a new spawn, descendant RSS is monitored against the sandbox limit, and ACP/mini session startup is serialized per work directory. These rules apply ambiently to all spawns during a session, not per-command at the call site.

### Where it lives

- `src/session_sandbox_policy/mod.rs` — names each policy aspect
- `src/malvin_sandbox.rs` — `malvin_std_command`, `malvin_tokio_command`, active session slot, process-group isolation, `MALLOC_ARENA_MAX=2`
- `src/acp_spawn_lock.rs` — `acquire_acp_spawn_lock` / `release_acp_spawn_lock` serialize session startup
- `src/process_group_rss/` — descendant USS monitoring (`malvin_session_rss_bytes`) against workspace memory limit
- `src/acp/process_group_mem_watch.rs` — RSS watch integration during active sessions

Callers use `malvin_std_command` or `malvin_tokio_command` rather than raw `Command::new`. The active sandbox session mutex holds the current process group, baseline PID set, work directory, and lock slot.

### Why there is no single type

Policy is ambient session state plus scattered enforcement hooks, not a constructed policy object passed through the call graph. Some aspects apply inside `malvin_std_command` (process group, malloc cap); others apply at session boundaries (dead-before-spawn, RSS monitor, spawn lock). Nothing bundles all five aspects into one configurable struct.

### Related typing aids

`SandboxSpawnPolicyAspect` enum lists the five aspects and notes which ones `malvin_std_command` applies directly. Production code references the enum at enforcement sites for documentation cross-links; runtime behavior remains in `malvin_sandbox` and related modules.

---

## §6 — Coder-prompt phase machine

### Problem it solves

Inside each `run_coder_prompt`, the mini inner loop must balance open-ended investigation against a controlled shutdown. The loop moves through three phases: **Investigate** (run bash fences, gather evidence), **WindDown** (finish remaining work with tighter constraints), and **Terminal** (emit final outcome and stop). Phase transitions depend on turn classification, bash execution, HTTP turn budgets, and explicit terminal signals from the model.

### Where it lives

- `src/coder_prompt_phase/mod.rs` — names the three phases
- `src/agent_backend/mini/loop_inner.rs` — outer phase dispatch (`LoopPhase::Investigate` → `WindDown` → done)
- `src/agent_backend/mini/loop_inner_phases.rs` — `run_investigate_turn`, `run_wind_down_turn`, step enums (`InvestigateStep`, `WindDownStep`)
- `src/agent_backend/mini/loop_inner_classify.rs` — classifies assistant output into continue, bash, done, or wind-down actions
- `src/agent_backend/mini/terminal.rs` — terminal emission with `MiniPhase` and `MiniTerminalReason` recorded in audit trace

Investigate turns increment HTTP and bash counters; exhausting `--mini-max-http-turns` during investigate triggers a partial-transcript terminal. Wind-down turns share the same HTTP budget but apply different classification rules. Terminal records which phase the loop was in at exit.

### Why there is no single type

Phase transition logic is split across `loop_inner`, `loop_inner_phases`, classification, bash handling, and terminal emission. `LoopPhase` in the inner loop and `MiniPhase` in the typing module are related but not unified into one state machine struct with explicit transition tables.

### Related typing aids

`MiniPhase` enum (`Investigate`, `WindDown`, `Terminal`) names phases for documentation and audit metadata. Transition rules live in the loop modules; the enum labels states rather than encapsulating the machine.

---

## §7 — Tenacious resilience tier

### Problem it solves

Operators need a single switch (`--tenacious` / `--no-tenacious`) to expand outer gate-loop and ACP spawn retry budgets without tuning `--max-loops` and `--max-acp-retries` independently. Default CLI parsing keeps tenacious expansion on; `--no-tenacious` opts into conservative budgets.

### Where it lives

- `src/reliability_tier/mod.rs` — `ReliabilityTier`, `ReliabilityTierFlags`, `ReliabilityTier::resolve`
- `src/cli/loop_opts.rs` — `apply_gate_loop_tenacious`, `apply_tenacious`, `TenaciousBudgetGuard`
- `src/cli/entrypoint_tenacious_tests.rs` — integration tests for tier resolution at parse time

When `ReliabilityTier::Tenacious` resolves and the operator has not set budgets explicitly, `apply_tenacious` expands `--max-loops` to `TENACIOUS_MAX_LOOPS` (9999) and `--max-acp-retries` to `TENACIOUS_MAX_ACP_RETRIES` (9999).

### Why there is no single type

Tier resolution is a small enum evaluated once at CLI parse time. Budget expansion mutates parsed flag values in place; there is no runtime coordinator that tracks remaining tenacious allowance across the session.

### Related typing aids

`ReliabilityTier` (`Tenacious`, `Conservative`) and `ReliabilityTierFlags` (`tenacious`, `no_tenacious`). `default_max_loops` and `default_max_acp_retries` document per-tier baseline budgets before explicit CLI overrides.
