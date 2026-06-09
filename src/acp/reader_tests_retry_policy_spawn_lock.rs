use crate::acp::{AgentRetryOutcome, plan_agent_retry};
use crate::support_paths::DEFAULT_MAX_ACP_RETRIES;

const TEST_MAX_ATTEMPTS: u32 = DEFAULT_MAX_ACP_RETRIES;

#[test]
fn acp_spawn_lock_errors_stop_retrying_without_sleep() {
    let msg = "ACP spawn lock held by pid 42 at /tmp/work/.malvin/acp_spawn.lock; nested malvin sessions cannot spawn another agent while a parent ACP session is active in this workspace";
    assert!(crate::acp_spawn_lock::agent_string_is_acp_spawn_lock_held(msg));
    let out = plan_agent_retry(msg, 1, TEST_MAX_ATTEMPTS).unwrap();
    assert!(matches!(out, AgentRetryOutcome::StopRetrying), "{out:?}");
}
