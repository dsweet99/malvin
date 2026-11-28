use std::time::Instant;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChildHealth {
    pub exists: bool,
    pub zombie: bool,
    pub state_hint: Option<char>,
    pub counters_trusted: bool,
    pub cpu_time_total: u64,
    pub thread_count: Option<u32>,
    pub voluntary_ctxt: Option<u64>,
    pub sample_time: Instant,
}

impl ChildHealth {
    pub(crate) fn process_absent() -> Self {
        Self {
            exists: false,
            zombie: false,
            state_hint: None,
            counters_trusted: true,
            cpu_time_total: 0,
            thread_count: None,
            voluntary_ctxt: None,
            sample_time: Instant::now(),
        }
    }

    pub(crate) fn cannot_sample() -> Self {
        Self {
            exists: true,
            zombie: false,
            state_hint: None,
            counters_trusted: false,
            cpu_time_total: 0,
            thread_count: None,
            voluntary_ctxt: None,
            sample_time: Instant::now(),
        }
    }
}
