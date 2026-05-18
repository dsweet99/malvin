use super::helpers::{
    collect_orchestrator_orphan_inc_paths, crate_top_modules_in_stringify_refs,
    lib_rs_top_level_module_names, manifest_root,
};

#[test]
fn stringify_refs_must_not_use_fake_submodule_paths() {
    let stringify_refs = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/coverage_kiss/stringify_refs.inc"
    ));
    for fake in [
        "acp::session_channels::",
        "acp::session_io::",
        "acp_memory_containment::linux_verify_abort::",
        "coverage_kiss::paths::",
        "output::terminal_wrap_a::",
        "acp::reader_stdout_body_b::",
        "acp::session_prompt_helpers::",
        "acp::client_impl_helpers::",
        "acp::client_impl_prompt_dispatch::",
        "orchestrator::run_code_review_phase",
    ] {
        assert!(
            !stringify_refs.contains(fake),
            "bug: stringify_refs.inc must not credit kiss with fake path `{fake}`"
        );
    }
}

#[test]
fn stringify_refs_must_not_reference_missing_lib_rs_modules() {
    let lib_rs = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/lib.rs"));
    let stringify_refs = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/coverage_kiss/stringify_refs.inc"
    ));
    let declared = lib_rs_top_level_module_names(lib_rs);
    let referenced = crate_top_modules_in_stringify_refs(stringify_refs);
    let missing: Vec<String> = referenced
        .iter()
        .filter(|name| !declared.contains(name.as_str()))
        .cloned()
        .collect();
    assert!(
        missing.is_empty(),
        "bug: stringify_refs.inc names crate modules not declared in lib.rs (kiss credit \
         without a real symbol path): {}",
        missing.join(", ")
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
