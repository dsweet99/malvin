use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct OomBaseline {
    pub events_oom_kill: u64,
    pub v1_under_oom: bool,
}

#[derive(Debug)]
pub(in crate::acp_memory_containment) struct ContainmentState {
    pub(in crate::acp_memory_containment) active: AtomicBool,
    pub(in crate::acp_memory_containment) cgroup_dir: Mutex<Option<PathBuf>>,
    pub(in crate::acp_memory_containment) oom_latched: AtomicBool,
    pub(in crate::acp_memory_containment) oom_baseline: OomBaseline,
}

#[derive(Clone, Debug)]
pub struct AcpMemoryContainment {
    pub(in crate::acp_memory_containment) state: Arc<ContainmentState>,
}

impl AcpMemoryContainment {
    pub(in crate::acp_memory_containment) fn from_parts(
        active: bool,
        cgroup_dir: Option<PathBuf>,
    ) -> Self {
        let oom_baseline = cgroup_dir
            .as_deref()
            .map(super::memory_limit_oom_baseline_at)
            .unwrap_or_default();
        let v1_only = cgroup_dir
            .as_ref()
            .is_some_and(|dir| active && !dir.join("memory.events").is_file());
        let oom_latched = v1_only && oom_baseline.v1_under_oom;
        Self {
            state: Arc::new(ContainmentState {
                active: AtomicBool::new(active),
                cgroup_dir: Mutex::new(cgroup_dir),
                oom_latched: AtomicBool::new(oom_latched),
                oom_baseline,
            }),
        }
    }

    #[must_use]
    pub fn inactive() -> Self {
        Self::from_parts(false, None)
    }

    #[must_use]
    pub fn active(&self) -> bool {
        self.state.active.load(Ordering::Relaxed)
    }

    #[must_use]
    pub fn memory_limit_exceeded(&self) -> bool {
        if self.state.oom_latched.load(Ordering::Relaxed) {
            return true;
        }
        if !self.active() {
            return false;
        }
        let cgroup_dir = self
            .state
            .cgroup_dir
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone();
        let Some(dir) = cgroup_dir.as_deref() else {
            return false;
        };
        super::memory_limit_exceeded_since_baseline(dir, self.state.oom_baseline)
    }

    #[cfg(all(test, target_os = "linux"))]
    pub fn cgroup_leaf_snapshot_for_tests(&self) -> Option<PathBuf> {
        self.state
            .cgroup_dir
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
    }
}

fn latch_oom_if_needed(state: &ContainmentState, had_oom: bool) {
    if had_oom {
        state.oom_latched.store(true, std::sync::atomic::Ordering::Relaxed);
    }
}

fn teardown_containment_state(state: &ContainmentState) {
    let cgroup_dir = state
        .cgroup_dir
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .take();
    if let Some(dir) = cgroup_dir {
        super::remove_cgroup_dir_at(&dir);
    }
    state
        .active
        .store(false, std::sync::atomic::Ordering::Relaxed);
}

pub fn finalize_containment_cgroup(containment: &AcpMemoryContainment) {
    let had_oom = containment.memory_limit_exceeded();
    let state = &containment.state;
    latch_oom_if_needed(state, had_oom);
    teardown_containment_state(state);
}

#[cfg(test)]
mod containment_state_tests {
    use super::{AcpMemoryContainment, ContainmentState, OomBaseline};

    #[test]
    fn oom_baseline_default_and_containment_state_fields() {
        let baseline = OomBaseline::default();
        assert_eq!(baseline.events_oom_kill, 0);
        let c = AcpMemoryContainment::inactive();
        assert!(!c.active());
        let _ = std::mem::size_of::<ContainmentState>();
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn v1_under_oom_at_activation_latches_memory_limit_exceeded() {
        let dir = tempfile::tempdir().expect("tempdir");
        std::fs::write(dir.path().join("memory.limit_in_bytes"), "1048576").expect("limit");
        std::fs::write(
            dir.path().join("memory.oom_control"),
            "oom_kill_disable 0\nunder_oom 1\n",
        )
        .expect("oom_control");
        assert!(!dir.path().join("memory.events").exists());
        let c = AcpMemoryContainment::from_parts(true, Some(dir.path().to_path_buf()));
        assert!(c.memory_limit_exceeded());
        std::fs::write(
            dir.path().join("memory.oom_control"),
            "oom_kill_disable 0\nunder_oom 0\n",
        )
        .expect("clear under_oom");
        assert!(c.memory_limit_exceeded());
    }

    #[test]
    fn finalize_containment_cgroup_runs_teardown_on_inactive() {
        let c = AcpMemoryContainment::inactive();
        super::finalize_containment_cgroup(&c);
        assert!(!c.active());
    }

    #[test]
    fn kiss_stringify_containment_state_units() {
        let _ = stringify!(super::latch_oom_if_needed);
        let _ = stringify!(super::teardown_containment_state);
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn finalize_containment_latches_oom_when_under_oom_at_teardown() {
        let dir = tempfile::tempdir().expect("tempdir");
        std::fs::write(dir.path().join("memory.limit_in_bytes"), "1048576").expect("limit");
        std::fs::write(
            dir.path().join("memory.oom_control"),
            "oom_kill_disable 0\nunder_oom 1\n",
        )
        .expect("oom_control");
        let c = AcpMemoryContainment::from_parts(true, Some(dir.path().to_path_buf()));
        super::finalize_containment_cgroup(&c);
        assert!(c.memory_limit_exceeded());
    }
}
