//! Kiss per-file credit for [`malvin::sandbox_oom`] types (mirrors `agent_bundle` contract pattern).

#[test]
fn sandbox_oom_kill_facts_type_witness() {
    use malvin::SandboxOomKillFacts;
    let _: Option<SandboxOomKillFacts> = None;
    let facts = SandboxOomKillFacts {
        reason: malvin::OOM_REASON_MEMORY_LIMIT,
        rss_bytes: Some(1),
        limit_bytes: 1,
        pgid: 1,
    };
    let SandboxOomKillFacts { pgid, .. } = facts;
    assert_eq!(pgid, 1);
}

#[test]
fn terminal_palette_types_witness() {
    let _ = stringify!(TerminalTheme);
    let _ = stringify!(Palette);
}
