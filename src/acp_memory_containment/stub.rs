use std::path::Path;

use super::containment_state::OomBaseline;
use super::AcpMemoryContainment;

#[must_use]
pub fn inactive_containment() -> AcpMemoryContainment {
    AcpMemoryContainment::inactive()
}

#[must_use]
pub const fn memory_limit_oom_baseline_at(_cgroup_dir: &Path) -> OomBaseline {
    OomBaseline {
        events_oom_kill: 0,
        v1_under_oom: false,
    }
}

#[must_use]
pub const fn memory_limit_exceeded_since_baseline(
    _cgroup_dir: &Path,
    _baseline: OomBaseline,
) -> bool {
    false
}

#[cfg(test)]
mod stub_tests {
    use super::inactive_containment;

    #[test]
    fn inactive_containment_returns_inactive() {
        assert!(!inactive_containment().active());
    }
}
