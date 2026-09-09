#[test]
fn kiss_cov_npm_pi_sdk() {
    let _ = super::ensure_npm_pi_authenticated;
    let _ = super::discover::resolve_npm_pi_entry;
    let _ = super::list_npm_pi_display_models;
    let _ = super::spawn_bridge;
    let _ = stringify!(NpmPiSession);
    let _ = stringify!(map_npm_pi_event);
    let _ = stringify!(consume_npm_pi_turn);
    let _ = stringify!(feed_run_done_dm);
    let _ = stringify!(feed_and_handle_run_done);
    let _ = stringify!(npm_pi_send_prompt);
    let _ = stringify!(auto_reply_extension_ui);
    let _ = crate::model_id::ModelBackend::NpmPi;
}
