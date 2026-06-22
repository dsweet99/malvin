//! Inner bash-fence loop for one `run_coder_prompt`.

#[path = "loop_http.rs"]
mod loop_http;
#[path = "loop_inner.rs"]
mod loop_inner;
#[path = "loop_mock.rs"]
mod loop_mock;
#[path = "loop_types.rs"]
mod loop_types;
pub use loop_inner::run_inner_loop;
pub(crate) use loop_inner::{classify_turn, exhausted_error, push_user_prompt, TurnAction, TurnContext};
pub use loop_mock::{LlmBackend, MockScript, MockStep};
pub use loop_types::{LoopDriverConfig, LoopDriverRun, LoopDriverSession};
#[cfg(test)]
#[path = "loop_driver_test.rs"]
mod loop_driver_test;
