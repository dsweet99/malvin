//! External kiss witnesses for [`crate::sandbox_oom`].

#[test]
fn kiss_witness_sandbox_oom_types() {
    use crate::sandbox_oom::SandboxOomKillFacts;
    let facts = SandboxOomKillFacts {
        reason: crate::sandbox_oom::OOM_REASON_MEMORY_LIMIT,
        rss_bytes: Some(1),
        limit_bytes: 1,
        pgid: 1,
    };
    let SandboxOomKillFacts {
        reason,
        rss_bytes,
        limit_bytes,
        pgid,
    } = facts;
    assert_eq!(reason, crate::sandbox_oom::OOM_REASON_MEMORY_LIMIT);
    assert_eq!(rss_bytes, Some(1));
    assert_eq!(limit_bytes, 1);
    assert_eq!(pgid, 1);
    let _ = crate::sandbox_oom::record_sandbox_oom_kill;
    let _ = crate::sandbox_oom::gate_iteration_oom_killed;
}
