use super::helpers::{collect_unincluded_inc_orphans, manifest_root};

#[test]
fn mid_session_gates_must_not_keep_orphan_gates_root_inc() {
    let dir = manifest_root().join("src/cli/mid_session_gates");
    let orphans = collect_unincluded_inc_orphans(&dir);
    assert!(
        orphans.is_empty(),
        "bug: mid_session_gates has include!-orphan .inc shard(s); gates_root.inc duplicates \
         mod.rs but is never compiled:\n{}",
        orphans.join("\n")
    );
}
