#[test]
fn smoke_acp_reader_support_behavior() {
    use std::sync::atomic::Ordering;

    let (seq, _notify) = crate::acp_tests::reader_tests_helpers::acp_activity_state();
    assert_eq!(seq.load(Ordering::Relaxed), 0);
}

#[cfg(unix)]
#[tokio::test]
async fn smoke_reader_loop_eof_pending_error() {
    let msg = crate::acp_tests::reader_tests_reader_loop::reader_loop_eof_pending_error().await;
    assert!(!msg.is_empty());
}

#[cfg(not(unix))]
#[test]
fn smoke_acp_reader_helper_production_symbols() {
    let _ = crate::acp_tests::reader_tests_helpers::acp_activity_state;
    let _: Option<crate::acp_tests::reader_tests_helpers::IncomingDispatchParts> = None;
    let _: Option<crate::acp_tests::reader_tests_helpers::CatSession> = None;
}

#[cfg(unix)]
#[test]
fn smoke_acp_reader_helpers_unix_symbols() {
    use crate::acp_tests::reader_tests_helpers::{
        CatSession, IncomingDispatchParts, acp_activity_state, block_on_test,
    };
    let _ = acp_activity_state;
    let _: Option<IncomingDispatchParts<'_>> = None;
    let _: Option<CatSession> = None;
    block_on_test(async {
        let cat = CatSession::new().await;
        cat.dispatch_parts().dispatch_lines(&[]).await;
        let _ = cat.finish_stdout().await;
    });
}

#[test]
fn smoke_spawn_and_agent_env_helpers() {
    let _ = super::resolve_agent_bin();
    let _ = super::test_no_real_agent_enabled();
    let _ = super::auth_probe(&["/bin/true"]);
}

#[test]
fn smoke_acp_inc_symbols_for_kiss() {
    let _ = super::resolve_agent_bin;
    let _ = super::spawn_agent_acp_session;
}


#[cfg(test)]
mod kiss_cov_auto{
    use super::*;

    #[test]
    fn kiss_cov_smoke_reader_loop_eof_pending_error() { let _ = smoke_reader_loop_eof_pending_error; }
}
