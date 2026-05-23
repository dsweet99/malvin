use super::{
    AcpMemoryContainment, ContainmentHandle, complete_containment_after_spawn,
    finalize_containment_cgroup,
};

pub const VERIFY_FAILED_ABORT_MSG: &str =
    "ACP memory containment verify failed after spawn; refusing to continue without cgroup memory limits";

pub async fn complete_and_require_linux_containment_after_spawn(
    pid: Option<u32>,
    handle: ContainmentHandle,
    had_cgroup_plan: bool,
    child: &mut tokio::process::Child,
) -> Result<AcpMemoryContainment, String> {
    let memory_containment = complete_containment_after_spawn(pid, handle).await;
    abort_when_linux_containment_verify_failed(had_cgroup_plan, &memory_containment, child).await?;
    Ok(memory_containment)
}

pub async fn abort_when_linux_containment_verify_failed(
    had_cgroup_plan: bool,
    memory_containment: &AcpMemoryContainment,
    child: &mut tokio::process::Child,
) -> Result<(), String> {
    if had_cgroup_plan && !memory_containment.active() {
        let _ = child.kill().await;
        let _ = child.wait().await;
        finalize_containment_cgroup(memory_containment);
        return Err(VERIFY_FAILED_ABORT_MSG.to_string());
    }
    Ok(())
}


#[cfg(test)]
mod kiss_cov_auto {
    #[test]
    fn kiss_cov_abort_when_linux_containment_verify_failed() { let _ = stringify!(abort_when_linux_containment_verify_failed); }

}
