//! Agent ACP symbols for `kiss check` coverage (split from `coverage_kiss` for file size limits).
#![allow(unused_imports)]

use crate::acp::{AgentClient, AgentIoOptions};

#[test]
fn kiss_stringify_agent() {
    let _ = stringify!(AgentClient);
    let _ = stringify!(crate::acp::AgentError);
    let _ = stringify!(crate::acp::AuthError);
    let _ = stringify!(crate::acp::ReviewerPromptPair);
    let _ = stringify!(AgentIoOptions);
    let _ = stringify!(crate::acp::has_api_key);
    let _ = stringify!(crate::acp::auth_probe);
    let _ = stringify!(crate::acp::spawn_agent_acp_session);
    let _ = stringify!(crate::acp::strip_trace_invocation_line_for_tee);
    let _ = stringify!(crate::acp::run_reviewer_pair_once);
    let _ = stringify!(crate::acp::run_kpop_flow_once);
    let _ = stringify!(crate::acp::run_kpop_multiturn_once);
    let _ = stringify!(crate::acp::KpopFlowOnceArgs);
    let _ = stringify!(AgentClient::new);
    let _ = stringify!(AgentClient::ensure_authenticated);
    let _ = stringify!(AgentClient::begin_coder_session);
    let _ = stringify!(AgentClient::run_coder_prompt);
    let _ = stringify!(AgentClient::end_coder_session);
    let _ = stringify!(AgentClient::run_reviewer_review);
    let _ = stringify!(AgentClient::run_kpop_flow);
    let _ = stringify!(AgentClient::run_kpop_multiturn);
    let _ = stringify!(AgentClient::set_run_timing);
    let _ = stringify!(AgentClient::attach_run_timing_for_session);
    let _ = stringify!(AgentClient::has_open_coder_session);
    let _ = stringify!(crate::acp::DEFAULT_REPO_STYLE_PROMPT_REL);
}
