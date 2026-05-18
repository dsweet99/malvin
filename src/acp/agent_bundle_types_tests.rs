use crate::acp::{AgentError, AuthError};

#[test]
fn agent_error_and_auth_error_display() {
    let ae = AgentError("agent".into());
    assert_eq!(format!("{ae}"), "agent");
    let auth = AuthError("auth".into());
    assert_eq!(format!("{auth}"), "auth");
}
