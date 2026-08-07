//! Sandbox as session-scoped spawn policy (see `concepts.md` §5).
//!
//! All malvin-started subprocesses are expected to go through `malvin_std_command` /
//! `malvin_tokio_command` while an active coder session holds the sandbox slot. Production
//! references [`SandboxSpawnPolicyAspect`] at enforcement sites in `malvin_sandbox`.

/// One aspect of the ambient session sandbox spawn policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SandboxSpawnPolicyAspect {
    /// New children run in an isolated process group (`process_group(0)`).
    ProcessGroupIsolation,
    /// `MALLOC_ARENA_MAX=2` on sandbox child commands.
    MallocArenaCap,
    /// Linux: child gets `PR_SET_PDEATHSIG` so it dies if malvin is SIGKILL'd.
    ParentDeathSignal,
    /// `assert_dead_before_next_spawn` blocks a new session while prior PIDs live.
    DeadBeforeNextSpawn,
    /// `malvin_session_rss_bytes` monitors descendant USS against the workspace limit.
    SessionRssMonitor,
    /// `acquire_acp_spawn_lock` serializes ACP / mini session startup per work dir.
    AcpSpawnLock,
}

impl SandboxSpawnPolicyAspect {
    /// All policy aspects in stable concept order.
    #[must_use]
    pub const fn all() -> &'static [Self] {
        &[
            Self::ProcessGroupIsolation,
            Self::MallocArenaCap,
            Self::ParentDeathSignal,
            Self::DeadBeforeNextSpawn,
            Self::SessionRssMonitor,
            Self::AcpSpawnLock,
        ]
    }
}

#[cfg(test)]
#[path = "session_sandbox_policy_tests.rs"]
mod session_sandbox_policy_tests;
