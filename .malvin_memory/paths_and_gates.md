# Paths and prompt context

TRIGGER: format_prompt_path, review_path, missing file
ADVICE: `format_prompt_path` in `src/orchestrator/helpers.rs` uses `resolve_path_against_base`: if `path.canonicalize()` fails (file not created yet), canonicalize `parent` and join `file_name` so `strip_prefix(work_dir)` still yields `./_malvin/...` paths for prompts.
CONFIDENCE: 3

TRIGGER: macOS, /var, canonicalize, strip_prefix
ADVICE: On macOS, `work_dir` may canonicalize to `/private/var/...` while a non-canonicalized absolute path stays under `/var/...`, breaking `strip_prefix`. Always resolve both sides consistently (canonicalize file or parent+name).
CONFIDENCE: 3

TRIGGER: kiss violation, nested closure, format_prompt_path
ADVICE: If `format_prompt_path` triggers kiss `nested_function_depth` or `calls_per_function`, extract path resolution into a named helper (e.g. `resolve_path_against_base`) instead of nested `unwrap_or_else` closures.
CONFIDENCE: 3

TRIGGER: linux pty, cli_parity
ADVICE: Linux PTY parity tests live in `tests/cli_parity_linux_pty_a.rs` and `tests/cli_parity_linux_pty_b.rs`, with helpers in `tests/common/`.
CONFIDENCE: 2
