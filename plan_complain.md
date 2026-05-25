# CLI: invent formatter, hunt WHERE, do pipe behavior, malvin complain

## User request (restatement)

Four related changes to malvin’s agent-backed subcommands:

1. **`malvin invent`** — Use the same logging/stdout formatter as `code`, `kpop`, `plan`, `hunt`, `tidy` (styled tool summaries, deferred-log ordering, markdown when enabled), not plain raw ACP text.

2. **`malvin hunt [WHERE]`** — Optional positional `WHERE` (free-text hint for discovery). When present, render `hunt_request.md` with `{{ hunt_where }}` set to `The user specified where to look: {WHERE}` (no extra quotes in the expanded text beyond what the sentence needs). When absent, expand `{{ hunt_where }}` to empty string. Values that match an existing bug id (`M` + five `a-z`/`0-9`) remain **fix-by-id**, not `WHERE`.

3. **`malvin do`** — Use the logging formatter on an **interactive** stdout (TTY); when stdout is **piped or redirected**, keep **raw** agent text without chrome (today’s piped behavior).

4. **`malvin complain COMPLAINT`** (new) — `COMPLAINT` is literal text or an existing `.md` file (same classification rules as `malvin plan`’s positional). Malvin turns the complaint into a **failing regression test**, then runs the **`tidy`** workflow.

5. **`.malvin/config.toml` `max_loops`** (unrelated) — Add a workspace config key that sets the default for `--max-loops` on every subcommand that supports it. If the key is **omitted from config**, the default remains **3** (today’s `malvin tidy` value). If present, that value becomes the clap default unless the user passes `--max-loops` on the CLI.

6. **Heartbeat message variety** (unrelated) — When emitting a heartbeat log line, do not always use the literal word `heartbeat`. Instead, pick **one line at random** from `default_prompts/heartbeats.txt` and use that phrase as the message payload (still formatted as `{ts} [malvin…] {phrase}`).

**RR / DCC:** This file only; no implementation in this step.

---

## Research: current codebase

### “Logging formatter” (what to match)

`build_agent` in `src/cli/code_flow_a.rs` sets `AgentStdoutTeeFlags { emit_stdout_markdown, raw_output: false, show_thoughts_on_stdout: true }`. That path uses deferred-log sinks, tool-summary lines, and optional termimad markdown on agent text — not `raw_output: true`.

| Subcommand | Client wiring | Source |
|------------|---------------|--------|
| `code`, `kpop`, `hunt`, `plan`, `tidy` (agent) | `build_agent(..., shared.acp_stdout_markdown_enabled())` | `code_flow_a.rs`, `kpop_flow_a.rs`, `bug_flow.rs`, `plan_flow_root.rs`, `tidy_flow/run_startup.rs` |
| **`invent`** | `raw_output: true`, `emit_stdout_markdown: false` | `ideas_flow.rs` `new_ideas_client` |
| **`do`** | **TTY:** `build_agent` + `show_thoughts_on_stdout` from `--thoughts`; **non-TTY:** `raw_output: true` | `do_flow.rs` + `output::stdout_is_interactive()` |
| `do` docs / `shared_opts` comment | Still say do **always** plain | `default_prompts/docs/malvin.md`, `do.md`, `shared_opts.rs` line 1 |

**Claim:** `do` pipe vs TTY behavior is **already implemented** in `do_flow.rs`; docs and help text are **stale**.

**Claim:** `invent` still needs a one-line client swap to `build_agent` (and doc updates).

### Hunt `{{ hunt_where }}`

- Template already has `{{ hunt_where }}` on line 6: `default_prompts/hunt_request.md`.
- `bug_kpop_request` returns `prompt_text("hunt_request.md")` **without** `store.render` — unresolved `{{ hunt_where }}` would reach `enforce_no_unresolved_braces` if that path rendered through the normal store pipeline; today it is passed as static text into KPOP (`bug_flow_remediation.rs`, `bug_flow.rs`).
- `BugArgs` has optional `bug_id` only (`args_bug_kpop.rs`); no `WHERE` positional yet.

**Disambiguation:** `is_valid_malvin_short_id` / `validate_malvin_short_id` in `src/malvin_short_id.rs` — reuse for “id vs hint” on a single optional positional.

### Plan-style `COMPLAINT` input

- `resolve_user_md_request` + `is_existing_md_file_path` (`src/artifacts/md_request.rs`): whitespace-free `.md` path that exists on disk → read file; else literal.
- `plan_resolve.rs` adds in-place review, `--plan_path`, normalization — complain likely needs a **subset**: required positional, write body to run `plan.md`, session `work_dir` from file parent or `.` for literal/`@file` via `resolve_user_request` if `@` paths are desired.

**Open design:** Whether complain accepts `@path` like `do`/`kpop` or only plan-style `.md` + literal (see Q2).

### Regression test + tidy

- Hunt post-KPOP: `bug_regression_test.md` then `bug_fix.md` (`session_flow.rs` `run_bug_remediation_until_pre_summary`).
- Complain: regression test **only**, then **tidy** (not `bug_fix`).
- `prepare_tidy_run` always uses `create_run_artifacts_from_text("tidy", Some(Path::new(".")))` — tidy gates and agent session use **cwd**, not an arbitrary complaint `work_dir` (`tidy_flow/run_startup.rs` line 86–87).

**Claim:** Complain must add something like `prepare_tidy_run_in_work_dir(work_dir, …)` (or pass existing `RunArtifacts` after regression) so tidy runs where the complaint session lived.

### New subcommand wiring checklist

| Area | Files / notes |
|------|----------------|
| Clap | `Commands::Complain`, `ComplainArgs` in `args.rs` |
| Entry | `entrypoint.rs` dispatch + `require_kiss_for_cli_command` |
| Flow | New `complain_flow.rs` (or `cli/complain_flow/`) |
| Prompt | `default_prompts/complain_regression_test.md` (adapt `bug_regression_test.md`; reference `{{ plan_path }}` not `{{ exp_log }}`) |
| Embed | `src/prompts/defaults.rs` `DEFAULT_PROMPTS` + `default_file` |
| Docs | `default_prompts/docs/complain.md`, `malvin.md` command table, `command_docs.rs` |
| Tests | Unit: `hunt_where` render; id vs WHERE parse; complain resolve; integration mock optional |

### Kiss

`require_kiss_for_cli_command` gates `code`, `tidy`, `plan`, `hunt` — add `complain`.

### `.malvin/config.toml` and `--max-loops` defaults (unrelated)

**Config today:** `default_repo/config.toml` (seeded by `malvin init` as `.malvin/config.toml`) has only `[logs]` (`max_age_days`, `max_runs`, `max_bytes`). Parsing lives in `src/log_gc_config.rs` via `load_logs_gc_config` / `malvin_config_path`.

**Commands with `--max-loops` (or alias):**

| Command | Flag field | Clap default today | Source |
|---------|------------|-------------------|--------|
| `malvin code` | `--max-loops` | **5** | `CodeArgs` in `args.rs` |
| `malvin tidy` | `--max-loops` | **3** | `TidyArgs` in `tidy_flow.rs` |
| `malvin kpop` | `--max-hypotheses` (alias `--max-loops`) | **10** | `KpopArgs` in `args_bug_kpop.rs` |
| `malvin hunt` | same as kpop | **10** | `BugArgs` |

**Claim:** Unifying “when config omits `max_loops`, default **3**” changes **code** (5→3) and **kpop/hunt** (10→3) unless we only apply config when the key is present and otherwise keep per-command clap literals (see Q6).

**Implementation sketch:**

- Extend config TOML (top-level or `[malvin]` section — pick one style and document).
- `load_max_loops_default(work_dir) -> usize` alongside logs GC (or shared config loader).
- At CLI parse time: clap `default_value` cannot read disk; use `default_value_os` / `default_values_os` from a function, or apply override after `try_parse_from` by detecting “flag not passed” (clap `ArgAction::Set` + `default_value_if` pattern, or post-parse merge).
- Seed `max_loops = 3` in `default_repo/config.toml` only if we want the file to document the default explicitly; user asked default **3 when not specified** — optional commented example in template is enough.
- Update `malvin init` template, `default_prompts/docs/init.md` / `malvin.md`, tests for parse + per-command wiring.

**Note:** `malvin complain` (if Q1-A flattens `TidyArgs`) would inherit the same default source.

### Heartbeat log text (unrelated)

**Source file:** `default_prompts/heartbeats.txt` — **31** non-empty lines (one phrase per line; e.g. `Still alive, still alive.`, `Run, malvin, run!`). Not embedded in Rust today.

**Emit sites (both hardcode `"heartbeat"` today):**

| Function | Location |
|----------|----------|
| `heartbeat_log_line_if_due` | `stdout_heartbeat.rs` line 38 — `format_line_with_timestamp(..., "heartbeat")` |
| `emit_heartbeat_line` | same file line 55 — same literal |

**Call graph:** `try_emit_heartbeat_if_due` → `heartbeat_log_line_if_due` → `write_heartbeat_log_line`; wall-clock poller (`HEARTBEAT_INTERVAL` 60s) and defer-sink path (`heartbeat_log_line_for_defer_sink`, `log_with_heartbeat` / `DeferredPayload::Heartbeat`) all ultimately use those helpers.

**Tests / docs that assume literal `heartbeat`:**

- `src/output/stdout_heartbeat_tests.rs` — `assert_eq!(payload, "heartbeat")`, several `contains("heartbeat")` checks, hardcoded sample lines.
- `.malvin/advice.md` — `rg` recipe anchored on `] heartbeat$` for stdout.log verification.

**Implementation sketch:**

- Parse phrases once at startup (or `include_str!` + split lines, trim, drop empties) — mirror other `default_prompts/` embeds in `src/prompts/defaults.rs` if phrases should ship in the binary.
- `random_heartbeat_phrase() -> &'static str` (or owned `String`) using `rand` or `std` thread_rng; **Hypothesis:** project may already depend on `rand` elsewhere — check `Cargo.toml` before adding.
- Replace both `"heartbeat"` call sites with the chosen phrase per emit (new random draw each time a line is due).
- Update tests: assert line matches `[malvin…]` + one of known phrases, or parse payload after `] `; update advice `rg` pattern to match any phrase or a looser anchor.

---

## Implementation plan (ordered)

### A. `malvin invent` → `build_agent`

- Replace `new_ideas_client` / `agent_io_options` with `build_agent(shared, workflow, shared.acp_stdout_markdown_enabled())`.
- Update `default_prompts/docs/invent.md`, `malvin.md` (remove “invent always plain”).
- Adjust tests if any assert raw invent stdout (`ideas_flow_tests`, PTY parity if present).

### B. `malvin hunt [WHERE]`

- Single optional positional `BUG_ID_OR_WHERE` (or keep field + accessors): if `is_valid_malvin_short_id` → fix-by-id; else non-empty → `WHERE` hint.
- `bug_kpop_request(store, hint)` → `store.render("hunt_request.md", ctx)` with `hunt_where` key.
- Update `default_prompts/docs/hunt.md`, `bug_flow_tests.rs`.

### C. `malvin do` (finish)

- **Code:** already branches on `stdout_is_interactive()` in `do_flow.rs`.
- **Remaining:** update `shared_opts.rs` comment, `malvin.md`, `do.md`; fix PTY tests in `tests/cli_parity_linux_pty_b.rs` if they still expect raw `**bold**` on TTY without `--no-markdown`.

### D. `malvin complain`

1. `malvin complain COMPLAINT` — required positional; resolve content like plan (`resolve_user_md_request` + write `plan.md` in run dir).
2. One coder prompt: `complain_regression_test.md` (or reuse `bug_regression_test.md` with different intro — prefer dedicated prompt).
3. `prepare_tidy_run_in_work_dir` + existing `run_tidy` agent path on same `work_dir`.
4. Full wiring + docs + kiss gate + tests.

### E. Config `max_loops` → default `--max-loops`

1. Add `max_loops` to `.malvin/config.toml` schema + `default_repo/config.toml` template.
2. Loader + constant `DEFAULT_MAX_LOOPS = 3` when key/section missing.
3. Wire into `code`, `tidy`, `kpop`, `hunt` (and `complain` if applicable) so CLI default comes from config, explicit `--max-loops` still wins.
4. Docs + unit tests (parse, fallback 3, override when set).

### F. Random heartbeat phrases

1. Load phrase list from `default_prompts/heartbeats.txt` (embed at compile time or read once — prefer `include_str!` + split for consistency with other defaults).
2. Central helper used by `heartbeat_log_line_if_due` and `emit_heartbeat_line`.
3. Update `stdout_heartbeat_tests.rs` and `.malvin/advice.md` heartbeat verification notes.
4. Optional: document phrase file in `default_prompts/docs/` or `malvin.md` if user-editable copies matter post-`init`.

---

## Resolved from research

| Item | Finding |
|------|---------|
| `hunt_request.md` placeholder | Already present; needs render, not file edit |
| `do` piped vs TTY | Implemented; docs/tests lag |
| Tidy work dir | Must extend; cwd-only today |
| `--max-loops` clap defaults | Differ by command (5 / 3 / 10); user wants config-driven default, **3** when config silent |
| Heartbeat text | Always literal `heartbeat` in `stdout_heartbeat.rs`; phrases file exists but unused |

---

## Open questions

**Q1 — Complain: expose tidy flags?**

- **A)** `#[command(flatten)]` `TidyArgs` on `ComplainArgs` (`--max-loops`, `--no-learn`, `--quick`).
- **B)** Tidy defaults only; no extra flags on complain.
- **C)** Only `--no-learn` and `--quick`; hide `--max-loops`.

**Q2 — Complain: input forms**

- **A)** Plan-style only: literal text or existing `.md` path (`resolve_user_md_request`).
- **B)** Also `@file` like `do`/`kpop` (`resolve_user_request`).
- **C)** Literal + `.md` only; reject `@file`.

**Q3 — `malvin hunt --fix "WHERE"`**

- **A)** Allow: `WHERE` applies to discovery KPOP request only.
- **B)** Forbid: clap `conflicts_with` between `--fix` and positional hint (hint only on plain `malvin hunt`).

**Q4 — Complain: one run dir or two?**

- **A)** Single `./.malvin/logs/<id>/` for regression + tidy (reuse `RunArtifacts` after phase 1).
- **B)** Separate run dirs for regression vs tidy (like separate `malvin tidy` invocation).

**Q5 — Invent: respect `--no-markdown`?**

- **A)** Yes — same as `code` (`acp_stdout_markdown_enabled()`).
- **B)** Always styled tool lines but never termimad on agent prose.

**Q6 — Config absent: unify all commands to 3?**

- **A)** Yes — missing `max_loops` in config ⇒ **3** for `code`, `tidy`, `kpop`, `hunt` (replaces today’s 5 and 10).
- **B)** Missing config ⇒ keep today’s per-command clap defaults; only when config **sets** `max_loops` apply it everywhere.
- **C)** Missing config ⇒ 3 for `tidy`/`complain` only; leave `code`=5 and `kpop`/`hunt`=10 until a later change.

**Q7 — Config key placement**

- **A)** Top-level `max_loops = 3` in `.malvin/config.toml`.
- **B)** `[malvin] max_loops = 3` section.
- **C)** `[defaults] max_loops = 3` section.

**Q8 — Phrase source at runtime**

- **A)** `include_str!("../../default_prompts/heartbeats.txt")` in binary (same text as repo default; workspace copy not read unless we add override later).
- **B)** Read `.malvin/heartbeats.txt` if present, else embed fallback.
- **C)** Always read `default_prompts/heartbeats.txt` from workspace on disk (breaks if file missing).

**Q9 — Randomness in tests**

- **A)** Inject RNG seed / stub phrase picker for deterministic unit tests.
- **B)** Assert payload ∈ known phrase set (31 lines) without fixing which phrase.
- **C)** `#[cfg(test)]` always return first line.

Recommendation: **Q1-A**, **Q2-A**, **Q3-A**, **Q4-A**, **Q5-A**, **Q6-A**, **Q7-A**, **Q8-A**, **Q9-A**.
