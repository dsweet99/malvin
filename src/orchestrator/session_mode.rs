#[derive(Debug, Clone, Copy)]
pub(super) enum OrchestratorSessionMode {
    Code,
    Sync,
}

impl OrchestratorSessionMode {
    pub(super) const fn include_implement_phase(self) -> bool {
        matches!(self, Self::Code)
    }

    pub(super) const fn include_sync_check_phase(self) -> bool {
        matches!(self, Self::Sync)
    }
}
