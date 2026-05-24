use std::path::Path;

use super::AcpMemoryContainment;
use super::containment_state::OomBaseline;

#[must_use]
pub fn inactive_containment() -> AcpMemoryContainment {
    AcpMemoryContainment::inactive()
}

#[must_use]
pub const fn inactive_platform_memory_limit_oom_baseline_at(_cgroup_dir: &Path) -> OomBaseline {
    OomBaseline {
        events_oom_kill: 0,
        v1_under_oom: false,
    }
}

#[must_use]
pub const fn inactive_platform_memory_limit_exceeded_since_baseline(
    _cgroup_dir: &Path,
    _baseline: OomBaseline,
) -> bool {
    false
}

#[cfg(test)]
mod stub_tests {
    use super::{
        inactive_containment, inactive_platform_memory_limit_exceeded_since_baseline,
        inactive_platform_memory_limit_oom_baseline_at,
    };

    #[test]
    fn inactive_containment_returns_inactive() {
        assert!(!inactive_containment().active());
    }

    #[test]
    fn kiss_cov_src_acp_memory_containment_stub_rs_inactive_platform_memory_limit_oom_baseline_at(
    ) {
        let dir = tempfile::tempdir().expect("tempdir");
        let baseline = inactive_platform_memory_limit_oom_baseline_at(dir.path());
        assert_eq!(baseline.events_oom_kill, 0);
    }

    #[test]
    fn kiss_cov_src_acp_memory_containment_stub_rs_inactive_platform_memory_limit_exceeded_since_baseline(
    ) {
        let dir = tempfile::tempdir().expect("tempdir");
        let baseline = inactive_platform_memory_limit_oom_baseline_at(dir.path());
        assert!(!inactive_platform_memory_limit_exceeded_since_baseline(
            dir.path(),
            baseline
        ));
    }
}
