//! Inner bash-fence loop for one `run_coder_prompt`.

#[path = "loop_http.rs"]
mod loop_http;
#[path = "loop_inner.rs"]
mod loop_inner;
#[path = "loop_mock.rs"]
mod loop_mock;
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

pub use loop_inner::run_inner_loop;
#[cfg(test)]
pub(crate) use loop_inner::{classify_turn, exhausted_error, push_user_prompt, TurnAction, TurnContext};
pub use loop_mock::{LlmBackend, MockScript, MockStep};
pub use loop_types::{LoopDriverConfig, LoopDriverRun, LoopDriverSession};
