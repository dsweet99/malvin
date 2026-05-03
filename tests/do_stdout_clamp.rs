mod common;

#[cfg(unix)]
use common::{
    acp_mock_do_streaming_update_js, prepare_do_auto_clamp_case, prepare_do_skip_clamp_case,
    run_do_say_hi_path_prefixed,
};

#[cfg_attr(unix, test)]
fn do_auto_runs_kiss_clamp_by_default_when_source_exists_and_kissconfig_missing() {
    let (ctx, marker, kissconfig) = prepare_do_auto_clamp_case(&acp_mock_do_streaming_update_js());
    let out = run_do_say_hi_path_prefixed(&ctx);
    assert!(
        out.status.success(),
        "malvin do failed: {:?}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(marker.exists(), "expected kiss clamp to run");
    assert_eq!(
        std::fs::read_to_string(&kissconfig).expect("read kissconfig"),
        "k\n"
    );
}

#[cfg_attr(unix, test)]
fn do_does_not_run_kiss_clamp_when_kissconfig_exists() {
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
        "did not expect kiss clamp to run when .kissconfig exists"
    );
    assert_eq!(
        std::fs::read_to_string(ctx.workspace.join(".kissconfig")).expect("read kissconfig"),
        existing
    );
}
