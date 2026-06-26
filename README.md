# malvin

## Installation

```bash
cargo install kiss-ai
cargo install malvin
```

## Notes

`malvin` allows all tool calls by default.

## Speed

`malvin` likes to run unit tests. It does its best to only run what's necessary, but these tools can help speed things up:

- [Python] [pytest-testmon](https://www.testmon.org) Runs only unit tests affected by code changes
- [Rust] [cargo-nextest](https://nexte.st) Faster than `cargo test`
- [Rust] [cargo-difftests](https://github.com/dnbln/cargo-difftests) Re-runs only tests whose executed code changed (LLVM coverage indexes)
- [Rust] [sccache](https://github.com/mozilla/sccache) Speeds up builds by caching build artifacts

### sccache (Rust builds)

Install once:

- macOS: `brew install sccache`
- Linux / other: `./admin/sccache_install.sh` (or `cargo install sccache --locked`)

This repo enables sccache via `.cargo/config.toml` (`rustc-wrapper = "sccache"`). Any `cargo build`, `cargo clippy`, or `cargo nextest` in this tree uses it automatically.

Verify: `./admin/verify_sccache.sh`

### cargo-difftests (selective Rust tests)

Requires nightly Rust, `llvm-tools-preview`, and `cargo-binutils` (`cargo-cov`). One-time install:

```bash
./admin/difftests_install.sh
./admin/difftests_verify.sh
```

Initial profiling pass (slow; builds/tests under `profile.difftests`):

```bash
./admin/difftests_collect.sh
```

After code changes, rerun only dirty tests and refresh indexes:

```bash
./admin/difftests_rerun_dirty.sh
```

Malvin gate runs use `./admin/malvin_rust_test_gate.sh` (listed in `.malvin/checks`): selective difftests when indexes are warm, full partitioned nextest otherwise. Override with `MALVIN_FORCE_FULL_RUST_TESTS=1` to always run the full suite.

Indexes live in `difftests-index-root/`; work artifacts under `target/tmp/difftests/`. Both are gitignored. Normal `cargo build`, `cargo clippy`, and `cargo nextest` stay uninstrumented; only the difftests scripts use `cargo +nightly difftests`.

## Test isolation

Unit and integration tests must not create, overwrite, or delete the real `~/.malvin_home/config.toml` on the developer machine. Wrap in-process tests with `with_isolated_home` (see `src/test_utils.rs`) or integration harness helpers (`tests/common/workspace.rs`: `with_isolated_home`, `activate_test_home`). Those helpers redirect `$HOME` to a temp directory and set `MALVIN_TEST_ALLOW_HOME_CONFIG_MUTATION=1`, which production code checks in test builds before any home-config disk mutation.
