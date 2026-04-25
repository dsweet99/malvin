# Malvin repo — tooling, layout, quality gates

## Required checks (repo root)

- `ruff check .`
- `kiss check .` (**not** bare `kiss`). See `.kissignore`.
- `pytest -sv tests` (minimal Python smoke; primary tests are Rust). If a test imports the repo as a package, run from repo root with `PYTHONPATH=.`.
- `cargo test`
- `cargo clippy --all-targets --all-features -- -D warnings` (`clippy::doc_markdown`: wrap code-like tokens in `//!`/`///` in backticks—bare identifiers fail under `-D warnings`)

Pre-commit **`cargo-clippy`** (must match `.pre-commit-config.yaml` `entry:` verbatim):

```text
cargo clippy --all-targets --all-features -- -D warnings -W clippy::pedantic -W clippy::nursery -W clippy::cargo -A clippy::must_use_candidate -A clippy::missing_errors_doc -A clippy::missing_panics_doc
```

Pre-commit also runs `ruff check .`, `kiss check .`, and `admin/check_untracked.sh` (fails if untracked `.rs`/`.py` sources exist). It does **not** run `cargo test` or `pytest`; run the full suite before merge.

If the workspace **ripgrep/search tool** errors (e.g. I/O), run **`rg` from a repo-root shell** instead. **KPOP experiment logs**, `malvin models` parser/ANSI caveats, and the same search fallback are indexed in **`.llm_style/malvin_debugging.md`**.

## Merge markers, `_malvin` plans, green tree

TRIGGER: merge conflict markers  
ADVICE: Search `<<<<<<<`, `=======`, `>>>>>>>` with repo-root **`rg`** on text globs (e.g. `--glob '*.rs' --glob '*.py' --glob '*.md' --glob '*.inc'`) and skip `target/`. Unfiltered `grep -r` over binaries can false-positive (ELF/shared objects, e.g. a local `rust_out` build artifact)—those byte matches are **not** conflicts.
CONFIDENCE: 0

TRIGGER: cited `_malvin` plan  
ADVICE: **`_malvin/**`** is in `.gitignore`; one-off specs live at paths like `_malvin/**/plan.md`. When the user cites that path, **read it by path**—workspace search may omit gitignored directories.
CONFIDENCE: 0

TRIGGER: verification-only cleanup  
ADVICE: If the task is marker cleanup or audit and markers are absent while **`ruff check .`**, **`kiss check .`**, **`cargo clippy`** (§ Required checks, verbatim `entry:`), **`cargo test`**, and **`pytest -sv tests`** all pass, **zero code changes** is a valid outcome.
CONFIDENCE: 0

TRIGGER: green tree no excuses  
ADVICE: When the user requires the full suite on **all** files, treat every failure as in-scope—fix or narrow with cited evidence; do not dismiss as "pre-existing" without the same remediation bar.
CONFIDENCE: 0

### Untracked source files (`admin/check_untracked.sh`)

`malvin init` copies **`default_repo/admin/check_untracked.sh`**: fails only when untracked `*.rs` or `*.py` exist (`git ls-files --others --exclude-standard`). The **malvin monorepo** root **`admin/check_untracked.sh`** (pre-commit `entry:`, not the init template) **additionally** fails when **`default_prompts/do_header.md`** exists on disk but is **not** in the index (`include_str!` in `src/prompts/defaults.rs`). **Agents** that must not run `git` cannot `git add`; ask the user to stage sources, or use tracked `tests/*.rs` (e.g. `cli_parity.rs`) to guard `*.rs` / `*.py` wiring. Keep ad-hoc top-level untracks (e.g. scratch CSV, local `models` captures) out of the same commit as product changes.

TRIGGER: check untracked init template  
ADVICE: Init template = untracked **`*.rs`/`*.py` only**; monorepo root script may add **`default_prompts/do_header.md`**—see § Untracked **above**; do not conflate the two when debugging consumer `malvin init` installs.
CONFIDENCE: 0

## Hard constraints

- **Never edit** `.kissconfig`.
- **Do not run git** in automated assistance for this project; users stage/commit locally.
- **Rust** `edition = "2024"`, `rust-version = "1.87"` in `Cargo.toml`.
- **`.kissignore`** may exclude paths; still run `kiss check .` on the analyzed set.

TRIGGER: clippy doc first paragraph  
ADVICE: **`clippy::too_long_first_doc_paragraph`**: keep the opening **`///`** paragraph short; put detail in following paragraphs. **`clippy::items_after_statements`**: `use` must come before other statements in function blocks (pre-commit uses `-D warnings`).
CONFIDENCE: 0

## Crate layout (high level)

| Area | Location |
|------|----------|
| Library entry | `src/lib.rs` (re-exports + deprecated `malvin::agent` shim) |
| Binary entry | `src/main.rs` → `src/cli/` |
| ACP JSON-RPC / session / agent client | `src/acp/` — **many** pieces are `include!`d (see below) |

**Binary vs library:** `src/cli/` is part of the **`malvin` binary crate**, not `malvin` the library. `pub(crate)` fields on `AgentClient` (e.g. `timing`) are visible inside `src/lib.rs` code only—**not** from `src/cli/`. Use public methods on the library client (e.g. `attach_run_timing_for_session`) or keep field access in lib modules (`src/orchestrator/`, …).
| Invocation argv | `src/invocation.rs` |
| Log path display | `src/log_paths.rs` |
| Run artifacts | `src/artifacts/` (`mod.rs`, `startup_tag.rs`, `grounding_backup.rs`) |
| Orchestrator | `src/orchestrator/`, `src/review_sync.rs`; `#[cfg(test)]` `src/orchestrator_tests.rs` |
| Run timing | `src/run_timing/mod.rs` + `src/run_timing/report.rs` — `malvin code` / `malvin kpop` / **cooked** `malvin do`: `run_timing.json` + one **stdout** `TIMING: ` line after the workflow body; default **raw** `malvin do` skips `attach_run_timing` / `emit_run_timing_after_acp` (no timing artifact/line). Details: `src/cli/do_flow.rs` and § **Run timing** / § **CLI** below; root `grounding.md` stays high-level (`malvin do` → `prompt` only) |
| Prompts | `src/prompts/` + `default_prompts/` (including `do_header.md` for raw `malvin do`, embedded in `src/prompts/defaults.rs`); `prompts/template.rs` holds merge/render helpers when `kiss` `lines_per_file` caps `mod.rs` (~250 lines) |

TRIGGER: malvin binary crate  
ADVICE: `src/cli/` is binary-only—not the `malvin` library—so `pub(crate)` on `AgentClient` fields is not visible there; use public lib methods or keep access in lib modules. Private `src/cli/*.rs` submodules: `pub fn` not `pub(crate) fn` when `clippy::redundant_pub_crate` fires. Library `///` docs must not link `[crate::cli::...]`—the CLI crate is separate; use plain text ("`malvin do`", …).
CONFIDENCE: 0

TRIGGER: lib artifacts submodule file  
ADVICE: Same rule as `cli mod sibling file` for **`src/artifacts/mod.rs`** (e.g. `grounding_backup.rs`); add **`tests/cli_parity.rs`** `include_str!` wiring checks when a new submodule is easy to omit from commits.
CONFIDENCE: 0

### ACP `include!` assembly (kiss dependency depth)

Navigate by **include file names** (not only `mod` tree): e.g. `ops_body.inc` (reviewer pair), `tee_strip_tests.inc` (test-only strip helper), `reader_inline.inc`, `agent_bundle.inc`, `transport/*.rs`, `coalesce.rs`.

**Included `.rs` files** (e.g. `transport/command.rs` pulled into `acp/mod.rs`) **inherit the parent module's `use`**—types like `Path` are not imported locally unless the include parent brings them.

## Child health + ACP silence (`src/child_health/`)

TRIGGER: child health module layout  
ADVICE: Library module at `src/child_health/mod.rs` with `linux.rs`, `macos.rs` (`libproc` + `errno`/`libc`), `other.rs`, and `tests/` (e.g. `macos_sample.rs`) when split helps `kiss` limits. Wired from `src/lib.rs` (`mod child_health`); RPC wait in `src/acp/transport/rpc.rs` (`child_pid` on `AcpStdioRpc`). `src/coverage_kiss.rs` `stringify!` for public helpers.
CONFIDENCE: 0

TRIGGER: process_absent cannot_sample  
ADVICE: **`process_absent`**: OS says PID row missing (`/proc` `NotFound`; macOS `proc_pidinfo` + `errno == ESRCH`). **`cannot_sample`**: I/O/parse failure or ambiguous read—`exists: true`, `counters_trusted: false`, zero placeholders. Do not conflate with "gone" (user-facing `acp child process is not running`).
CONFIDENCE: 0

TRIGGER: counters_trusted progress  
ADVICE: **`health_indicates_progress`** compares CPU/context/thread fields only when **both** snapshots have `counters_trusted`. If the first sample is untrusted, do **not** infer progress from a lone trusted second read (typical Linux `/proc` counters would always look "busy"). `silence_grace_for_rpc_timeout` clamps `rpc_timeout/8` to 50–250ms.
CONFIDENCE: 0

TRIGGER: rpc_wait_response health race  
ADVICE: After the silence `sleep(rpc_timeout)`, use `tokio::select!` so the JSON-RPC **`oneshot`** and **`evaluate_after_acp_silence`** (grace sleep inside) are polled together—inbound responses during grace must return success, not `AppearsHung`. Regression: `transport_tests::rpc_response_arriving_during_child_health_grace_is_delivered`.
CONFIDENCE: 0

TRIGGER: voluntary_ctxt switches parse  
ADVICE: In `child_health/linux.rs` **`parse_status_voluntary_ctxt`**, after `strip_prefix("voluntary_ctxt_switches:")`, use **`rest.trim().parse::<u64>()`**—`trim_start()` alone leaves a trailing **`\r`** on the value token and **`u64` parse returns `Err`**, so voluntary context switches are dropped and progress detection weakens. Regression: `child_health::tests::linux_parse::voluntary_ctxt_parses_when_value_has_trailing_cr`.
CONFIDENCE: 0

## Run timing (`malvin code` / `malvin kpop` / `malvin do`)

TRIGGER: TIMING line JSON parity  
ADVICE: One `serde_json::Value` from `to_json_value` is written pretty to `run_timing.json` and passed to `format_timing_stdout_line_from_json` for stdout—keeps the line aligned with disk. The stdout text after the timestamp prefix uses [`RUN_TIMING_SUMMARY_PREFIX`] in `src/run_timing/mod.rs` — literal **`"TIMING: "`** (colon + **one ASCII space** before the first field); do not describe it as bare `` `TIMING:` `` in docs. `PHASE_MS_KEYS_JSON_ORDER` in `report.rs` must match `phases_ms` keys in `to_json_value`.
CONFIDENCE: 0

TRIGGER: CLI emit run timing after ACP  
ADVICE: `src/cli/timing_merge.rs` — `emit_run_timing_after_acp(client, run_dir, &timing, acp_result)` wraps `finalize_and_emit_run_timing` + `set_run_timing(None)` + `merge_acp_and_timing_results`; used by `do_flow` and `kpop_flow` (not async-generic—avoids `&mut AgentClient` + `Future` lifetime issues).
CONFIDENCE: 0

- **Code:** `src/run_timing/` (`mod.rs`, `report.rs`); orchestrator sets `AgentClient::timing` and finalizes after `run_with_coder_session`; KPOP and **cooked** `malvin do` attach timing and finalize via `emit_run_timing_after_acp` (`do_flow::run_do_with_timing` returns early for **raw** `do` and never calls it). ACP `client_impl.inc` / `ops_body.inc` record `session/prompt` duration and bounded-retry sleeps.
- **Artifacts:** `run_timing.json` in the run directory; one timestamp-prefixed **stdout** line beginning with **`TIMING: `** (see `RUN_TIMING_SUMMARY_PREFIX`, `src/run_timing/report.rs`) when timing runs—**not** for default raw `malvin do`.
- **Dual failure:** If timing I/O and workflow/ACP both fail, return the **primary** error first (`prefer_primary_errors_over_timing` in `src/orchestrator/mod.rs`; `merge_acp_and_timing_results` in `timing_merge.rs`).
- **Rustdoc:** Helpers that merge `Result`s after timing I/O must not read as reordering stdout vs the orchestrator/CLI contract—ordering is established in the orchestrator / KPOP / `do` callers.

### Error merge (`src/cli/timing_merge.rs`)

TRIGGER: merge_acp_and_timing_results  
ADVICE: After ACP body, **`emit_run_timing_after_acp`** passes **`merge_acp_and_timing_results(acp_result, timing_result)`**—ACP `Err` wins; timing `Err` only if ACP was `Ok`.
CONFIDENCE: 0

TRIGGER: prefer_primary_string_errors  
ADVICE: **`prefer_primary_string_errors(primary, restore)`**—used after **`malvin code`** / **`malvin kpop`** when restoring workspace **`grounding.md`** from `~/.malvin/groundings/...`; primary workflow/ACP `Err(String)` wins over restore failure (same "prefer primary" idea as timing merge).
CONFIDENCE: 0

## Repo style file (optional)

TRIGGER: DEFAULT_REPO_STYLE_PROMPT_REL  
ADVICE: Public const **`DEFAULT_REPO_STYLE_PROMPT_REL`** (`"coder_style.md"`) in **`src/acp/client_impl.inc`**; **`AgentClient::new`** sets **`style_prompt_path`** from it. **`read_coder_repo_style_text`** / **`prepend_coder_repo_style_to_prompt`** share trim/empty rules with coder and reviewer paths. New repos may get copy via **`default_repo/grounding.md`** from **`malvin init`**; root **`grounding.md`** in this repository does not define a `## Repo style file` heading—the injection behavior is authoritative in **`client_impl`** and related helpers.
CONFIDENCE: 0

## Docs parity (rustdoc ↔ `grounding.md`)

TRIGGER: rustdoc section cite  
ADVICE: Repository **`grounding.md`** uses Markdown **`##` / `###` headings**—in **`///`** comments refer to **`## Heading name`** (bold or backticks), not typographic "§ Section" labels, so readers can search the file.
CONFIDENCE: 0

## ACP traces, coalescing, tee

- **Trace format:** After `AcpSession::prompt`, trace may start with plaintext `Command: …\n` (from `invocation`), then JSON lines from agent stdout—not guaranteed pure JSONL when that prelude exists.
- **Tee:** Live trace tee goes through the stdout reader (`trace_file_write_line` / coalescing). **No** post-prompt stub—historical `maybe_tee_log` was removed. Post-hoc strip contract: `strip_trace_invocation_line_for_tee` lives in **`tee_strip_tests.inc`** (test-only `include!`). No-newline `Command:`-only buffers strip to empty (documented + tested).
- **Coalescing:** Verbose/trace paths track **Unicode scalar counts** per buffer in `coalesce.rs` to avoid repeated full-buffer `chars().count()` in flush loops.

TRIGGER: ACP trace labels  
ADVICE: Directional ACP tags label the **outer prompt template filename stem** (`implement.md` → `implement`, `review_1.md` → `review_1`). For **`malvin do`**, `run_coder_prompt`’s `who` / tee stem is **`header`** when **`--cooked`** (payload from `header.md`) and **`raw`** for default **raw** mode (payload from `do_header.md` + user; `DO_RAW_ACP_TRACE_STEM` in `do_flow.rs`). **Both** paths may use **`do_trace_split`** so the trace can show `>header` / `>prompt` (and optional `>style`). Run timing still records **Implement** when timing is attached (cooked `do`, not raw).
CONFIDENCE: 0

### ACP learn tee (outbound vs inbound)

TRIGGER: learn stem who  
ADVICE: `learn.md` coder turns use `who`/`stem` **`"learn"`**—from `prompt_md_stem` in `orchestrator/mod.rs` `run_coder_prompt`, or hard-coded **`"learn"`** in `ops_body.inc` for standalone KPOP learn. `AcpSession::prompt(..., who)` sets trace tags and tee metadata.
CONFIDENCE: 0

TRIGGER: learn outbound stdout omit  
ADVICE: **`trace_write_outgoing_prompt`** (`session_trace.rs`) calls **`acp_tee_echo_outgoing_prompt_lines(tee_stdout, stem)`**; when `stem == "learn"`, skip **`print_stdout_acp_tee_line`** (`AcpTeeDirection::ToAgent`) for each outgoing `>` line. Trace file still writes full lines via **`format_line`**.
CONFIDENCE: 0

TRIGGER: learn inbound placeholder  
ADVICE: **`prompt_stdout_replacement`** (`session.rs`) yields **`LEARNING_PLACEHOLDER`** for `who == "learn"`; **`trace_tee_stdout_line`** (`coalesce.rs`) prints it at most once to stdout while the trace file records real agent chunks—do not strip learn text from disk traces.
CONFIDENCE: 0

TRIGGER: clippy match same arms JSON wall  
ADVICE: Prefer `v.get("wall_clock_ms").and_then(Value::as_u64)` over duplicated `match` arms for wall `n/a` vs numeric—`clippy::match_same_arms` (`run_timing/report.rs`).
CONFIDENCE: 0

TRIGGER: outgoing prompt stdout  
ADVICE: **All** outgoing prompts: no body tee to stdout (`acp_tee_echo_outgoing_prompt_lines` returns `false`); only `[{stem}...]` announcement via `print_outgoing_prompt_log`. Disk trace keeps full `>{stem}` lines. For `do --cooked`: announce each segment (`[style...]`, `[header...]`, `[prompt...]`). Inbound `<learn`: one `[learning...]` placeholder (`prompt_stdout_replacement`).
CONFIDENCE: 0

## Tests

TRIGGER: docs parity llm_style  
ADVICE: `tests/cli_parity.rs` uses **`include_str!`** for selected **`src/`** files, **`Cargo.toml`**, and **`.gitignore`** fixtures (see that file)—not `grounding.md` / **`.llm_style/*.md`**. When changing invariants the tests assert (ACP spawn args, `malvin do` wiring, gitignore, …), run **`cargo test`** (or at least `cli_parity`) before merge.
CONFIDENCE: 0

TRIGGER: malvin_tooling path strings vs src  
ADVICE: After a module split/rename, update **`.llm_style/malvin_tooling.md`** crate-layout table and path ADVICEs to match **`src/lib.rs`** / real dirs (e.g. run artifacts: **`src/artifacts/`** `mod.rs` + `startup_tag.rs`). Add **`tests/cli_parity.rs`** `include_str!` wiring checks when a new submodule is easy to commit without its sibling file; existing examples include **`artifacts_grounding_backup_module_is_declared_and_source_tracked`** and **`init_template_gitignore_*`**.
CONFIDENCE: 0

- **Node:** Many ACP tests use executable Node scripts as mock `agent acp` children; `node` must be on `PATH` or handshake tests fail. Spawns that need a minimal UNIX layout use **`prepend_standard_path_for_child`** (`src/acp/transport/command.rs`) so `#!/usr/bin/env node` resolves.
- **Brittle source tests:** Prefer behavioral tests over `include_str!` substring checks on `mod.rs` that break on refactors.
- **CLI / gitignore guards:** Cross-cutting behavioral checks and `git check-ignore` fixtures often live in `tests/cli_parity.rs` (alongside ACP spawn string guards).
- **Grounding vs code:** Prefer behavioral tests; add `include_str!` of `grounding.md` only when a regression clearly needs that snapshot.

TRIGGER: source-shape regression tests  
ADVICE: After changing ACP prompt signatures or include-body call shapes, check string-based tests like `tests/review_ops_order.rs` and docs-parity tests that `include_str!` source files—they may need updates even when runtime behavior is correct.
CONFIDENCE: 0

TRIGGER: code stdout regression  
ADVICE: Guard protocol-leak fixes with behavior tests in `tests/cli_parity.rs` (for `malvin code`) and `tests/do_stdout.rs` (for `malvin do`): assert parsed agent text appears and raw `"jsonrpc"` lines do not.
CONFIDENCE: 0

TRIGGER: lib test_utils binary  
ADVICE: `malvin::test_utils` is lib `#[cfg(test)]` only—binary tests use `tests/*.rs` + `env!("CARGO_BIN_EXE_malvin")` (see `init_pre_commit.rs`).
CONFIDENCE: 0

## Review sync, `review.md`, shared output

TRIGGER: RunArtifacts review paths  
ADVICE: Use `RunArtifacts::artifact_review_md()` / `workspace_review_md()` in `src/artifacts/mod.rs` for workspace ↔ run artifact `review.md` paths; avoid duplicating `run_dir.join("review.md")` / `work_dir.join("review.md")` at call sites.
CONFIDENCE: 0

TRIGGER: sync_review_then_is_lgtm  
ADVICE: `src/review_sync.rs` — `sync_review_then_is_lgtm` returns **`io::Result<bool>`** (propagate read/write with `?`); map to `AgentError` / `WorkflowError` in `ops_body.inc` and `orchestrator/review_loop.rs`. Do not treat sync I/O failure as "not LGTM."
CONFIDENCE: 0

TRIGGER: sync_review_file clear stale LGTM  
ADVICE: **`sync_review_file`** (**`src/review_sync.rs`**) **writes an empty file** to the artifact path when the workspace `review.md` is **missing** or **whitespace-only after trim**, so a previous **`LGTM`** in the artifact cannot survive. Parent dirs are created as needed. Non-empty workspace text overwrites the artifact as before. **`is_lgtm`** still maps **`read_to_string`** failures to **`false`**. Regress in-crate tests in **`review_sync.rs`** + **`orchestrator_tests.rs`**.
CONFIDENCE: 0

TRIGGER: reviewer pair order regression  
ADVICE: `tests/cli_parity.rs` **`reviewer_pair_ops_calls_review_prompt`** `include_str!`s **`src/acp/ops_body.rs`** and asserts the reviewer `session/prompt` call shape for the pair path. Deeper order guarantees belong in behavioral tests in **`src/review_sync.rs`** (not only substring guards).
CONFIDENCE: 0

TRIGGER: shared stdout stderr output  
ADVICE: **`src/output/mod.rs`** (+ optional **`src/output/*.rs`**, e.g. **`acp_tee.rs`**) — line-oriented helpers (`format_line`, `print_stdout_line`, …). **`pub use`** re-exports preserve **`malvin::output::`** paths after splits. Align `#[must_use]` with sibling APIs if plain `cargo clippy` warns; pre-commit allows `-A clippy::must_use_candidate`.
CONFIDENCE: 0

## Prefixed log lines (`src/output/`)

TRIGGER: LOG_TAG_INNER_WIDTH bracket who  
ADVICE: `format_log_tag_inner` pads/truncates the bracket label to **`LOG_TAG_INNER_WIDTH`** Unicode scalars. Same width applies to ACP trace lines built with `format_line` / `format_acp_directional_tag_prefix` (directional `>`/`<` stem before padding).
CONFIDENCE: 0

TRIGGER: plain format_line files only  
ADVICE: On-disk logs and traces use **`format_line`** / **`format_line_with_timestamp`** only—e.g. `trace_file_write_line` (`coalesce.rs`), `trace_write_*` (`session_trace.rs`), `emit_command_line` (`cli/mod.rs`). Never write **`format_line_with_timestamp_ansi`** or escape codes to files.
CONFIDENCE: 0

TRIGGER: stdout ANSI gate  
ADVICE: `init_stdout_style(no_color)` runs after **`Cli::parse()`** and after **`require_kiss_for_cli_command`** when applicable—if `kiss` is missing for **`code`**/**`kpop`**, **`entrypoint`** exits first (stderr via **`print_stderr_line`** / plain **`format_line`**) without calling **`init_stdout_style`**. Otherwise sets color when `stdout().is_terminal()` and not `--no-color` (`GlobalOpts` in **`shared_opts.rs`**) and **`NO_COLOR`** is unset. `print_stdout_line` chooses ansi vs plain; pipes/tests stay uncolored.
CONFIDENCE: 0

TRIGGER: ACP tee direction colors  
ADVICE: Live tee only: **`print_stdout_acp_tee_line`** in **`src/output/acp_tee.rs`** — **`AcpTeeDirection::ToAgent`**: bright **green** `[who]:` prefix (prompt text to agent) from **`trace_write_outgoing_prompt`** (`session_trace.rs`); **`FromAgent`**: bright **magenta** (agent stream / learn placeholder) from **`trace_tee_stdout_line`** (`coalesce.rs`). Payload text stays unstyled. **Disk** traces still **`format_line`** only (no escapes).
CONFIDENCE: 0

TRIGGER: output kiss lines_per_file split  
ADVICE: When **`kiss`** `lines_per_file` (~250) fires on **`src/output/`**, split into **`mod.rs`** + focused sibling (e.g. **`acp_tee.rs`**, **`terminal_wrap.rs`**) instead of shrinking behavior; re-export at **`output` module** root.
CONFIDENCE: 0

### Terminal wrap (TTY)

TRIGGER: terminal_wrap module  
ADVICE: **`src/output/terminal_wrap.rs`** — **`terminal_columns()`** = **`columns_from_env()`** (valid **`COLUMNS`** **20–500**) **or** **`columns_from_tty()`** (**`terminal_size::terminal_size()`**, width **≥20** capped at **500**; **`None`** if query fails or too narrow) **or** **80**. **`stdout_line_wrap_meta`** / **`line_wrap_meta`** use that width; **`acp_tee.rs`** tees with same wrap; **`trace_line_write.rs`** uses **`terminal_columns()`** for raw stdout wrap. **`stdout_allows_log_word_wrap()`**, **`wrap_words_bounded`**. **`pub(crate) mod`** under `output/`; **`clippy::redundant_pub_crate`** → **`pub fn`** inside the private child module.
CONFIDENCE: 1

TRIGGER: terminal_size crate  
ADVICE: Dependency **`terminal_size`** (see **`Cargo.toml`**). **`terminal_size()`** returns **`Option<(Width, Height)>`**; use **`usize::from(w.0)`** for column count (**`Width`** is a **`u16`** newtype—do not cast the tuple element incorrectly).
CONFIDENCE: 0

TRIGGER: print_stdout wrap rule  
ADVICE: **`print_stdout_line`** / **`print_stderr_line`**: wrap only when **`stdout`/`stderr`** is a TTY **and** `line.chars().count() > max_payload`, where **`max_payload = terminal_columns().saturating_sub(prefix_len).max(1)`** and **`prefix_len`** = **`format_line_with_timestamp(ts, who, "").chars().count()`** (plain prefix). **Same `ts`** for all continuation lines. **Pipes** (`!is_terminal()`): one unwrapped line, original spacing preserved when no wrap path runs.
CONFIDENCE: 0

TRIGGER: acp_tee wrap  
ADVICE: **`print_stdout_acp_tee_line`** (`acp_tee.rs`): same **`max_payload`** / prefix rule as **`print_stdout_line`**; ANSI tee colors unchanged on each physical line.
CONFIDENCE: 0

TRIGGER: raw trace stdout wrap  
ADVICE: **`trace_tee_stdout_line`** (`src/acp/trace_line_write.rs`): if **`writer.raw_output`**, wrap **plain** stdout at **`terminal_columns()`** without malvin prefix. **`trace_file_write_line`** still **`write_all(format_line(...))`** to disk **unwrapped**—on-disk format stable.
CONFIDENCE: 0

TRIGGER: coalesce not TTY wrap  
ADVICE: **`ACP_VERBOSE_COALESCE_MAX`** and **`coalesce_append_chunk`** (`coalesce.rs`) buffer **JSON `session/update` chunks** for trace + verbose **`tracing`**; **`coalesce_flush_cap`** splits at the last **word boundary** (space) before the 125-scalar cap via **`coalesce_word_split_points`** so downstream **`wrap_words_bounded`** never sees mid-word hard clips. The cap is **not** terminal width—TTY reflow is **`wrap_words_bounded`** only.
CONFIDENCE: 1

TRIGGER: kiss static coverage per module  
ADVICE: If **`kiss check`** reports **test_coverage** gaps for a file, add **`#[cfg(test)]`** **`stringify!`** (and minimal smoke calls) **in that same source file**—not only **`src/coverage_kiss.rs`**—so static coverage attributes to the implementation module.
CONFIDENCE: 0

TRIGGER: kiss GlobalOpts shared_opts not args only  
ADVICE: New root-level flattened flags (e.g. **`GlobalOpts`**) belong in **`shared_opts.rs`** when **`kiss`** `concrete_types_per_file` would break if added to **`args.rs`** alone; **`Cli`** flattens them from there.
CONFIDENCE: 0

TRIGGER: redundant_pub_crate  
ADVICE: `clippy::redundant_pub_crate`: in a non-`pub` submodule, prefer `pub struct` over `pub(crate) struct` when the type is only re-exported at the parent (e.g. `acp/session_types.rs` → `acp/mod.rs`).
CONFIDENCE: 0

### Repo-wide string contracts (renames, banned fragments)

When removing or renaming a user-facing term, **`rg` the whole repository** (implementation, `grounding.md`, `default_prompts/`, `.cursorrules`, `.llm_style/`, `_kpop/` logs). A short **forbidden substring** may appear inside unrelated English words—verify with context, not only exact tokens. In **learn/review prompts**, distinguish **agent pacing** (latency, thoroughness) from product **metrics** wording when that distinction matters for user-visible copy.

## kiss

TRIGGER: kiss limits split modules  
ADVICE: Enforces lines-per-file, call counts, duplication, etc. Use `src/coverage_kiss.rs` and `kiss_refs` / `stringify!` so symbols stay visible. Split modules when limits hit (e.g. extract `report.rs`, **`src/output/acp_tee.rs`**, thin orchestrator `run()` helpers when `calls_per_function` fires). Run **`kiss check .`** during multi-step work—not only at the end.
CONFIDENCE: 0

## Breaking API notes

- Document consumer-visible removals (e.g. old `malvin::agent` paths) in **`CHANGELOG.md`**.

## CLI (`src/cli/`)

TRIGGER: clap Commands enum order  
ADVICE: `clap` prints **`Commands:`** in **`#[derive(Subcommand)]`** variant order (`src/cli/args.rs`). To change `malvin --help` list order (e.g. `init`, `do`, `code`, …), reorder the enum—not `mod.rs` match arms (those can stay any order).
CONFIDENCE: 0

TRIGGER: CLI help and shared opts  
ADVICE: `src/cli/args.rs`, `mod.rs`, `shared_opts.rs`; `disable_help_subcommand = true`; `SharedOpts::tee_startup_stdout`.
CONFIDENCE: 0

TRIGGER: clap help manual default text  
ADVICE: For extra prose on **`#[arg]`** lines (beyond clap's auto **`[default: …]`**), use **`[default: …]`** in **`///`** docs, not **`(default: …)`**, so usage stays consistent (`src/cli/shared_opts.rs`, `init_cmd.rs` pattern).
CONFIDENCE: 0

TRIGGER: cli mod sibling file  
ADVICE: Each `mod name;` in `src/cli/mod.rs` requires `src/cli/name.rs` (e.g. `do_flow`, `timing_merge`). Add the `.rs` in the **same change** as the `mod` line so checkouts compile; agents do not run `git`—users stage the pair.
CONFIDENCE: 0

TRIGGER: CLI kiss gate  
ADVICE: **`malvin code`** / **`malvin kpop`** require a **`kiss`** executable on **`PATH`** (`lookup_bin_on_path` in **`src/env_path.rs`**). **`require_kiss_for_malvin`** returns an install hint: **`cargo install kiss-ai`**. **`require_kiss_for_cli_command`** in **`src/cli/mod.rs`** runs **immediately after** **`Cli::parse()`** and **before** **`init_stdout_style`** / Tokio so missing-`kiss` exits fail fast; stderr does not need stdout ANSI setup. **`malvin init`** also calls **`require_kiss_for_malvin("init")`** before **`kiss init`**. Binary regression: **`tests/kiss_code_kpop_path.rs`** (minimal isolated **`PATH`**, **`env!("CARGO_BIN_EXE_malvin")`**—same spawn pattern as **`tests/init_pre_commit.rs`**).
CONFIDENCE: 0

- **Startup (shared):** `emit_run_startup_sequence` in `mod.rs` — echo primary artifact, `command.log` / optional `Command:`, then `Logs: …` — used by `code` / `kpop` / `do` **only when `do` is not raw** (i.e. **`--cooked`**: `run_do` skips this for default raw `do`).
- **`do`:** `do_flow.rs` — `DoArgs` is **defined** here; `args.rs` only **imports** it for `Commands::Do(DoArgs)` (kiss `concrete_types_per_file` is per file—do not assume `DoArgs` lives in `args.rs` when editing or navigating). **`--cooked`:** `prepare_do_prompt_store`, `combine_do_acp_prompt_header_and_user` build context with **`workflow_context`** (`kpop_common.md` → `kpop` key) so a customized `header.md` can use `{{ kpop }}`; **raw (default):** `combine_do_raw_header_and_user` uses **`workflow_context_paths_only`** (artifact paths only—no `kpop_common` render) for shipped `do_header.md`. Both branches pass **`header_user_for_trace`** into `run_coder_prompt` with **`do_trace_split: Some((header, user))`**. `skip_repo_style: !do_args.cooked` (no injected repo style on first turn when raw). `run_do_with_timing` (raw: no `emit_run_timing_after_acp`); binary `#[cfg(test)]` parses `Cli::try_parse_from` and exercises combine. Root **`grounding.md`** lists `malvin do` in one line; default **raw** skips startup `emit_run_startup_sequence`, no trailing **`DONE`**, and no post-ACP timing file—see **Startup (shared)** above and `run_do` in `do_flow.rs` for the full contract.

### malvin do ACP trace (split stems)

TRIGGER: malvin do split trace stems  
ADVICE: **`run_coder_prompt`** passes **`do_trace_split: Some((header, user))`** for **both** default **raw** and **`--cooked`** `do` when a combined header+user payload is built → **`AcpSession::prompt_do_trace_split`** with **`DoPromptTraceSplit`** (`src/acp/outgoing_prompt_trace.rs`). The **`header`** segment is the rendered `do_header.md` (raw) or `header.md` (cooked) body; **`user`** is the user request. Outgoing trace: **`>style`** (if injected repo style prepended), **`>header`**, **`>prompt`**. **`who`** to **`prompt`/`run_coder_prompt`** is for non-split and tee labels: **`raw`** vs **`header`**; on the split path, stems come from the split (`client_impl` docs). **`kiss`:** split types into **`outgoing_prompt_trace.rs`** when **`session_types.rs`** hits **`concrete_types_per_file`**.
CONFIDENCE: 0

TRIGGER: repo style single read  
ADVICE: **`coder_prompt_body_with_optional_repo_style`** (`client_impl.inc` top) returns **`(full_prompt, repo_style)`** with at most **one** read of the repo style file; **`repo_style.as_deref()`** feeds **`DoPromptTraceSplit.style_text`** when **`do_trace_split`** is **`Some`**—do not read the style path again for trace.
CONFIDENCE: 0

TRIGGER: coder_prompt_body session  
ADVICE: `coder_prompt_body_with_optional_repo_style` at top of `client_impl.inc` (with `read_coder_repo_style_text` / `prepend_coder_repo_style_to_prompt`): prepends injected repo style when `style_on_first_turn && !skip_repo_style &&` file nonempty (trim nonempty). `begin_coder_session` sets `coder_style_on_next_prompt`; `run_coder_prompt` passes it into compose then clears it. Default raw `malvin do` sets `skip_repo_style` (no repo-style prepend); the **coder prompt body** is still `do_header.md` + user, not the user line alone. Tests: `compose_coder_prompt_tests` in `agent_bundle.inc`; CLI `tests/cli_parity.rs`: `malvin_do_default_skips_repo_style_prepend_contract`, `malvin_do_raw_uses_do_header_cooked_uses_header_contract`.
CONFIDENCE: 0

- **Timing merge:** `timing_merge.rs` — `merge_acp_and_timing_results` shared with `kpop_flow.rs` (avoid duplicated merge helpers; kiss `duplication`).
- **`src/cli/args.rs`, `mod.rs`, `shared_opts.rs`:** `tee_startup_stdout` gates startup `Command:` + plan echo vs `--no-tee`.
- **Default model:** `DEFAULT_CLI_MODEL` in `shared_opts.rs`; `malvin models` footer must use the same constant (see `tests/cli_parity.rs`).

## `malvin init`, `plan.md`, env

- **Implementation:** `src/cli/init_cmd.rs` — templates from `default_repo/`, `admin/check_untracked.sh`, then tooling bootstrap (order documented in `plan.md`).
- **Tests:** `tests/init_pre_commit.rs` spawns the real binary with a minimal `PATH` to assert fail-fast when `pre-commit` is missing (avoids relying on `malvin::test_utils` from the binary crate).
- **Agent on PATH:** `src/env_path.rs` — `lookup_bin_on_path`, `agent_or_cursor_agent_bin()` (same `agent` → `cursor-agent` preference as `resolve_agent_bin` in `ops_body.inc`).

## ACP bounded retry (where it lives)

- **Policy:** `src/acp/retry_policy.inc` (`MAX_AGENT_ATTEMPTS`, `plan_agent_retry`, retriable / upgrade-plan strings).
- **Retriable strings:** `agent_string_is_retriable` uses ASCII-lowercased `contains` checks—add **narrow** substrings for transient transport/session teardown (e.g. writable/readable iterable closed), not broad patterns that mask logic errors.
- **`timeout` wording:** A bare `contains("timeout")` can match **config/validation** copy (`timeout_ms`, `grpc_timeout_ms`). The policy uses `timeout_word_without_identifier_false_positive` in `retry_policy.inc` (skip when `timeout_` appears) while keeping phrases like `timed out`. Regress in `agent_bundle.inc` `retry_policy_tests`.
- **Sleep/break loop:** `backoff_after_agent_failure` in `src/acp/client_impl.inc` (included via `agent_bundle.inc`).
- **Included in:** `agent_bundle.inc` pulls `retry_policy.inc`, `ops_body.inc`, `client_impl.inc`.
- **Unit tests:** `retry_policy_tests` in `src/acp/agent_bundle.inc` (policy helpers are not only tested from integration tests).
- **User-facing exhaustion messages:** `client_impl.inc` formats `failed after {retries} retries` using `retries = attempts_used.saturating_sub(1)` (post-first-failure attempts), not raw `MAX_AGENT_ATTEMPTS`.

TRIGGER: ACP upgrade plan eprintln  
ADVICE: Upgrade-plan `Err`: **single** `eprintln!` at `src/cli/mod.rs` (not duplicated in `client_impl.inc`); see `retry_policy.inc` / `client_impl.inc`.
CONFIDENCE: 0

- **Ad-hoc task specs:** `_malvin/**/plan.md` may hold one-off agent instructions—implement when the user cites that path; product/bootstrap `plan.md` remains the shipped template story (`init_cmd`).

## Reviewer workflow (conceptual)

After **review** prompt: sync workspace review to artifact, **`is_lgtm`** on artifact before **kpop** prompt—implementation in `src/acp/ops_body.inc` / orchestrator, not a single legacy `src/agent/ops.rs` file.

Root **`review.md`** is the working reviewer checklist ("problems only" / resolved). After fixing issues, update it so it does not stay stale versus `grounding.md` and the code.

## KPOP `--p-creative` / MBC2

- **Selection:** `src/kpop_acp_prompt.rs` — `kpop_acp_user_prompt`, `KpopAcpPromptPick`, `CREATIVE_MIN_INTERACTION`, `kpop_standalone_outbound_prompt_count`, `kpop_creative_enabled`.
- **Session:** `src/acp/ops_body.inc` `run_kpop_flow_once` — main KPOP `session/prompt` (interaction **0**), optional `learn` (`learn.md`, interaction **1**). No synthetic continuation prompts; MBC2 gating uses only those real workflow turns.
- **Prompts:** `default_prompts/mbc2.md`; embedding / defaults in `src/prompts/mod.rs`; CLI in `src/cli/args.rs` (`KpopArgs`).

TRIGGER: KPOP outbound count contract  
ADVICE: `kpop_standalone_outbound_prompt_count(has_learn)` returns **1** (main only) or **2** (main + learn)—**not** extra rounds for `--p-creative`; creative mode only changes text via `kpop_acp_user_prompt`. Keep `run_kpop_flow_once` aligned (e.g. `debug_assert_eq!` vs that count) so the `pub` helper stays live and `dead_code` clean.
CONFIDENCE: 0

TRIGGER: KPOP MBC2 interaction gate  
ADVICE: `skip_mbc2_for_interaction_index` (or equivalent): when `CREATIVE_MIN_INTERACTION == 0`, index-based skip is off; keep an `else` branch with `interaction_index < CREATIVE_MIN_INTERACTION` so raising the constant later does not require rewriting the gate—use one `const fn` + `#[allow(clippy::absurd_extreme_comparisons)]` on that fn only.
CONFIDENCE: 0

## Clippy tunable const + kiss (malvin)

TRIGGER: clippy absurd_extreme_comparisons  
ADVICE: Comparisons like `u32 < PUB_CONST` when the const is **0** trigger `absurd_extreme_comparisons`; centralize in a helper `const fn` with `if CONST == 0 { false } else { … }` and a **single** `allow` on that fn—see § KPOP MBC2 interaction gate.
CONFIDENCE: 0

TRIGGER: kiss attributes_per_function  
ADVICE: Threshold is **1** attribute per function—do not stack `#[inline]` + `#[allow(clippy::…)]`; drop `inline` if the allow is required for a small `const fn`.
CONFIDENCE: 0

TRIGGER: clippy useless_let_if_seq  
ADVICE: Prefer `let n = if let Some(…) = … { …; 2 } else { 1 };` over `let mut n = 1;` + conditional reassignment when sequencing main vs optional follow-up prompts (`ops_body.inc` pattern).
CONFIDENCE: 0

## Rust edition 2024 + clippy (malvin)

- **`gen` is a keyword:** do not call `rng.gen()`; use e.g. `rand::distributions::{Distribution, Uniform}` and `sample(rng)`.
- **Float guards:** prefer `!x.is_finite() || x <= 0.0` over `!(x > 0.0)` where clippy flags `neg_cmp_op_on_partial_ord` (NaN / ordering).
- **`use` placement:** avoid `use` items after other statements in a block (`clippy::items_after_statements`); lift imports to the enclosing module (e.g. `rand` in `agent_bundle.inc` for `ops_body.inc`).
- **Async + RNG:** `thread_rng()` / `ThreadRng` is not `Send`; do not hold it across `.await`. For multiple `session/prompt` rounds in one async fn, use one `rand::rngs::StdRng::from_entropy()` (or seed) and `&mut rng`.
- **kiss arity:** if `arguments_per_function` fires, group parameters in a struct (same pattern as `KpopAcpPromptPick`).

TRIGGER: Rust 2024 unsafe env tests  
ADVICE: **`std::env::set_var`** / **`remove_var`** are **`unsafe`** in Rust **2024**. Repo **`Cargo.toml`** **`[lints.rust] unsafe_code = "deny"`**—tests that mutate **`COLUMNS`** (or other env) need **`#[allow(unsafe_code)]`** on the test (or narrow scope) and **`unsafe { … }`** around those calls; **`std::sync::Mutex`** can serialize env tests. **`clippy::redundant_closure_for_method_calls`:** **`Mutex::lock`** poison recovery → **`unwrap_or_else(std::sync::PoisonError::into_inner)`**.
CONFIDENCE: 0

## LiteLLM / token cost (external proxy)

TRIGGER: LiteLLM token cost  
ADVICE: Prefer **provider `usage`** on each response when cost matters; LiteLLM **`token_counter`** is **heuristic** (tiktoken/HF, fallbacks)—treat counts as **approximate** vs Anthropic/Gemini/etc. Bullets below expand pricing/counter behavior.
CONFIDENCE: 0

- Prefer **provider `usage`** on each response when cost matters; that is authoritative when present.
- LiteLLM **`token_counter`** uses **tiktoken** / HF tokenizers + message/tool heuristics; unknown OpenAI-style models may fall back to **`cl100k_base`**—treat counts as **approximate** vs Anthropic/Gemini/etc.
- **`completion_cost`** multiplies tokens (from usage or estimate) by `model_cost_map` prices; **`litellm.disable_token_counter`** can zero counts.

## Keyword index (moved from `style.md` surface)

- **MSRV / edition:** `edition = "2024"`, `rust-version = "1.87"` in `Cargo.toml`; mention in `README.md` if documenting toolchain.
- **Orchestrator prompt stems:** `prompt_md_stem` / `strip_suffix(".md")` in `src/orchestrator/` — avoid `len()-3` slicing.
- **Prompts `include_str!`:** defaults live under `default_prompts/`; paths in `src/prompts/mod.rs`.

## `malvin code` workflow structure

TRIGGER: malvin code workflow  
ADVICE: `implement.md` → review loop (review_1/review_2 with `kpop_review.md`, not `kpop.md`) → `learn.md`. No `validate_plan.md` step.
CONFIDENCE: 0

TRIGGER: malvin code check_plan pacing  
ADVICE: **`check_plan`** can dominate **`TIMING:`** wall/LLM seconds vs **`implement`** on greenfield temp repos (e.g. **`evaluations/calc_cli_rs.sh`**). Agents often spend long spans on **`kiss`** static expectations, **`cargo fmt --check`**, and strict **`cargo clippy`** before implementation converges. Debug with stdout **`TIMING:`** + **`_malvin/<run>/`** logs; see **`malvin_evaluations.md`** § calc eval wall time.
CONFIDENCE: 0

TRIGGER: kpop.md vs kpop_review.md  
ADVICE: **`kpop.md`** is for standalone `malvin kpop` runs. **`kpop_review.md`** is used in `malvin code` review loops—validates and revises `review.md`. Both in `default_prompts/` and `src/prompts/defaults.rs`.
CONFIDENCE: 0

TRIGGER: concerns ABORT result_path  
ADVICE: `concerns.md` may write "ABORT" to `{{result_path}}` (`_malvin/<run>/result.md`). After concerns, orchestrator calls `check_abort` in `src/orchestrator/helpers.rs`—if file contains "ABORT", workflow halts with error.
CONFIDENCE: 0

TRIGGER: workflow template context  
ADVICE: `workflow_context` in `src/orchestrator/helpers.rs` provides: `plan_path`, `kpop_log_dir`, `review_path`, `result_path`. All paths point to `_malvin/<run>/` artifacts except user-provided `plan_path`.
CONFIDENCE: 0

## Repo workspace gates (`src/cli/repo_checks.rs`)

TRIGGER: repo workspace gates  
ADVICE: `run_repo_workspace_gates`: `kiss_clamp::ensure_kiss_clamp_if_needed` → `warn_kissconfig_test_coverage_if_needed` (parse `[gate].test_coverage_threshold` in `.kissconfig`; warn if missing or `< 90`; on read/parse error print warning with underlying `io`/`toml` error) → `run_pre_commit_checks_or_warn` (no `.pre-commit-config.yaml` → warn; else `pre-commit run --all-files` via `Command::output`, `format_pre_commit_failure` on non-success: exit code or `signal`, stdout/stderr, `trim_detail_chars`). Wired from **`run_code`** (`mod.rs`), **`run_kpop`** (`kpop_flow.rs`), **`run_do`** (`do_flow.rs`). Implementation: `repo_checks.rs`; kiss clamp logic: `kiss_clamp.rs`.
CONFIDENCE: 0

## Kiss structural refactors

TRIGGER: kiss structural refactors  
ADVICE: When `kiss check` fires on arity/size: **args** → one `struct` per call site pattern; **calls** in one function → extract named helper (`run_repo_workspace_gates`); **lines** in `cli/mod.rs` → new file (e.g. `exit.rs` for `Exit` + `Termination`). Binary `stringify_cov.rs` may need new `stringify!` refs.
CONFIDENCE: 0

## Diff thrash metrics

TRIGGER: diff thrash metric wording  
ADVICE: Byte- or path-summed edit costs and **gross/net ratios** depend on checkpoint cadence and diff math—do not treat "1.0" or low gross as proof the agent made no mistakes; state assumptions.
CONFIDENCE: 0
