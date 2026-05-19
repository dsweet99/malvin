//! Linux cgroup v2 (and v1 memory fallback) memory limits for `agent acp` children only.
#![allow(unsafe_code)]

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

#[cfg(target_os = "linux")]
#[allow(unsafe_code)]
include!("cgroup_memory.inc");
include!("linux_fs.inc");
include!("linux_parent_death.inc");
include!("linux_spawn.inc");

#[cfg(not(target_os = "linux"))]
mod stub;

pub const AGENT_EXCEEDED_MEMORY_LIMIT_MSG: &str = "agent exceeded memory limit";
pub const CONTAINMENT_UNAVAILABLE_WARN: &str =
    "warning: ACP memory containment unavailable; running agent without cgroup memory limit";

static CGROUP_SEQ: AtomicU64 = AtomicU64::new(0);

mod containment_state;

pub use containment_state::AcpMemoryContainment;
#[allow(unused_imports)]
pub use containment_state::OomBaseline;
pub use containment_state::finalize_containment_cgroup;

#[cfg(not(target_os = "linux"))]
pub fn half_physical_memory_bytes() -> Option<u64> {
    None
}

#[must_use]
pub fn map_acp_child_exit_message(containment: &AcpMemoryContainment, default: &str) -> String {
    if containment.memory_limit_exceeded() {
        AGENT_EXCEEDED_MEMORY_LIMIT_MSG.to_string()
    } else {
        default.to_string()
    }
}

pub enum ContainmentHandle {
    #[cfg(target_os = "linux")]
    Linux {
        cgroup_dir: PathBuf,
        memory_max_bytes: u64,
    },
    Inactive,
}

pub fn begin_containment_for_command(cmd: &mut tokio::process::Command) -> ContainmentHandle {
    #[cfg(target_os = "linux")]
    {
        let cgroup_for_pre_exec = try_prepare_cgroup_spawn_plan(&next_cgroup_suffix());
        let (cgroup_opt, handle) = match cgroup_for_pre_exec {
            Some(plan) => (
                Some(plan.cgroup_dir.clone()),
                ContainmentHandle::Linux {
                    cgroup_dir: plan.cgroup_dir,
                    memory_max_bytes: plan.memory_max_bytes,
                },
            ),
            None => (None, ContainmentHandle::Inactive),
        };
        apply_linux_child_pre_exec(cmd, cgroup_opt);
        handle
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = (cmd, stub::inactive_containment);
        ContainmentHandle::Inactive
    }
}

#[allow(clippy::redundant_pub_crate)]
pub(crate) async fn complete_containment_after_spawn(
    pid: Option<u32>,
    handle: ContainmentHandle,
) -> AcpMemoryContainment {
    #[cfg(target_os = "linux")]
    {
        let ContainmentHandle::Linux {
            cgroup_dir,
            memory_max_bytes,
        } = handle
        else {
            return AcpMemoryContainment::inactive();
        };
        let Some(pid) = pid else {
            remove_cgroup_dir(&cgroup_dir);
            return AcpMemoryContainment::inactive();
        };
        let plan = CgroupSpawnPlan {
            cgroup_dir: cgroup_dir.clone(),
            memory_max_bytes,
        };
        if wait_for_cgroup_join(pid, &plan).await {
            return AcpMemoryContainment::from_parts(true, Some(cgroup_dir));
        }
        discard_prepared_cgroup_after_failed_join(pid, &cgroup_dir);
        AcpMemoryContainment::inactive()
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = (pid, handle, stub::inactive_containment);
        AcpMemoryContainment::inactive()
    }
}

pub fn remove_containment_handle(handle: ContainmentHandle) {
    #[cfg(target_os = "linux")]
    {
        let ContainmentHandle::Linux { cgroup_dir, .. } = handle else {
            return;
        };
        remove_cgroup_dir(&cgroup_dir);
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = handle;
    }
}

pub fn emit_containment_unavailable_warn() {
    use crate::output::{MALVIN_WHO, print_stderr_line};
    print_stderr_line(MALVIN_WHO, CONTAINMENT_UNAVAILABLE_WARN);
}

#[must_use]
pub const fn spawn_should_warn_containment_unavailable(containment_active: bool) -> bool {
    !containment_active
}

fn remove_cgroup_dir_at(path: &Path) {
    #[cfg(target_os = "linux")]
    remove_cgroup_dir(path);
    #[cfg(not(target_os = "linux"))]
    {
        let _ = path;
    }
}

fn next_cgroup_suffix() -> String {
    let n = CGROUP_SEQ.fetch_add(1, Ordering::Relaxed);
    format!("{}-{}", std::process::id(), n)
}

#[cfg(target_os = "linux")]
include!("linux_verify_abort.inc");

#[cfg(test)]
#[path = "tests/containment_tests_root.rs"]
pub(crate) mod acp_memory_containment_unit_tests;

#[cfg(test)]
#[allow(dead_code)]
pub mod test_support {
    use super::{
        AcpMemoryContainment, begin_containment_for_command, complete_containment_after_spawn,
    };

    #[cfg(target_os = "linux")]
    #[must_use]
    pub fn writable_cgroups_on_host() -> bool {
        crate::acp_memory_containment::resolve_writable_cgroup_parent().is_some()
    }

    #[cfg(target_os = "linux")]
    pub fn require_cgroup_integration_test() {
        if writable_cgroups_on_host() {
            return;
        }
        crate::output::print_stderr_line(
            crate::output::MALVIN_WHO,
            "SKIP: cgroup integration test requires writable cgroups on this host",
        );
        panic!("cgroup integration test requires writable cgroups on this host");
    }

    /// Synthetic test fixture only (not from a real containment spawn).
    #[must_use]
    pub fn active_with_cgroup_dir(cgroup_dir: std::path::PathBuf) -> AcpMemoryContainment {
        AcpMemoryContainment::from_parts(true, Some(cgroup_dir))
    }

    #[must_use]
    pub async fn active_via_true_child_spawn()
    -> Option<(AcpMemoryContainment, tokio::process::Child)> {
        let mut cmd = tokio::process::Command::new("true");
        let handle = begin_containment_for_command(&mut cmd);
        let mut child = cmd.spawn().ok()?;
        let pid = child.id()?;
        let containment = complete_containment_after_spawn(Some(pid), handle).await;
        if !containment.active() {
            let _ = child.kill().await;
            let _ = child.wait().await;
            return None;
        }
        Some((containment, child))
    }
}
