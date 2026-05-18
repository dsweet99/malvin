#[cfg(target_os = "linux")]
use crate::acp_memory_containment::{
    CgroupSpawnPlan, apply_linux_child_pre_exec, try_prepare_cgroup_spawn_plan,
    wait_for_cgroup_join,
};

#[cfg(target_os = "linux")]
pub async fn spawn_sleep_in_prepared_cgroup(
    suffix: &str,
) -> Option<(
    tokio::process::Child,
    u32,
    std::path::PathBuf,
    CgroupSpawnPlan,
)> {
    use std::process::Stdio;
    use tokio::process::Command;

    let plan = try_prepare_cgroup_spawn_plan(suffix)?;
    let cgroup_dir = plan.cgroup_dir.clone();
    let mut cmd = Command::new("sleep");
    cmd.arg("120");
    cmd.stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    apply_linux_child_pre_exec(&mut cmd, Some(cgroup_dir.clone()));
    let mut child = cmd.spawn().ok()?;
    let pid = child.id()?;
    if !wait_for_cgroup_join(pid, &plan).await {
        let _ = child.kill().await;
        let _ = child.wait().await;
        return None;
    }
    Some((child, pid, cgroup_dir, plan))
}

#[cfg(target_os = "linux")]
pub fn child_still_running(child: &mut tokio::process::Child) -> bool {
    matches!(child.try_wait(), Ok(None))
}
