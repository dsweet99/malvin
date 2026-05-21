# CLI flags and Malvin review workflow

TRIGGER: no-markdown, NO_MARKDOWN, shared_opts, CLI help
ADVICE: `--no-markdown` help text is `NO_MARKDOWN_HELPTEXT` in `src/cli/shared_opts.rs` (clap `help = …` on `SharedOpts.no_markdown`). Behavior: `acp_stdout_markdown_enabled()` → `!no_markdown`. Parse tests: `src/cli/markdown_flag_parse_tests.rs`; unix `--help` regression: `tests/cli_parity.rs`.
CONFIDENCE: 3

TRIGGER: cli_parity help, --help test, global flag help
ADVICE: In `tests/cli_parity.rs`, `help_option_count` only checks a flag appears once on `malvin --help`. Also assert the help line body (e.g. find the `--no-markdown` line and check it contains the expected phrase like `Disable styled markdown`).
CONFIDENCE: 3

TRIGGER: KPop, review_prep, scope filter, LGTM review.md
ADVICE: After a broad `review_prep.md`, re-scope to items directly tied to `_malvin/<run>/plan.md` or that block a quality gate from the plan change. In-scope bugs get failing regression tests; if nothing remains, write exactly `LGTM` to `_malvin/<run>/review.md` (no extra text).
CONFIDENCE: 3

TRIGGER: KPop plan check, read-only review, falsify reading
ADVICE: Plan-review KPop is read-only: falsify by reading code (files, `git diff`), not by running tests or editing source. Acceptable plan → `review.md` = `LGTM` only.
CONFIDENCE: 3

TRIGGER: kiss stringify, kiss coverage cheat, inv_test_coverage
ADVICE: On code review, grep `fn kiss_stringify` under `src/`—about 68 test functions name symbols via `stringify!` without executing code (`src/cli/kiss_stringify_cov.rs`, `src/malvin_kiss_coverage.rs`, many `.inc` test fragments). Full inventory: `cheats.md` or `.malvin_memory/linter_cheats.md`. Treat as coverage inflation, not behavioral tests.
CONFIDENCE: 3

TRIGGER: kiss stats, lines per file, metric ceiling
ADVICE: Run `kiss stats`; when max/p99 hug thresholds (e.g. 250 lines/file, 23 functions/file, 20 calls/function), expect `include!("*.inc")` or new sibling modules (e.g. `gate_log.rs`, `stderr_log.rs`). Grep `include!(` before growing a file further.
CONFIDENCE: 3

TRIGGER: review_prep_regression, cgroup test cheat, silent return
ADVICE: `src/review_prep_regression.rs` string-guards against cgroup tests that silently `return` when cgroups are unavailable. New cgroup integration tests should use `require_cgroup_integration_test` or `#[cfg(target_os = "linux")]` modules, not early return.
CONFIDENCE: 2

TRIGGER: include_str session_spawn, parity guard, spawn_verbose
ADVICE: Do not test spawn policy with `include_str!("…/session_spawn.inc")` and `.contains("emit_containment_unavailable_warn")`; that is a documentation parity guard (coding rules forbid it). Test `emit_containment_unavailable_warn_after_spawn` with `capture_stderr_output`, or timed `AcpSession::spawn` in `src/acp_memory_containment/tests/spawn_verbose_warn.rs`.
CONFIDENCE: 0

TRIGGER: kiss check focus, test_coverage gate, scoped kiss
ADVICE: While editing one module, run `kiss check . src/<module>` to see metric violations without the repo-wide `GATE_FAILED:test_coverage` on unrelated files below 90%. Still run full `kiss check` before calling CI green.
CONFIDENCE: 0

TRIGGER: review scope, test coverage gap, LGTM not bug
ADVICE: After `review_prep.md`, drop items that are only “missing test on Linux with cgroups” or kiss-smoke branch gaps—they are not product bugs and do not block gates if `kiss check`/`nextest` pass. Bugs get failing tests; if nothing in-scope remains, `review.md` is exactly `LGTM`.
CONFIDENCE: 0
