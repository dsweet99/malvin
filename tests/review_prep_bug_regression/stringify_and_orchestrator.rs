use super::helpers::{collect_orchestrator_orphan_inc_paths, manifest_root};

#[test]
fn coverage_kiss_must_not_ship_unwired_stringify_refs_inc() {
    let path = manifest_root().join("src/coverage_kiss/stringify_refs.inc");
    assert!(
        !path.is_file(),
        "bug: stringify_refs.inc is dead kiss gaming; remove it or wire only real tests in \
         coverage_kiss/mod.rs"
    );
}

#[test]
fn orchestrator_must_not_keep_orphan_inc_sources() {
    let orchestrator_dir = manifest_root().join("src/orchestrator");
    let orphans = collect_orchestrator_orphan_inc_paths(&orchestrator_dir);
    assert!(
        orphans.is_empty(),
        "bug: src/orchestrator/*.inc are not include!d anywhere; remove orphan copies:\n{}",
        orphans.join("\n")
    );
}
