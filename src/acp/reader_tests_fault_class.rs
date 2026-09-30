use crate::acp::{AgentError, AgentFault, AgentRetryOutcome, AttemptCeiling, plan_agent_retry};

#[test]
fn constructor_classifies_text_and_retry_uses_the_fault() {
    let fatal = AgentError("upgrade your plan to continue".into());
    assert_eq!(fatal.fault, AgentFault::NonRetryable);
    let err = plan_agent_retry(&fatal, 1, AttemptCeiling::unlimited()).expect_err("fatal");
    assert_eq!(err.message, fatal.message);

    let plain = AgentError::ordinary("upgrade your plan to continue");
    assert_eq!(plain.fault, AgentFault::Ordinary);
    assert!(matches!(
        plan_agent_retry(&plain, 1, AttemptCeiling::unlimited()).expect("retry"),
        AgentRetryOutcome::Sleep(_)
    ));

    let busy = AgentError("already has active run".into());
    assert_eq!(busy.fault, AgentFault::CursorBusy);
    assert!(busy.requires_coder_session_teardown());

    let dead = AgentError("bridge stdout closed".into());
    assert_eq!(dead.fault, AgentFault::SessionDead);
    assert!(dead.requires_coder_session_teardown());
}

#[test]
fn unlimited_ceiling_does_not_stop_on_attempt_count() {
    let err = AgentError::ordinary("timed out");
    let out = plan_agent_retry(&err, u32::MAX, AttemptCeiling::unlimited()).expect("sleep");
    assert!(matches!(out, AgentRetryOutcome::Sleep(_)));
}
