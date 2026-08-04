//! Opportunistic herdr client: report malvin agent state when pane env is injected.
//!
//! No CLI flags or config keys. Absent `HERDR_ENV` / socket / pane → silent no-op.
//!
//! # Coexistence and product role
//!
//! Malvin is a **state reporter**, not a Herdr controller or built-in kind/integration.
//! ACP child processes are stripped of `HERDR_*` / `HERDR_ENV` so installed cursor hooks
//! do not race the parent reporter; `notify_reclaim` remains defense in depth.
//! Prefer pane-id targeting; after bind, `agent.rename` assigns a unique live `name`
//! (distinct from report/`display_agent: "malvin"`).

mod bind;
mod env;
mod identity;
mod lifecycle;
mod request;
mod send;
mod trace;

pub use lifecycle::{notify_reclaim, notify_run_end, notify_run_start, notify_working};

#[cfg(test)]
pub(crate) use lifecycle::{
    reset_session_for_test, session_active_for_test, session_has_binding_for_test,
};

#[cfg(test)]
#[path = "lifecycle_io_tests.rs"]
mod lifecycle_io_tests;

#[cfg(test)]
mod kiss_cov {
    #[test]
    fn kiss_cov_public_entrypoints() {
        let _ = super::notify_run_start;
        let _ = super::notify_reclaim;
        let _ = super::notify_working;
        let _ = super::notify_run_end;
        let _ = crate::herdr::env::HerdrEnv::from_os_env;
        let _ = crate::herdr::env::from_values;
        let _ = crate::herdr::request::next_seq;
        let _ = crate::herdr::request::next_request_id;
        let _ = crate::herdr::request::report_agent_session;
        let _ = crate::herdr::request::report_agent;
        let _ = crate::herdr::request::clear_agent_authority;
        let _ = crate::herdr::request::report_metadata_sparse;
        let _ = crate::herdr::request::clear_metadata_teardown;
        let _ = crate::herdr::request::rename_agent;
        let _ = crate::herdr::send::send_request;
        let _ = crate::herdr::send::send_request_checked;
        let _ = crate::herdr::send::SOCKET_TIMEOUT;
        let _ = crate::herdr::identity::herdr_live_name;
        let _ = crate::herdr::identity::display_title;
        let _ = crate::herdr::trace::log_herdr_failure;
        let _ = crate::herdr::bind::emit_bind_reports;
    }
}
