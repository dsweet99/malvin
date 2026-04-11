//! CLI entry: `malvin code …` and `malvin kpop …`.

mod cli;

fn main() -> cli::Exit {
    cli::entrypoint()
}
