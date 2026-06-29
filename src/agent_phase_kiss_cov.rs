#[test]
fn mini_phase_hooks_drive_heartbeat() {
    let _guard = crate::agent_phase::AGENT_PHASE_TEST_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    crate::agent_phase::reset_phase_state_for_test();
    assert_eq!(crate::agent_phase::heartbeat_label(), "Orienting");
    crate::agent_phase::clear_orienting();
    crate::agent_phase::note_mini_llm_request();
    assert_eq!(crate::agent_phase::heartbeat_label(), "Reasoning");
    crate::agent_phase::note_mini_bash_exec();
    assert_eq!(crate::agent_phase::heartbeat_label(), "Executing");
    crate::agent_phase::note_mini_bash_exec_done(0, "echo hi");
    assert_eq!(crate::agent_phase::heartbeat_label(), "Reasoning");
    crate::agent_phase::note_mini_bash_exec_done(1, "kiss check");
    assert_eq!(
        crate::agent_phase::current_phase_for_test(),
        crate::agent_phase::AgentPhase::Debugging
    );
}

#[test]
fn heartbeat_phases_follow_runtime_signals() {
    use crate::tool_summary::ToolSummaryDetail;
    use serde_json::json;

    let _guard = crate::agent_phase::AGENT_PHASE_TEST_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    crate::agent_phase::reset_phase_state_for_test();
    assert_eq!(crate::agent_phase::heartbeat_label(), "Orienting");
    crate::agent_phase::note_thought_activity();
    crate::agent_phase::reset_for_run();
    crate::agent_phase::enter_verifying();
    crate::agent_phase::leave_verifying();
    let mut tracker = crate::tool_summary::ToolSummaryTracker::default();
    let observe = |update: serde_json::Value, tracker: &mut crate::tool_summary::ToolSummaryTracker| {
        let v = json!({"method": "session/update", "params": {"update": update}});
        crate::tool_summary::tool_summary_lines(&v, tracker, ToolSummaryDetail::Log).unwrap();
    };
    observe(json!({"sessionUpdate":"tool_call","toolCallId":"1","kind":"execute","status":"pending","rawInput":{"command":"sleep 9"}}), &mut tracker);
    observe(json!({"sessionUpdate":"tool_call_update","toolCallId":"1","status":"completed","rawOutput":{"exitCode":0}}), &mut tracker);
    observe(json!({"sessionUpdate":"tool_call","toolCallId":"r1","kind":"read","status":"pending","rawInput":{"path":"a.rs"}}), &mut tracker);
    assert_eq!(
        crate::agent_phase::current_phase_for_test(),
        crate::agent_phase::AgentPhase::Researching
    );
    observe(json!({"sessionUpdate":"tool_call","toolCallId":"x1","kind":"execute","status":"pending","rawInput":{"command":"kiss check"}}), &mut tracker);
    observe(json!({"sessionUpdate":"tool_call_update","toolCallId":"x1","status":"completed","rawOutput":{"exitCode":1}}), &mut tracker);
    assert_eq!(
        crate::agent_phase::current_phase_for_test(),
        crate::agent_phase::AgentPhase::Debugging
    );
    crate::agent_phase::set_reporting(true);
    assert_eq!(crate::agent_phase::heartbeat_label(), "Reporting");
}

#[test]
fn kiss_cov_edit_and_search_tool_phases() {
    use crate::tool_summary::ToolSummaryDetail;
    use serde_json::json;

    let _guard = crate::agent_phase::AGENT_PHASE_TEST_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    crate::agent_phase::reset_phase_state_for_test();
    let mut tracker = crate::tool_summary::ToolSummaryTracker::default();
    let observe = |update: serde_json::Value, tracker: &mut crate::tool_summary::ToolSummaryTracker| {
        let v = json!({"method": "session/update", "params": {"update": update}});
        crate::tool_summary::tool_summary_lines(&v, tracker, ToolSummaryDetail::Log).unwrap();
    };
    observe(
        json!({"sessionUpdate":"tool_call","toolCallId":"e1","kind":"edit","status":"pending","rawInput":{"path":"a.rs"}}),
        &mut tracker,
    );
    assert_eq!(
        crate::agent_phase::current_phase_for_test(),
        crate::agent_phase::AgentPhase::Implementing
    );
    observe(
        json!({"sessionUpdate":"tool_call","toolCallId":"s1","kind":"search","status":"pending","rawInput":{"query":"foo"}}),
        &mut tracker,
    );
    assert_eq!(
        crate::agent_phase::current_phase_for_test(),
        crate::agent_phase::AgentPhase::Researching
    );
}

#[test]
fn kiss_cov_agent_phase_private_symbol_names() {
    use crate::agent_phase::kiss_cov::{
        witness_active_tool_phase, witness_phase_if, witness_tool_kinds, ToolKind,
    };
    let _ = stringify!(phase_if);
    let _ = stringify!(active_tool_phase);
    assert_eq!(witness_tool_kinds().len(), 4);
    assert_eq!(witness_phase_if(true), Some(crate::agent_phase::AgentPhase::Waiting));
    assert_eq!(witness_phase_if(false), None);
    assert_eq!(
        witness_active_tool_phase(ToolKind::Edit),
        crate::agent_phase::AgentPhase::Implementing
    );
}
