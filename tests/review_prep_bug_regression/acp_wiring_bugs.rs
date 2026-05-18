//! Bugs from `review_prep.md` § Bugs — untracked ACP shards wired via `mod` / `include!`.

use super::helpers::{assert_tracked_in_git, manifest_root};

const WIRED_ACP_SHARDS: &[(&str, &str)] = &[
    (
        "src/acp/jsonl_trace.rs",
        "mod jsonl_trace",
    ),
    (
        "src/acp/transport_tests_inline.inc",
        "transport_tests_inline.inc",
    ),
    (
        "src/acp/session_types_tests.inc",
        "session_types_tests.inc",
    ),
    (
        "src/acp/ops_inline_tests_unix.inc",
        "ops_inline_tests_unix.inc",
    ),
];

fn wiring_corpus() -> String {
    let root = manifest_root();
    let paths = [
        root.join("src/acp/mod.rs"),
        root.join("src/acp/session_types.rs"),
        root.join("src/acp/agent_bundle.rs"),
    ];
    paths
        .iter()
        .map(|p| std::fs::read_to_string(p).expect("read wiring source"))
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn acp_wired_include_shards_must_be_tracked_in_git() {
    let corpus = wiring_corpus();
    let mut missing = Vec::new();
    for (rel, needle) in WIRED_ACP_SHARDS {
        if !corpus.contains(needle) {
            continue;
        }
        let path = manifest_root().join(rel);
        assert!(
            path.is_file(),
            "bug: wired shard missing on disk: {}",
            path.display()
        );
        let tracked = std::process::Command::new("git")
            .args(["ls-files", "--error-unmatch", rel])
            .current_dir(manifest_root())
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);
        if !tracked {
            missing.push(*rel);
        }
    }
    assert!(
        missing.is_empty(),
        "bug: ACP shards are wired in the crate but not tracked in git (clone/CI break):\n{}",
        missing.join("\n")
    );
}

#[test]
fn jsonl_trace_must_be_tracked_when_mod_declared() {
    let acp_mod = std::fs::read_to_string(manifest_root().join("src/acp/mod.rs")).expect("mod.rs");
    if acp_mod.contains("mod jsonl_trace") {
        assert_tracked_in_git("src/acp/jsonl_trace.rs");
    }
}
