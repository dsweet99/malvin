use std::num::NonZeroU32;
use std::time::{Duration, Instant};

pub const MAX_CONSECUTIVE_SAME_BACKEND_ERRORS: u32 = 3;

const fn default_consecutive() -> NonZeroU32 {
    match NonZeroU32::new(MAX_CONSECUTIVE_SAME_BACKEND_ERRORS) {
        Some(n) => n,
        None => NonZeroU32::MIN,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ConsecutiveErrorLimit(NonZeroU32);

impl ConsecutiveErrorLimit {
    #[must_use]
    pub const fn new(n: NonZeroU32) -> Self {
        Self(n)
    }

    #[must_use]
    pub const fn get(self) -> u32 {
        self.0.get()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AcpRetryCount(NonZeroU32);

impl AcpRetryCount {
    #[must_use]
    pub const fn at_least_one(raw: u32) -> Self {
        match NonZeroU32::new(raw) {
            Some(n) => Self(n),
            None => Self(NonZeroU32::MIN),
        }
    }

    #[must_use]
    pub const fn get(self) -> u32 {
        self.0.get()
    }

    #[must_use]
    pub const fn as_consecutive_limit(self) -> ConsecutiveErrorLimit {
        ConsecutiveErrorLimit::new(self.0)
    }
}

pub const LOCAL_MAX_BACKEND_ERRORS: u32 = 10;

pub const LOCAL_MAX_BACKEND_ERROR_WINDOW: Duration = Duration::from_mins(5);

pub const SAME_BACKEND_ERROR_STREAK_RESET: Duration = Duration::from_mins(1);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BackendErrorTracker {
    consecutive_count: u32,
    last_error: Option<String>,
    last_error_at: Option<Instant>,
    max_consecutive: ConsecutiveErrorLimit,
    errors_since_success: u32,
    first_error_at: Option<Instant>,
}

impl Default for BackendErrorTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl BackendErrorTracker {
    #[must_use]
    pub const fn new() -> Self {
        Self::with_limit(ConsecutiveErrorLimit::new(default_consecutive()))
    }

    #[must_use]
    pub const fn with_limit(max_consecutive: ConsecutiveErrorLimit) -> Self {
        Self {
            consecutive_count: 0,
            last_error: None,
            last_error_at: None,
            max_consecutive,
            errors_since_success: 0,
            first_error_at: None,
        }
    }

    pub const fn set_max_consecutive(&mut self, limit: ConsecutiveErrorLimit) {
        self.max_consecutive = limit;
    }

    #[must_use]
    pub const fn max_consecutive(&self) -> u32 {
        self.max_consecutive.get()
    }

    #[must_use]
    pub const fn consecutive_count(&self) -> u32 {
        self.consecutive_count
    }

    #[must_use]
    pub fn last_error(&self) -> Option<&str> {
        self.last_error.as_deref()
    }

    pub fn record_success(&mut self) {
        self.consecutive_count = 0;
        self.last_error = None;
        self.last_error_at = None;
        self.errors_since_success = 0;
        self.first_error_at = None;
    }

    pub fn record_error(&mut self, error: &str) -> bool {
        self.record_error_at(error, Instant::now())
    }

    pub fn record_error_at(&mut self, error: &str, now: Instant) -> bool {
        self.errors_since_success = self.errors_since_success.saturating_add(1);
        self.first_error_at.get_or_insert(now);
        let recent = self
            .last_error_at
            .is_some_and(|t| now.saturating_duration_since(t) <= SAME_BACKEND_ERROR_STREAK_RESET);
        self.last_error_at = Some(now);
        let is_same = recent
            && self
                .last_error
                .as_ref()
                .is_some_and(|prev| prev == error || prev.trim() == error.trim());
        if is_same {
            self.consecutive_count = self.consecutive_count.saturating_add(1);
        } else {
            self.consecutive_count = 1;
            self.last_error = Some(error.to_string());
        }
        self.consecutive_count >= self.max_consecutive.get()
    }

    #[must_use]
    pub const fn should_stop_and_exit(&self) -> bool {
        self.consecutive_count >= self.max_consecutive.get()
    }

    #[must_use]
    pub fn local_retry_cap_errors(&self) -> Option<u32> {
        let reached = self.errors_since_success >= LOCAL_MAX_BACKEND_ERRORS
            || self
                .first_error_at
                .is_some_and(|t| t.elapsed() >= LOCAL_MAX_BACKEND_ERROR_WINDOW);
        reached.then_some(self.errors_since_success)
    }
}

#[must_use]
pub fn format_backend_consecutive_error_message(
    backend_label: &str,
    error: &str,
    limit: u32,
) -> String {
    let times = if limit == 1 { "time" } else { "times" };
    format!(
        "{backend_label} backend error repeated {limit} {times} in a row; stopping and exiting. Last error:\n{error}"
    )
}

#[must_use]
pub fn format_local_backend_retry_cap_message(
    backend_label: &str,
    error: &str,
    errors: u32,
) -> String {
    format!(
        "{backend_label} local backend failed {errors} times without a successful turn (limit {LOCAL_MAX_BACKEND_ERRORS} errors or {} s); stopping and exiting. Last error:\n{error}",
        LOCAL_MAX_BACKEND_ERROR_WINDOW.as_secs()
    )
}
