//! CLI entry: `malvin do`, `malvin kpop`, `malvin tidy`, `malvin models`.
#![allow(clippy::multiple_crate_versions, clippy::redundant_pub_crate)]

fn main() -> malvin::cli::Exit {
    malvin::cli::entrypoint()
}
