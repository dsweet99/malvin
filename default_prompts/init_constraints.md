- Discover how this repo runs quality gates today and write the result to `.malvin/checks` (one shell command per non-empty line, in run order).
- Prefer repo-native invocations over malvin builtins when evidence exists in the tree (e.g. `uv run pytest`, `make test`, `pnpm test`, `cargo test -p crate`, scoped pytest paths).
- Deduplicate: one canonical line per tool (e.g. one `ruff check .`, not three pre-commit ruff hooks).
- Always include `kiss check` unless the repo explicitly excludes kiss.
- Malvin builtin fallbacks when no stronger signal exists:
  - `kiss check`
  - `ruff check .` (Python)
  - `pytest -sv tests` (Python with test modules)
  - `cargo clippy --all-targets --  --no-deps -D warnings` (Rust)
  - `cargo nextest run` or `cargo test` (Rust; prefer nextest when available)
- Signal priority (highest first):
  1. `.pre-commit-config.yaml` hook `entry` lines
  2. `Makefile` / `justfile` test and lint targets
  3. `pyproject.toml` / `package.json` scripts
  4. `.github/workflows/*.{yml,yaml}` CI steps
- Success means a faithful command list, not passing every gate on first run. Append `## KPOP_SOLVED` when `.malvin/checks` is correct.
- Note sources and trade-offs briefly in this experiment log.
