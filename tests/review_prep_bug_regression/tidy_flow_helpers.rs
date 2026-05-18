use super::helpers::{assert_tracked_in_git, collect_unincluded_inc_orphans, manifest_root};

#[test]
fn tidy_flow_prelude_inc_must_not_be_tracked_include_orphan() {
    let rel = "src/cli/tidy_flow/prelude.inc";
    let path = manifest_root().join(rel);
    if !path.is_file() {
        return;
    }
    let orphans = collect_unincluded_inc_orphans(&manifest_root().join("src/cli/tidy_flow"));
    let prelude_orphan = orphans.iter().any(|p| p.ends_with("prelude.inc"));
    let tracked = std::process::Command::new("git")
        .args(["ls-files", "--error-unmatch", rel])
        .current_dir(manifest_root())
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);
    assert!(
        !(prelude_orphan && tracked),
        "bug: {rel} is in git but never include!d under src/cli/tidy_flow/; edits there do not \
         compile (delete or wire via include!)"
    );
    if tracked {
        assert_tracked_in_git(rel);
    }
}

#[test]
fn tidy_flow_helpers_root_inc_must_not_be_tracked_orphan() {
    let tidy_flow = std::fs::read_to_string(manifest_root().join("src/cli/tidy_flow.rs"))
        .expect("read tidy_flow.rs");
    let inc = manifest_root().join("src/cli/tidy_flow/helpers_root.inc");
    assert!(
        !inc.is_file(),
        "bug: delete orphan helpers_root.inc; live wiring is helpers_root_body.inc only"
    );
    assert!(
        tidy_flow.contains("include!(\"tidy_flow/helpers_root_body.inc\")"),
        "bug: tidy_flow.rs must wire helpers via helpers_root_body.inc"
    );
    assert!(
        !tidy_flow.contains("helpers_root.inc"),
        "bug: tidy_flow.rs must not reference stale helpers_root.inc"
    );
    let orphans = collect_unincluded_inc_orphans(&manifest_root().join("src/cli/tidy_flow"));
    assert!(
        !orphans.iter().any(|p| p.ends_with("helpers_root.inc")),
        "bug: helpers_root.inc is tracked but never include!d; edits there do not compile:\n{}",
        orphans.join(", ")
    );
}

const LIVE_INC_TO_ORPHAN_RS: &[(&str, &str)] = &[
    ("src/cli/tidy_flow/prep.inc", "src/cli/tidy_flow/helpers/prep.rs"),
    ("src/cli/tidy_flow/prompt.inc", "src/cli/tidy_flow/helpers/prompt.rs"),
    (
        "src/cli/tidy_flow/interleaved_loop.inc",
        "src/cli/tidy_flow/helpers/interleaved_loop.rs",
    ),
    (
        "src/cli/tidy_flow/recovery.inc",
        "src/cli/tidy_flow/helpers/recovery.rs",
    ),
    ("src/cli/tidy_flow/run.inc", "src/cli/tidy_flow/helpers/run.rs"),
    (
        "src/cli/tidy_flow/run_startup.inc",
        "src/cli/tidy_flow/helpers/run_startup.rs",
    ),
];

#[test]
fn tidy_flow_wires_helpers_only_through_helpers_root_inc() {
    let tidy_flow = std::fs::read_to_string(manifest_root().join("src/cli/tidy_flow.rs"))
        .expect("read tidy_flow.rs");
    assert!(
        tidy_flow.contains("include!(\"tidy_flow/helpers_root_body.inc\")"),
        "bug: expected tidy_flow helpers via helpers_root_body.inc"
    );
    assert!(
        !tidy_flow.contains("tidy_flow/helpers/mod.rs"),
        "bug: tidy_flow must not also mod-include the orphan helpers/*.rs tree"
    );
}

#[test]
fn tidy_flow_must_not_keep_orphan_helpers_rs_directory() {
    let helpers_dir = manifest_root().join("src/cli/tidy_flow/helpers");
    assert!(
        !helpers_dir.is_dir(),
        "bug: {helpers_dir:?} is not compiled (build uses helpers_root.inc + .inc shards only); \
         delete the orphan .rs tree so edits are not applied to the wrong copy"
    );
}

#[test]
fn tidy_flow_orphan_helpers_rs_must_not_drift_from_live_inc() {
    let root = manifest_root();
    let mut drifted = Vec::new();
    for (inc_rel, rs_rel) in LIVE_INC_TO_ORPHAN_RS {
        let inc_path = root.join(inc_rel);
        let rs_path = root.join(rs_rel);
        if !rs_path.is_file() {
            continue;
        }
        let inc = std::fs::read_to_string(&inc_path).expect("read inc");
        let rs = std::fs::read_to_string(&rs_path).expect("read rs");
        if inc != rs {
            drifted.push(format!("{rs_rel} != {inc_rel}"));
        }
    }
    assert!(
        drifted.is_empty(),
        "bug: orphan helpers/*.rs drifts from live .inc sources; wrong tree was edited:\n{}",
        drifted.join("\n")
    );
}

#[test]
fn tidy_flow_orphan_helpers_mod_rs_must_not_exist() {
    let mod_rs = manifest_root().join("src/cli/tidy_flow/helpers/mod.rs");
    assert!(
        !mod_rs.is_file(),
        "bug: {mod_rs:?} suggests a second module tree but is never wired from tidy_flow.rs"
    );
}

#[test]
fn tidy_flow_helpers_tests_rs_must_not_remain() {
    let tests_rs = manifest_root().join("src/cli/tidy_flow/helpers/tests.rs");
    assert!(
        !tests_rs.is_file(),
        "bug: helpers/tests.rs stringify!s super:: paths for a module tree that is no longer built"
    );
}
