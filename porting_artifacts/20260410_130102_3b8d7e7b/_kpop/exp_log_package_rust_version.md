# KPOP experiment log: `package.rust-version` with `edition = "2024"`

## Problem (restated)

The crate used `edition = "2024"` without `rust-version`, so developers on compilers older than the 2024 edition minimum see edition errors instead of a clear “you need at least Rust X.Y” signal from Cargo’s MSRV field.

## Hypothesis (H1)

**H1:** Setting `rust-version = "1.85"` (the release that stabilized Rust 2024) is accepted by the current Cargo toolchain and does not break the build or tests.

## Predict / falsifying test

If **H1** is false, `cargo test` or `cargo metadata` fails after adding the field.

## Falsify (command + outcome)

```text
cargo test
cargo clippy --all-targets -- -D warnings
```

**Result (2026-04-10):** After adding `rust-version = "1.85"` to `[package]` in `Cargo.toml`, `cargo test` exited **0** (full suite). `cargo clippy --all-targets` exited **0**.

**Conclusion:** **H1 is not rejected** on this machine (rustc **1.93.0**). The MSRV line documents the edition floor for contributors.
