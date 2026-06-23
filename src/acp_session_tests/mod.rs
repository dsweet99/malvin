#[path = "session_inner.rs"]
pub mod session_inner;

#[path = "smoke.rs"]
mod smoke;
#[path = "cancel.rs"]
mod cancel;

#[cfg(unix)]
#[path = "unix_helpers.rs"]
mod unix_helpers;
#[cfg(unix)]
#[path = "unix_shutdown.rs"]
mod unix_shutdown;

#[cfg(all(test, unix))]
#[path = "kiss_unix_shutdown.rs"]
mod kiss_unix_shutdown;

#[cfg(test)]
#[path = "kiss_cov_external.rs"]
mod kiss_cov_external;
