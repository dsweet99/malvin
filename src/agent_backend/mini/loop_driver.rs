//! Inner bash-fence loop for one `run_coder_prompt`.

#[path = "loop_http.rs"]
mod loop_http;
#[path = "loop_http_retry.rs"]
mod loop_http_retry;
#[path = "loop_inner_finish.rs"]
mod loop_inner_finish;
#[path = "loop_inner_prompt.rs"]
mod loop_inner_prompt;
#[path = "loop_inner_phases.rs"]
mod loop_inner_phases;
#[path = "loop_inner_types.rs"]
mod loop_inner_types;
#[path = "loop_inner_classify.rs"]
mod loop_inner_classify;
#[path = "loop_inner_http.rs"]
mod loop_inner_http;
#[path = "loop_inner_bash.rs"]
mod loop_inner_bash;
#[path = "loop_inner.rs"]
mod loop_inner;
#[path = "loop_mock.rs"]
mod loop_mock;
#[path = "loop_mock_outcomes.rs"]
mod loop_mock_outcomes;
#[path = "loop_types.rs"]
mod loop_types;

#[cfg(test)]
#[path = "loop_driver_unit_tests.rs"]
mod loop_driver_unit_tests;
#[cfg(test)]
#[path = "loop_driver_tests.rs"]
mod loop_driver_tests;
#[cfg(test)]
#[path = "loop_driver_no_fence_tests.rs"]
mod loop_driver_no_fence_tests;

#[cfg(test)]
#[path = "loop_driver_kiss_cov.rs"]
mod loop_driver_kiss_cov;

pub use loop_inner::run_inner_loop;
#[cfg(test)]
pub(crate) use loop_inner_classify::classify_turn;
#[cfg(test)]
pub(crate) use loop_inner_finish::exhausted_error;
#[cfg(test)]
pub(crate) use loop_inner_prompt::push_user_prompt;
#[cfg(test)]
pub(crate) use loop_inner_types::TurnAction;
pub use loop_mock::{LlmBackend, MockScript, MockStep};
pub use loop_types::{LoopDriverConfig, LoopDriverOutcome, LoopDriverRun, LoopDriverSession};
#[allow(unused_imports)]
pub(crate) use loop_http::{HttpCompletionError, HttpRetryRequest};
#[allow(unused_imports)]
pub(crate) use loop_http_retry::{
    complete_with_http_retries, HttpRetryCounters, HttpRetryLimits,
};
