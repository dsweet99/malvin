//! Test-only modules kept at crate root (not under [`super::cli`]) to limit coupling metrics.

#[cfg(test)]
#[path = "cli_kiss_cov_smoke_test.rs"]
pub mod cli_kiss_cov_smoke_tests;
