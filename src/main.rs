//! CLI entry: `malvin init`, `malvin do`, `malvin code`, `malvin kpop`, `malvin bug`, `malvin tidy`, `malvin plan`, `malvin models`.
// Match `lib.rs`: allow duplicate transitive versions under `clippy::cargo`.
#![allow(clippy::multiple_crate_versions, clippy::redundant_pub_crate)]

fn main() -> malvin::cli::Exit {
    malvin::cli::entrypoint()
}
