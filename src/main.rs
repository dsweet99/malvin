//! CLI entry: `malvin code`, `malvin kpop`, `malvin do`, `malvin init`, `malvin models`.
// Match `lib.rs`: allow duplicate transitive versions under `clippy::cargo`.
#![allow(clippy::multiple_crate_versions)]

mod cli;

fn main() -> cli::Exit {
    cli::entrypoint()
}
