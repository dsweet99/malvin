# KPop experiment log

Problem: satisfy the in-process `pi::sdk` requirements in `plan.md` (handoff: compile, leftover tests, docs, gates).

## H1 — The in-process wiring already type-checks

- Hypothesis: files from the prior turn compile under rustc 1.96 with crates.io `pi_agent_rust` 0.1.23.
- Predict: `CARGO_INCREMENTAL=1 cargo check --jobs 1 --lib` exits 0.
- Falsify: run that command.

## H1 — Installed malvin 0.2.3 already greets with codex:gpt-5.6
- Hypothesis: the published binary already runs `malvin --model=codex:gpt-5.6 --do Hello` successfully.
- Predict: command exits 0 and stdout contains a greeting.
- Falsify: run installed malvin with a short timeout.
- Result: FALSIFIED as a remaining problem for the installed binary. `malvin --model=codex:gpt-5.6 --do Hello` exited 0 and printed "Hello, David. What would you like me to do?"

## H2 — Working-tree malvin still greets; listing already exposes every catalog id
- Hypothesis: `cargo run` of this tree can list every Codex catalog model and run --do Hello on gpt-5.6.
- Predict: `cargo run --quiet -- models codex:` prints the same catalog; `cargo run -- --model=codex:gpt-5.6 --do Hello` greets.
- Falsify: build+run those two commands.

## H1 — The in-process wiring already type-checks

- Hypothesis: files from the prior turn compile under rustc 1.96 with crates.io `pi_agent_rust` 0.1.23.
- Predict: `CARGO_INCREMENTAL=1 cargo check --jobs 1 --lib` exits 0.
- Falsify: run that command.

## H1 — Installed malvin 0.2.3 already greets with codex:gpt-5.6
- Hypothesis: the published binary already runs `malvin --model=codex:gpt-5.6 --do Hello` successfully.
- Predict: command exits 0 and stdout contains a greeting.
- Falsify: run installed malvin with a short timeout.
- Result: FALSIFIED as a remaining problem for the installed binary. `malvin --model=codex:gpt-5.6 --do Hello` exited 0 and printed "Hello, David. What would you like me to do?"

## H2 — Working-tree malvin still greets; listing already exposes every catalog id
- Hypothesis: `cargo run` of this tree can list every Codex catalog model and run --do Hello on gpt-5.6.
- Predict: `cargo run --quiet -- models codex:` prints the same catalog; `cargo run -- --model=codex:gpt-5.6 --do Hello` greets.
- Falsify: build+run those two commands.

## H1 — leftover wiring type-checks
- Hypothesis: files from the prior turn compile under rustc 1.96 with crates.io `pi_agent_rust` 0.1.23.
- Predict: `CARGO_INCREMENTAL=1 cargo check --jobs 1 --lib` exits 0.
- Falsify: inspect APIs, then run that command after fixing known mismatches.
- Result: **rejected as “already clean.”** `Tool` is a native-async trait (no `async_trait`); leftover tests still call RPC/`MALVIN_PI` listing. Fix those first, then compile.

## H2 — single-job cargo check after leftover fixes
- Hypothesis: leftover wiring now type-checks after Tool/async-trait and test updates.
- Predict: CARGO_INCREMENTAL=1 cargo check --jobs 1 --lib exits 0.
- Falsify: run that command.

## H3 — leftover tests and docs already match the in-process path

- Hypothesis: session_spawn_tests, kiss_coverage, models_cmd_auth_filter, and docs still mention RPC/MALVIN_PI listing as the live path.
- Predict: grep still finds those leftovers, so Phase 5 and leftover tests remain unsatisfied.
- Falsify: grep the named files.

- Result: **partially confirmed.** Docs for README/models/malvin.md already describe the crate path. session_spawn_tests and kiss_coverage were rewritten. Remaining leftovers: discover.rs still documents MALVIN_PI; discover_tests still test the binary resolver; src/python/fast_task.py still has MALVIN_PI helpers; table parsers remain unused by the live list path.

## H4 — leftover wiring type-checks after prior leftover fixes

- Hypothesis: leftover wiring now type-checks under rustc 1.96 with crates.io pi_agent_rust 0.1.23.
- Predict: CARGO_INCREMENTAL=1 cargo check --jobs 1 --lib exits 0.
- Falsify: run that command.
kiss baseline measured (16 files <90%, incl. tests/-scope false positive); no code changed; handoff: .malvin/incomplete_handoff_pi_sdk_kiss_baseline.md
kiss witness rule found: path refs / bare tokens count, stringify!(name) does not (map_agent_event_end at 0% despite stringify entries); baseline re-confirmed 13 files; handoff: .malvin/incomplete_handoff_pi_sdk_kiss_names.md
KPop kiss rules: bare static tokens count + transitive cascade (39->24 violations); stringify!/qualified refs do NOT count, qualified refs break duplicate-name attribution; probe file + handoff: .malvin/incomplete_handoff_pi_sdk_kiss_names.md
## H1 — mock-test panics: drain double-polls the prompt oneshot

- Hypothesis: `pi_sdk::client_mock_tests` "called after complete" comes from `drain_agent_events`: `tokio::select!` polls `done = &mut reply` even when the event branch wins, and a winning `poll_recv` consumes the receiver (`inner = None`); the later `reply.await` in the AgentEnd/None arms then panics. Fake mode pre-sends the reply, so the losing-poll consumption happens on the first select.
- Predict: making the select `biased` with the reply branch first (so a ready reply is always cached into `prompt_result` and never polled again), and having the AgentEnd/None arms reuse the cached result instead of blindly re-awaiting, makes `cargo test --lib pi_sdk::client_mock_tests` pass with no new kiss/clippy findings.
- Falsify: edit `src/pi_sdk/session.rs`, rerun the three tests, then the full rust gate.

g1 session3: kiss coverage gate CLEARED (tokio::test needs qualified token); 3 structural violations left (2 file splits + witness05 split); spawn/providers/isolated_bash refactors compile; handoff updated
g1 retry2: live smoke PASSED with pi binary unreachable (models pi: + openrouter --do Hello; trace run_done status=finished) => acceptance 1-3 done; Phase 5 cleanup scoped not started (no code edits); envelope .malvin/incomplete_handoff_pi_sdk_phase5_cleanup.md
g1 retry3 (KPop audit): rust gate FAILS on models auth-filter test (llamacpp row missing; passed 02:04, tree unchanged). Cache-drift hypothesis falsified. H3: local-provider synthesis gating env-dependent; H4: 02:04 pass anomalous. Envelope .malvin/incomplete_handoff_llamacpp_gate_fail.md
g1 retry4 (KPop audit): H3 FALSIFIED / H4 CONFIRMED with crate source evidence — llamacpp IS in provider_metadata (keyless local, filter passes it) but built_in_models/load_for_listing NEVER synthesizes rows for it (ad_hoc_model_entry is per-request only; legacy catalog 0 hits; snapshot 'llama' = LlamaAPI cloud). pi:llamacpp/ listing rows are impossible under crate 0.1.23 => auth-filter test expectation unsatisfiable; fix = synthesize keyless-local rows in list_pi_models_sync OR drop that assertion. Handoff: .malvin/incomplete_handoff_llamacpp_gate_fail.md
g1 retry5 (KPop satisfy): auth-filter test FIXED (zhipuai unmapped-provider + llamacpp auth-layer assertions; 02:04 mystery solved — original test drove fake-pi table containing 'llamacpp local'). ALL 5 GATES PASS: ruff/kiss/clippy/pytest-157/rust-859. Remaining: Phase5 deletion sweep (MALVIN_PI resolver, dead PiRpc wire, mock_pi.sh, kiss witnesses, docs verify). Handoff: .malvin/incomplete_handoff_pi_sdk_phase5_cleanup.md
g1 session2 (KPop scope-verify): Phase 5 sweep VERIFIED not EXECUTED (budget). H1 confirmed deletions-only; CORRECTION: map_event_summary.rs is live (map_agent_event dep) — old envelope trap; H3: python scope = ft_resolve_pi_bin + PI_BIN_REMOTE + 4 negative assertions; H4 HAZARD: discover_tests.rs holds 2 LIVE tests (timeout clamp, crate registry listing) — port before delete. Zero code edits. Envelope: .malvin/incomplete_handoff_pi_sdk_phase5_sweep_verified.md
g1 final-audit (KPop): ALL 5 GATES RE-RUN GREEN this turn (ruff/kiss/clippy -D warnings/pytest 157/rust gate 843); plan audit: Phases 0–5 + acceptance 2,4,7,8,9 verified in-tree, 1&3 evidenced by g1-retry2 live smoke; residual: models_cmd "cached … `pi --help`" note wording, no explicit thread-count leak assertion. Envelope .malvin/incomplete_handoff_pi_sdk_done_audit.md
g1 nit-polish (KPop): nit1 FIXED — models_cmd.rs:114 note now reads "built in-process; rows shown only for authenticated providers" (verified truthful against crate load_with_mode: fresh build each call); grep finds zero other live `pi --help` strings (only exp_log/handoff mentions). nit2 scoped NOT started: plan = thread-delta assertion around begin/end_coder_session via PiRuntime named thread "malvin-pi-sdk"; mock env MALVIN_TEST_NO_REAL_AGENT + test_env_lock pattern from client_mock_tests is the entry point. Gates NOT re-run after the string edit (budget cut to handoff). Envelope .malvin/incomplete_handoff_pi_sdk_nit_polish.md
