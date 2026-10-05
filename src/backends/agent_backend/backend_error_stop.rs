use crate::agent_process::{AgentError, AgentFault};

use super::backend_error_tracker::{
    format_backend_consecutive_error_message, format_local_backend_retry_cap_message,
};
use super::sdk_client::SdkClient;

pub const LOCAL_BACKEND_GPU_MEMORY_HINT: &str =
    "Hint: the local server may be out of GPU memory; see `malvin --doc`, section Local LLMs.";

const LOCAL_BACKEND_GPU_MEMORY_NEEDLES: &[&str] = &[
    "compute error",
    "unexpected content-type",
    "empty body",
    "empty response",
];

fn is_keyless_local(client: &SdkClient) -> bool {
    crate::local_llm::model_needs_local_llm(&client.model)
}

#[must_use]
pub fn error_suggests_local_gpu_memory(error: &str) -> bool {
    let text = error.to_ascii_lowercase();
    LOCAL_BACKEND_GPU_MEMORY_NEEDLES
        .iter()
        .any(|needle| text.contains(needle))
}

#[must_use]
pub(super) fn with_local_backend_hint(client: &SdkClient, message: String) -> String {
    if is_keyless_local(client) && error_suggests_local_gpu_memory(&message) {
        format!("{message}\n{LOCAL_BACKEND_GPU_MEMORY_HINT}")
    } else {
        message
    }
}

pub(super) fn backend_error_stop(client: &mut SdkClient, error: &str) -> Option<AgentError> {
    let label = client.model.backend.label();
    let message = if client.record_backend_error(error) {
        format_backend_consecutive_error_message(label, error, client.max_acp_retries.get())
    } else {
        let errors = is_keyless_local(client)
            .then(|| client.backend_error_tracker.local_retry_cap_errors())
            .flatten()?;
        format_local_backend_retry_cap_message(label, error, errors)
    };
    Some(AgentError {
        message: with_local_backend_hint(client, message),
        fault: AgentFault::BackendRetryLimit,
    })
}
