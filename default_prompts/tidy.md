Get the codebase to pass all of the quality gates:

- `pre-commit run --all-files`

Also run any applicable checks not already covered by an equivalent pre-commit hook:

- Rust: `cargo clippy --all-targets --all-features -- -D warnings -W clippy::pedantic -W clippy::nursery -W clippy::cargo -A clippy::must_use_candidate -A clippy::missing_errors_doc -A clippy::missing_panics_doc`
- Rust: `cargo test`
- Rust & Python: `kiss check`
- Python: `ruff check`
- Python: `pytest -sv tests`


Do not introduce any new discrepancies from `{{ grounding_path }}`. (Also, don't fix existing discrepancies from `{{ grounding_path }}`.)