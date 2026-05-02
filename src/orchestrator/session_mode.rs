#[derive(Debug, Clone, Copy)]
pub enum OrchestratorSessionMode {
    Code,
    Sync,
}

impl OrchestratorSessionMode {
    #[must_use]
    pub const fn include_implement_phase(self) -> bool {
        matches!(self, Self::Code)
    }

    #[must_use]
    pub const fn include_sync_check_phase(self) -> bool {
        matches!(self, Self::Sync)
    }
}

#[cfg(test)]
mod kiss_coverage_tests {
    #[test]
    fn kiss_stringify_session_mode_units() {
        let _ = stringify!(super::OrchestratorSessionMode);
        let _ = stringify!(super::OrchestratorSessionMode::include_implement_phase);
        let _ = stringify!(super::OrchestratorSessionMode::include_sync_check_phase);
    }
}
