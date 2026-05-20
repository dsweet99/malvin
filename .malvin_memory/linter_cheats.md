# Linter cheat patterns (kiss, clippy, ruff)

TRIGGER: cheats.md, linter cheat, cheat inventory
ADVICE: Audit with a scoped scan (exclude `target/`, `_malvin/`, `.git/`): `#[allow(`, `#![allow(`, `#[cfg_attr(..., allow(`, `fn kiss_stringify`, `.kissignore`, and `# noqa` / `type: ignore`. Summarize in `cheats.md` with counts; read multi-line `#![allow(` blocks in `src/lib.rs` and `src/acp/mod.rs` manually—they are under-counted by single-line grep.
CONFIDENCE: 2

TRIGGER: ruff, noqa, python files, ruff check
ADVICE: The malvin application repo has no tracked `.py` files; in-tree ruff cheats (`# noqa`, `type: ignore`) will not appear. Pre-commit still runs `ruff check .`; template-only config is under `default_repo/hooks/ruff.yaml` (hook entry, no `per-file-ignores`).
CONFIDENCE: 2

TRIGGER: kiss ignore, kiss pragma, kiss inline
ADVICE: Kiss has no inline source pragmas in this repo. Bypasses are `.kissignore` path exclusions (`target/` at repo root and in `default_repo/kissignore`), `fn kiss_stringify_*` tests using `stringify!` only, and structural `include!` splits (see `src/acp/mod.rs` module docs).
CONFIDENCE: 2

TRIGGER: lib.rs, crate allow, clippy allow block
ADVICE: The largest clippy suppression is `#![allow(...)]` at `src/lib.rs:12-34` (22 lints, e.g. `missing_errors_doc`, `must_use_candidate`, `implicit_hasher`). Tests add `#![cfg_attr(test, allow(...))]` at `src/lib.rs:2-10`. Check here before adding more crate-level allows.
CONFIDENCE: 2

TRIGGER: unsafe_code deny, allow unsafe_code
ADVICE: `Cargo.toml` sets `[lints.rust] unsafe_code = "deny"`, but many scoped overrides exist: `#![allow(unsafe_code)]` on modules (`acp_memory_containment`, test mods), `#[allow(unsafe_code)]` on items, and `#[cfg_attr(test, allow(unsafe_code))]` in `src/acp/mod.rs`. Grep `allow(unsafe_code` when auditing “cheats.”
CONFIDENCE: 2

TRIGGER: missing_errors_doc, session_dotfile_backup allow
ADVICE: `src/session_dotfile_backup.rs` repeats `#[allow(clippy::missing_errors_doc)]` on many public fns—high-density clippy cheat cluster; prefer fixing docs or a single module-level allow over sprinkling per-fn attributes.
CONFIDENCE: 1

TRIGGER: rg slow, ripgrep timeout, search whole repo
ADVICE: Unscoped `rg` from repo root can run 90s+ (large `_malvin/` traces). Scope to `src/` and `tests/`, or use a short Python `Path.rglob` loop skipping `target`, `_malvin`, `.git`. Redirect huge output to a temp file before reading.
CONFIDENCE: 2
