# malvin

## Installation

```bash
cargo install kiss-ai
cargo install malvin
```

## Notes

`malvin` allows all tool calls by default.

## Speed

`malvin` like to run unit tests. It does its best to only run what's necessary, but these tools can help speed things up:

- [Python] [pytest-testmon](https://www.testmon.org) Runs only unit tests affected by code changes
- [Rust] [cargo-nextest](https://nexte.st) Faster than `cargo test`
- [Rust] [sccache](https://github.com/mozilla/sccache) Speeds up builds by caching build artifacts

### sccache (Rust builds)

Install once (macOS): `brew install sccache`

This repo enables sccache via `.cargo/config.toml` (`rustc-wrapper = "sccache"`). Any `cargo build`, `cargo clippy`, or `cargo nextest` in this tree uses it automatically.

Verify: `./admin/verify_sccache.sh`
