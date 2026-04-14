
Use parallelized subagents (at most 4).

Work until the end without asking for user input. If you are uncertain about an implementation
detail, use your best judgement. There will always be an opportunity to revise later on.

Some projects are Rust. Some are Python. Some are both. You are not required to have both languages. Consult grounding.md, .pre-commit-config.yaml, and the existing codebase to determine which language(s) apply to this project. Do NOT add a language that is not already present in the project.

Be sure that all applicable checks pass. Only run checks for languages that are actually used in this project:
- If Cargo.toml exists: cargo clippy --all-targets --all-features -- \
    -D warnings \
    -W clippy::pedantic \
    -W clippy::nursery \
    -W clippy::cargo \
    -A clippy::must_use_candidate \
    -A clippy::missing_errors_doc \
    -A clippy::missing_panics_doc
- If .py files exist: ruff check
- Always: kiss check

and all applicable unit tests pass:
- If Cargo.toml exists: cargo test
- If .py files exist: pytest -sv tests

Run checks & tests frequently to avoid a big cleanup at the end.

Do not write "documentation parity guards". Do not write comments. NEVER EDIT grounding.md.

Write code to fail fast. Assert liberally. DRY.

## Nota Bene
ALL checks and tests should pass on ALL	files (not just the ones you modified). Don't tell me
 about "pre-existing" problems. We're here to work. To fix. Be tenacious. There's no excuse
 for not getting ALL checks and tests to pass on ALL files.
Don't touch .kissconfig ever.
Don't add `# noqa` except to ensure correct functioning of the code.
Don't cheat the tests. Make earnest attempts to pass the linters and unit tests in the spirit
 in which they were designed.
Your task is to get ALL checks and tests to pass on ALL files.
Do NOT create Rust code in a Python-only project, or Python code in a Rust-only project.
