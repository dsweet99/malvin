# Implicit concepts in malvin

Useful architectural ideas that recur across the codebase but remain procedural—no first-class type names the concept. Each entry explains why it matters and points to where the behavior lives.

Named counterparts live elsewhere: dual-contract observability (`ObservabilityChannel`, `AuditEventKind` in `src/observability/`) and KPop architecture (`KPopEngine`, `KPopProgram`, … in `src/kpop_engine/` and related modules).


---

## 3. ACP trace impersonation

When `--mini` is active, the in-process bash loop writes **synthetic ACP-shaped JSON-RPC lines** into `trace.jsonl` so downstream gate loops, audit tooling, and parity tests can treat mini and Cursor ACP traces interchangeably.

The shim is procedural (`acp_trace_shim`, `MiniTraceSink`) rather than a named adapter trait like `BackendTraceFormat`.

**Named counterpart:** `SyntheticAcpSessionUpdate` in `src/acp_trace_impersonation/`.

**Where:** `src/agent_backend/mini/acp_trace_shim.rs`, `src/agent_backend/mini/trace.rs`, `tests/observability_parity.rs`

---

## 4. Prompt stratification

Agent prompts are assembled in **layers** with different injection timing:

- embedded defaults and optional custom roots (`PromptStore`)
- workflow headers and constraint files prepended once per session
- user request text copied into `plan.md`
- gate-loop iteration prompts and KPop blocks injected at outer-loop boundaries
- placeholder substitution via `{{ key }}` context maps (`current_state`, `exp_log`, `quality_gates`, …)

There is no prompt graph or AST; assembly is spread across `prompts/`, workflow `run_loop.rs` files, and `client.rs`.

**Named counterpart:** `PromptStratum`, `WorkflowRenderContext`, and `join_strata` in `src/prompt_stratification/`.

**Where:** `src/prompt_stratification/`, `src/prompts/`, `src/workflow_context.rs`, `src/cli/do_flow_prompt.rs`, `src/kpop_turn_prompts.rs`, `src/kpop_engine/kpop_session.rs`

---

## 5. Sandbox as session-scoped spawn policy

All malvin-started subprocesses are expected to go through **`malvin_std_command`** (process-group isolation, `MALLOC_ARENA_MAX=2`) while an active coder session holds the sandbox slot. Mini bash fences inherit the session cwd; RSS is monitored against the workspace memory limit.

There is no `SandboxHandle` passed through the loop—policy is ambient global state plus convention.

**Named counterpart:** `SandboxSpawnPolicyAspect` in `src/session_sandbox_policy/`.

**Where:** `src/malvin_sandbox.rs`, `src/process_group_rss.rs`, `src/mem_limit_config.rs`, `src/agent_backend/mini/bash_adapter.rs`, `src/current_state.rs`

---

## 6. Investigate / WindDown / Terminal phase machine

Inside each `run_coder_prompt`, the mini loop moves through **Investigate** (multi-turn HTTP + bash fences), **WindDown** (one grace completion after limits or premature fenceless replies), and **Terminal** (record outcome and stop). The phase enum is named in `coder_prompt_phase/` and re-exported from `terminal.rs`; transition logic stays procedural in the inner-loop modules.

The same three-phase *shape* echoes informally at outer gate loops (try → wind down → exit) without a shared abstraction.

**Named counterpart:** `MiniPhase` in `src/coder_prompt_phase/` (re-exported from `terminal.rs`).

**Where:** `src/agent_backend/mini/loop_inner_phases.rs`, `src/agent_backend/mini/loop_inner_classify.rs`, `src/agent_backend/mini/terminal.rs`

---

## 7. Tenacious resilience tier

By default, malvin expands gate-loop and retry budgets aggressively (“tenacious” mode). **`--no-tenacious`** opts into conservative limits. This is a cross-cutting **reliability tier** affecting outer loop counts, not a configuration object—behavior is toggled through `SharedOpts` and interpreted independently in each workflow’s `effective_*_max_loops` helper.

**Named counterpart:** `ReliabilityTier` in `src/reliability_tier/`.

**Where:** `src/cli/shared_opts.rs`, `src/cli/loop_opts.rs`, workflow-specific `run_loop.rs` files
