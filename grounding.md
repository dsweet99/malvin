# Malvin — grounding

## Purpose

Malvin drives a structured **implementation and review** workflow for software work: it coordinates plan-driven development and review using Cursor’s **Agent Client Protocol** (`agent acp`).

## Workflows

- **`malvin code`**
- header
- coding_rules
- implement
- review_1; kpop review.md; break if LGTM; concerns; up to max_loops times
- review_2; kpop review.md; break if LGTM; concerns; up to max_loops times
- learn

- **`malvin kpop`**
- header
- kpop
- learn

- **`malvin do`**
- prompt

## Constraints on future changes

- **Scope**: Change only what the task requires. Match existing naming, layout, and documentation tone so new code reads consistent with the rest of the project.
- **Reasoning**: Treat uncertain conclusions as hypotheses; reserve firm claims for statements you can back with evidence (code, tests, logs, or metrics).
- **Debugging**: Reproduce failures as observed, capture them with tests when appropriate, then fix—avoid speculative changes without observation.
- **Quality bar**: Treat passing the project’s automated checks (lint, tests, and project-specific validators) as part of completing a change.
- **Safety and toolchain**: Do not introduce `unsafe` Rust in **non-test** code. Test-only code may use `unsafe` when the standard library requires it (for example environment-variable fixtures), kept localized and gated with explicit `#[allow(unsafe_code)]` on the smallest enclosing test module. Stay within the crate’s declared Rust edition and minimum supported version unless the project explicitly moves them.
- **Tee** (`--no-tee` to disable): When tee is on, the primary plan/request document, the recorded invocation line (`Command: …`) printed at startup, and ACP session log content are echoed to stdout. Outbound (`>`) lines for the `learn.md` prompt are **not** echoed to stdout (trace files on disk still record the full prompt text). Trace files on disk still begin with the same `Command: …` prelude when applicable; stdout tee of a trace skips repeating that prelude so the invocation line is not shown twice. With tee off, those streams are not printed to stdout; run-directory files (for example `command.log` and trace logs) are still written for inspection.
- **Prefixed log lines** (`YYYYMMDD.HHMMSS.mmm:[who]: …`): The bracketed `who` label is padded or truncated to **10** Unicode scalars (`LOG_TAG_INNER_WIDTH` in `src/output/mod.rs`). When **stdout** is a terminal, Malvin ANSI-colors the timestamp and `[who]:` prefix on stdout unless `--no-color` is passed or the `NO_COLOR` environment variable is set. **On-disk** logs and traces (`command.log`, ACP trace files, and any write that uses the plain line formatter) use the same logical layout **without** ANSI escape sequences.
- **ACP trace labels**: Directional ACP tags (`[>…]` / `[<…]`) identify the prompt provenance for that coder turn. Default **`malvin do`** sends only the user request (no bundled `header.md`), with trace stems `[>raw]` / `[<raw]`. With **`malvin do --cooked`**, `header.md` is prepended to the request; the trace may split into optional injected repo style, header, and user segments (`>style` / `>header` / `>prompt`) while the run-timing bucket remains Implement. The first coder `session/prompt` after `begin_coder_session` may prepend optional **`coder_style.md`** text when present (unless skipped for default raw `malvin do`). When tee streams these lines to a color-capable stdout, the timestamp-prefixed line uses **green** for the outbound (`>`) `[who]:` bracket and **magenta** for the inbound (`<`) bracket (payload text is not colored); on-disk trace files stay plain.

## Repo style file

Optional repo-local **`coder_style.md`** at the workflow working tree root (see `DEFAULT_REPO_STYLE_PROMPT_REL` in `src/acp/client_impl.inc`). When not skipped, its trimmed non-empty contents may be prepended on the first coder turn after `begin_coder_session` and on review prompts.

## Outgoing prompts

Malvin prints a stdout bracket line (`[label...]`) and the **full** outgoing prompt body for each `session/prompt` (see `print_outgoing_prompt_log`). When **tee** is enabled, ACP trace mirroring also writes timestamp-prefixed `>` lines to stdout (with the `learn.md` exception noted under **Tee**). For **`malvin do --cooked`**, tee shows split outgoing segments first—optional `>style`, a single collapsed `>header` line, then per-line `>prompt`—before that `[do...]` bracket line and full payload, matching `trace_write_outgoing_prompt_do`.

- **Run timing** (`malvin code`, `malvin kpop`, and `malvin do`): After the workflow body Malvin writes `run_timing.json` under the run directory and prints one **stdout** summary line (timestamp-prefixed `YYYYMMDD.HHMMSS.mmm`, prefix `TIMING:`) with compact `name = value` pairs for wall clock, cumulative LLM wait (`session/prompt`), cumulative agent retry/backoff (sleeps between bounded retries—not model time), and every phase bucket in the JSON. Displayed durations use seconds rounded to one fractional digit. Phase buckets in the JSON match the orchestrator: Implement; Review-1 and Review-2 split into **review** vs **kpop** per attempt; Concerns; Learn when enabled. For `malvin do`, only the Implement bucket is used; default raw `malvin do` still records time in that bucket with display name `raw`, and the stdout summary uses `raw = ...` for that bucket. Other traces, `tracing`, and ACP logs keep their own formats; this line is only the run-timing summary.

## Git and tooling

**ACP-driven** Malvin workflows (`malvin code`, `malvin kpop`, `malvin do`) should not mutate git state; use git read-only when inspecting history or worktrees. **`malvin init`** is different: it may invoke **`git`** (including Git LFS) to set up a new repo—see `src/cli/init_cmd.rs`.
