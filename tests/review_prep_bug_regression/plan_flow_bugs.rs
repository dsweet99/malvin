use super::helpers::{collect_unincluded_inc_orphans, manifest_root};

#[test]
fn plan_flow_root_inc_must_be_included_by_mod_rs() {
    let dir = manifest_root().join("src/cli/plan_flow");
    let orphans = collect_unincluded_inc_orphans(&dir);
    let has_root_orphan = orphans
        .iter()
        .any(|p| p.ends_with("plan_flow_root.inc"));
    assert!(
        !has_root_orphan,
        "bug: plan_flow_root.inc is not include!d; mod.rs inlines duplicate plan logic so \
         edits to the .inc do not compile (orphan duplicate): {}",
        orphans.join(", ")
    );
}

#[test]
fn plan_prompt_inc_must_not_exist_as_orphan_duplicate() {
    let inc = manifest_root().join("src/cli/plan_flow/plan_prompt.inc");
    assert!(
        !inc.is_file(),
        "bug: plan_prompt.inc is an orphan duplicate of plan_prompt.rs (`mod plan_prompt` \
         compiles only the .rs file); remove the .inc or wire it via include!"
    );
}

#[test]
fn plan_flow_must_not_keep_orphan_plan_resolve_rs() {
    let rs = manifest_root().join("src/cli/plan_flow/plan_resolve.rs");
    assert!(
        !rs.is_file(),
        "bug: plan_flow loads plan_resolve.inc via include! in mod.rs; a tracked plan_resolve.rs \
         is an orphan duplicate that drifts from the live .inc (kiss orphan_module)"
    );
}

#[test]
fn plan_flow_mod_must_include_plan_resolve_inc_not_rs_module() {
    let mod_rs = std::fs::read_to_string(manifest_root().join("src/cli/plan_flow/mod.rs"))
        .expect("read plan_flow/mod.rs");
    assert!(
        mod_rs.contains("include!(\"plan_resolve.inc\")"),
        "bug: plan resolver must live in plan_resolve.inc"
    );
    assert!(
        !mod_rs.contains("mod plan_resolve"),
        "bug: mod plan_resolve would compile orphan plan_resolve.rs instead of the .inc shard"
    );
}
