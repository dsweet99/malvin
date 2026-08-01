mod common;

#[cfg(unix)]
use common::{
    acp_mock_do_streaming_update_js, prepare_do_auto_clamp_case, prepare_do_skip_clamp_case,
    run_do_say_hi_path_prefixed,
};

#[cfg_attr(unix, test)]
fn do_does_not_run_clamp_when_snapshotting() {
    let (ctx, marker, _kissconfig) = prepare_do_auto_clamp_case(&acp_mock_do_streaming_update_js());
    let out = run_do_say_hi_path_prefixed(&ctx);
    assert!(
        out.status.success(),
        "malvin do failed: {:?}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        !marker.exists(),
        "malvin do must not run clamp during snapshot"
    );
}

#[cfg_attr(unix, test)]
fn do_leaves_existing_kissconfig_unchanged_when_present() {
    let existing = "k\n";
    let (ctx, marker) = prepare_do_skip_clamp_case(&acp_mock_do_streaming_update_js(), existing);
    let out = run_do_say_hi_path_prefixed(&ctx);
    assert!(
        out.status.success(),
        "malvin do failed: {:?}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        !marker.exists(),
        "malvin do must not invoke clamp"
    );
    assert_eq!(
        std::fs::read_to_string(ctx.workspace.join(".kissconfig")).expect("read kissconfig"),
        existing
    );
}
