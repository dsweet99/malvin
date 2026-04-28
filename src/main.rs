//! CLI entry: `malvin init`, `malvin do`, `malvin code`, `malvin kpop`, `malvin sync`, `malvin models`, `malvin ground`.
// Match `lib.rs`: allow duplicate transitive versions under `clippy::cargo`.
#![allow(clippy::multiple_crate_versions)]

mod cli;

fn main() -> cli::Exit {
    cli::entrypoint()
}
