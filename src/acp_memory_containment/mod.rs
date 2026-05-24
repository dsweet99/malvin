//! Linux cgroup v2 (and v1 memory fallback) memory limits for `agent acp` children only.
#![allow(unsafe_code)]

use std::path::Path;
#[cfg(target_os = "linux")]
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

#[cfg(target_os = "linux")]
#[path = "cgroup_memory.rs"]
mod cgroup_memory;
#[cfg(any(test, target_os = "linux"))]
#[path = "cgroup_line.rs"]
mod cgroup_line;
#[cfg(target_os = "linux")]
#[path = "linux_fs.rs"]
mod linux_fs;
#[cfg(target_os = "linux")]
#[path = "linux_parent_death.rs"]
mod linux_parent_death;
#[cfg(target_os = "linux")]
#[path = "linux_spawn.rs"]
mod linux_spawn;
#[cfg(target_os = "linux")]
pub(crate) use cgroup_memory::*;
#[cfg(target_os = "linux")]
pub(crate) use linux_fs::*;
#[cfg(target_os = "linux")]
pub(crate) use linux_parent_death::*;
#[cfg(target_os = "linux")]
pub(crate) use linux_spawn::*;

#[cfg(not(target_os = "linux"))]
mod stub;
#[cfg(not(target_os = "linux"))]
pub(crate) use stub::{
    inactive_platform_memory_limit_exceeded_since_baseline as memory_limit_exceeded_since_baseline,
    inactive_platform_memory_limit_oom_baseline_at as memory_limit_oom_baseline_at,
};

pub const AGENT_EXCEEDED_MEMORY_LIMIT_MSG: &str = "agent exceeded memory limit";
pub const CONTAINMENT_UNAVAILABLE_WARN: &str =
    "ACP memory containment unavailable; running agent without cgroup memory limit";

#[cfg_attr(not(target_os = "linux"), allow(dead_code))]
static CGROUP_SEQ: AtomicU64 = AtomicU64::new(0);

mod containment_state;

pub use containment_state::AcpMemoryContainment;
#[allow(unused_imports)]
pub use containment_state::OomBaseline;
pub use containment_state::finalize_containment_cgroup;

#[cfg(not(target_os = "linux"))]
#[must_use]
#[allow(dead_code)]
pub const fn half_physical_memory_bytes() -> Option<u64> {
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

#[cfg_attr(not(target_os = "linux"), allow(clippy::missing_const_for_fn))]
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

#[cfg_attr(not(target_os = "linux"), allow(clippy::missing_const_for_fn))]
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
    crate::output::print_log_warning(CONTAINMENT_UNAVAILABLE_WARN);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ContainmentUnavailableWarnAtSpawn {
    pub log_full_outgoing_prompts: bool,
    pub containment_active: bool,
}

pub fn emit_containment_unavailable_warn_after_spawn(ctx: ContainmentUnavailableWarnAtSpawn) {
    if ctx.log_full_outgoing_prompts
        && spawn_should_warn_containment_unavailable(ctx.containment_active)
    {
        emit_containment_unavailable_warn();
    }
}

#[must_use]
pub const fn spawn_should_warn_containment_unavailable(containment_active: bool) -> bool {
    !containment_active
}

#[cfg_attr(not(target_os = "linux"), allow(clippy::missing_const_for_fn))]
fn remove_cgroup_dir_at(path: &Path) {
    #[cfg(target_os = "linux")]
    remove_cgroup_dir(path);
    #[cfg(not(target_os = "linux"))]
    {
        let _ = path;
    }
}

#[cfg_attr(not(target_os = "linux"), allow(dead_code))]
fn next_cgroup_suffix() -> String {
    let n = CGROUP_SEQ.fetch_add(1, Ordering::Relaxed);
    format!("{}-{}", std::process::id(), n)
}

#[cfg(target_os = "linux")]
#[path = "linux_verify_abort.rs"]
mod linux_verify_abort;
#[cfg(target_os = "linux")]
pub(crate) use linux_verify_abort::*;

#[cfg(test)]
#[path = "tests/containment_tests_root.rs"]
pub(crate) mod acp_memory_containment_unit_tests;

#[cfg(test)]
#[path = "test_support.rs"]
pub mod test_support;

#[cfg(test)]
mod kiss_cov_auto {
    use super::{half_physical_memory_bytes, spawn_should_warn_containment_unavailable};

    #[test]
    fn kiss_cov_src_acp_memory_containment_mod_rs_half_physical_memory_bytes() {
        #[cfg(target_os = "linux")]
        {
            let value = half_physical_memory_bytes();
            assert!(value.is_some());
            assert!(value.unwrap() > 0);
        }
        #[cfg(not(target_os = "linux"))]
        assert_eq!(half_physical_memory_bytes(), None);
    }

    #[test]
    fn kiss_cov_spawn_should_warn_containment_unavailable() {
        assert!(spawn_should_warn_containment_unavailable(false));
        assert!(!spawn_should_warn_containment_unavailable(true));
    }
}
