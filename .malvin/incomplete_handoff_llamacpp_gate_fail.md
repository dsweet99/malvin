# Incomplete handoff: pi::sdk in-process — llamacpp filter-test root cause FOUND

Status: **no code edits**. Gates this turn: ruff ✓ kiss ✓ clippy ✓ pytest 157 ✓
rust gate ✗ (1 failed / 200+ passed, fail-fast cancelled 658). Do NOT commit
(standing instruction). Work dir `/home/dsweet/Projects/malvin`.

## Root cause of `run_models_filters_pi_rows_using_live_provider_auth_map` failure (SETTLED)

H3 (env-gated synthesis) FALSIFIED; H4 CONFIRMED: the test asserts something the
crate registry can never produce. The 02:04 PASS remains unexplained but cannot
recur under current code + crate 0.1.23.

Evidence chain (all read from `~/.cargo/registry/src/index.crates.io-*/pi_agent_rust-0.1.23/`):

1. `llamacpp` IS registered in `provider_metadata.rs` (~line 1413): canonical_id
   `"llamacpp"`, `auth_env_keys: &[]`, routing base_url `127.0.0.1:8080/v1`,
   `auth_header: false`. So malvin's `provider_has_access`
   (`src/pi_sdk/auth.rs`) passes it via the `keys.is_empty()` branch — the
   FILTER is not the bug.
2. BUT `ModelRegistry::load_for_listing` → `built_in_models()`
   (`src/models.rs:1249`) builds rows ONLY from:
   - `legacy_generated_models()` = parsed `models.generated.ts` (grep count of
     "llamacpp": **0**);
   - `append_upstream_nonlegacy_models()` over
     `docs/provider-upstream-model-ids-snapshot.json` keys (contains `llama` =
     LlamaAPI cloud provider with auth keys, a DIFFERENT provider; no
     `llamacpp` key).
3. The #104 "local providers synthesize READY keyless entries" logic lives in
   `ad_hoc_model_entry(provider, model_id)` (`models.rs:2438`) and runs only on
   the per-request resolve path (`--provider llamacpp --model X`), never during
   listing. No env var gates it; it is simply not called by
   `load_for_listing`.
4. Therefore `list_pi_models_sync()` (`src/pi_sdk/models_list.rs`) can never
   emit `pi:llamacpp/<id>` rows and the assertion at
   `src/cli/models_cmd_auth_filter_tests.rs:30` is unsatisfiable by
   construction. Test isolation itself is sound: crate honors
   `PI_CODING_AGENT_DIR` (`config.rs:1030`).

## Fix direction (choose ONE, prefer 1)

1. Make malvin synthesize keyless-local-provider rows itself in
   `list_pi_models_sync()` after building from the registry: for each provider
   where malvin's own map has no env keys AND
   `pi::provider_metadata::provider_is_keyless_local(provider)` is true (it is
   `pub`; check re-export path via `pi::` root), append a placeholder row per
   plan acceptance #3 spirit ("lists models … hides providers with no access";
   keyless local providers have access). Suggested row shape matching existing
   grammar: one row per known local provider, e.g. id
   `llamacpp/<model-id>` requires a model id — either reuse routing_defaults
   example id (none exists) or list the provider with a generic id like
   `<local-model>`; NOTE plan acceptance #3 wording must stay satisfied
   (openai hidden without key, openrouter shown with key — keep
   `assert_live_auth_filter` semantics intact).
2. Alternative: drop the llamacpp assertion from
   `run_models_filters_pi_rows_using_live_provider_auth_map` and instead unit-
   test `is_provider_authenticated("llamacpp") == true` directly (already
   covered indirectly by `auth.rs` tests). Weaker: loses end-to-end filter
   proof for keyless providers.
Do NOT weaken the openai/openrouter assertions.

## Everything else this turn

- All other requirements verified satisfied: rustc 1.96 pinned
  (`Cargo.toml` `rust-version = "1.96"` + `rust-toolchain.toml` channel
  `1.96.0`); `pi_agent_rust 0.1.23` crates.io dep, default-features off,
  `sqlite-sessions` on; asupersync defaults off; embedded session/runtime/
  isolated-bash/map_agent_event files present; docs (README,
  `default_prompts/docs/{malvin,models}.md`, `ops/fast_task.py`) already on the
  crate path; `MALVIN_PI` discover leftovers gated `#[cfg(test)]`.
- Phase-3-style mem watch wired through
  `watch_process_group_memory_with_optional_pgid(pgid=0)` per prior handoff.
- Remaining plan items beyond the failing test (lower priority, mostly Phase 5):
  - RPC remnants kept intentionally until live `--do` smoke is re-verified
    after the fix (`mock_pi.sh`, `session_io.rs` parts, `BridgeWire::PiRpc`).
  - Optional live smoke `malvin --model=pi:<provider>/<model> --do Hello` with
    no `pi` on PATH (prior turn PASSED once: models pi: + openrouter --do
    Hello, run_done finished).

## Next-agent start

1. Apply fix 1 above (edit `src/pi_sdk/models_list.rs::list_pi_models_sync`),
   or fix 2 if you decide listing-time synthesis is out of scope for malvin.
2. `cargo test --lib cli::models_cmd_auth_filter_tests` → expect 2 passed.
3. Full gates in order: ruff check; kiss check; clippy
   `--jobs 3 --all-targets --all-features -- -D warnings -W clippy::cargo`;
   `pytest tests`; `./admin/malvin_rust_test_gate.sh`.
4. Re-run live smoke (acceptance 1–3) if any listing code changed.
Predicted: 20–40 min.
