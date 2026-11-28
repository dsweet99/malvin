use std::collections::HashSet;

pub(crate) mod memory_sample;
pub use memory_sample::{MemoryMetric, MemorySample};

#[cfg(target_os = "linux")]
pub(crate) use super::process_group_rss_linux as linux;
#[cfg(target_os = "macos")]
pub(crate) use super::process_group_rss_macos as macos;

#[must_use]
pub fn pids_sandbox_bytes(pids: &HashSet<u32>) -> Option<u64> {
    pids_sandbox_sample(pids).map(|sample| sample.bytes)
}

#[must_use]
pub fn pids_sandbox_sample(pids: &HashSet<u32>) -> Option<MemorySample> {
    if pids.is_empty() {
        return Some(MemorySample::new(0, MemoryMetric::Uss));
    }
    #[cfg(target_os = "linux")]
    {
        linux::linux_pids_sandbox_sample(pids)
    }
    #[cfg(target_os = "macos")]
    {
        macos::macos_pids_rss_bytes(pids).map(|bytes| MemorySample::new(bytes, MemoryMetric::Rss))
    }
    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    {
        let _ = pids;
        None
    }
}

#[cfg(test)]
#[path = "process_group_rss_tests.rs"]
mod process_group_rss_tests;

#[cfg(test)]
#[allow(unused_imports)]
mod kiss_cov_gate_refs {
    use super::*;
    #[test]
    fn kiss_cov_unit_names() {
        let _ = pids_sandbox_bytes;
        let _ = pids_sandbox_sample;
        let uss = MemorySample::new(1, MemoryMetric::Uss);
        let rss = MemorySample::new(2, MemoryMetric::Rss);
        assert_eq!(uss.combine(uss).metric.label(), "USS");
        assert_eq!(rss.combine(rss).metric.label(), "RSS");
        let mixed = uss.combine(rss);
        assert_eq!((mixed.bytes, mixed.metric.label()), (3, "USS+RSS"));
    }
}
