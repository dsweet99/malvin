//! `malvin delight` is a router-backed request wrapper.

#[cfg(unix)]
mod common;

#[cfg(unix)]
use common::{
    DelightSpawn, acp_mock_router_no_work_js, bin_path_with_fake_kiss, combined_cli_output,
    seed_malvin_checks, spawn_delight, fast_test_home_workspace, cached_mock_executable,
};

#[cfg(unix)]
#[test]
fn delight_router_succeeds_with_mock() {
    let (root, home, workspace) = fast_test_home_workspace();
    seed_malvin_checks(&workspace, "true\n");
    let path = bin_path_with_fake_kiss(&root);
    let mock = cached_mock_executable(&acp_mock_router_no_work_js());
    let out = spawn_delight(&DelightSpawn {
        workspace: &workspace,
        home: &home,
        mock: &mock,
        path_var: &path,
        extra_args: &["--max-loops", "1"],
    });
    assert!(
        out.status.success(),
        "delight must succeed via router: {:?}",
        combined_cli_output(&out)
    );
    let pitch = std::fs::read_to_string(workspace.join("pitch.md")).expect("read pitch");
    assert!(!pitch.is_empty(), "composed delight request should yield a pitch");
}

#[cfg(unix)]
#[test]
fn delight_embeds_guidance_in_router_request() {
    let (root, home, workspace) = fast_test_home_workspace();
    seed_malvin_checks(&workspace, "true\n");
    let path = bin_path_with_fake_kiss(&root);
    let mock = cached_mock_executable(&acp_mock_router_no_work_js());
    let out = spawn_delight(&DelightSpawn {
        workspace: &workspace,
        home: &home,
        mock: &mock,
        path_var: &path,
        extra_args: &["--max-loops", "1", "focus on latency UX"],
    });
    let combined = combined_cli_output(&out);
    assert!(out.status.success(), "delight with guidance must succeed: {combined:?}");
    // Startup emits the composed request; guidance must be visible in CLI output/logs.
    assert!(
        combined.contains("focus on latency UX") || combined.contains("User guidance"),
        "composed request must embed user guidance: {combined:?}"
    );
}

#[cfg(unix)]
#[test]
fn delight_allocates_sibling_when_default_pitch_preexists() {
    let (root, home, workspace) = fast_test_home_workspace();
    seed_malvin_checks(&workspace, "true\n");
    std::fs::write(workspace.join("pitch.md"), "existing\n").expect("seed pitch");
    let path = bin_path_with_fake_kiss(&root);
    let mock = cached_mock_executable(&acp_mock_router_no_work_js());
    let out = spawn_delight(&DelightSpawn {
        workspace: &workspace,
        home: &home,
        mock: &mock,
        path_var: &path,
        extra_args: &["--max-loops", "1"],
    });
    assert!(out.status.success(), "sibling alloc must allow router run: {:?}", combined_cli_output(&out));
    let stale = std::fs::read_to_string(workspace.join("pitch.md")).expect("read stale pitch");
    assert_eq!(stale, "existing\n", "original pitch.md must be untouched");
    assert!(
        workspace.join("pitch_1.md").exists(),
        "preflight must allocate pitch_1.md"
    );
}

#[cfg(unix)]
#[test]
fn delight_fails_when_custom_out_path_preexists() {
    let (root, home, workspace) = fast_test_home_workspace();
    seed_malvin_checks(&workspace, "true\n");
    std::fs::create_dir_all(workspace.join("plans")).expect("mkdir");
    std::fs::write(workspace.join("plans/existing.md"), "existing\n").expect("seed plan");
    let path = bin_path_with_fake_kiss(&root);
    let mock = cached_mock_executable(&acp_mock_router_no_work_js());
    let out = spawn_delight(&DelightSpawn {
        workspace: &workspace,
        home: &home,
        mock: &mock,
        path_var: &path,
        extra_args: &["--max-loops", "1", "--out-path", "plans/existing.md"],
    });
    let combined = combined_cli_output(&out);
    assert!(!out.status.success(), "expected failure when custom path exists: {combined:?}");
    assert!(
        combined.contains("refusing to overwrite"),
        "expected overwrite refusal: {combined:?}"
    );
}
