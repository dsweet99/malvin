//! CLI entry: `malvin code …` and `malvin kpop …`.
// Match `lib.rs`: allow duplicate transitive versions under `clippy::cargo`.
#![allow(clippy::multiple_crate_versions)]

mod cli;

fn main() -> cli::Exit {
    cli::entrypoint()
}
