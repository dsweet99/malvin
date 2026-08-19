#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SandboxSpawnPolicyAspect {
    ProcessGroupIsolation,
    MallocArenaCap,
    ParentDeathSignal,
    DeadBeforeNextSpawn,
    SessionRssMonitor,
    AcpSpawnLock,
}

impl SandboxSpawnPolicyAspect {
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
