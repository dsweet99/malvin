#[test]
fn quality_gate_commands_do_not_reference_acp_containment() {
    let root = std::env::current_dir().expect("cwd");
    let commands = crate::repo_gates::gate_command_lines(&root).expect("gate lines");
    for line in commands {
        assert!(!line.contains("begin_containment_for_command"));
        assert!(!line.contains("acp_memory_containment"));
    }
}

#[cfg(target_os = "linux")]
mod linux {
    use crate::acp_memory_containment::{
        begin_containment_for_command, complete_containment_after_spawn,
        finalize_containment_cgroup,
    };

    #[tokio::test]
    async fn linux_spawn_wires_begin_and_complete_containment() {
        let mut cmd = tokio::process::Command::new("true");
        let handle = begin_containment_for_command(&mut cmd);
        let mut child = cmd.spawn().expect("spawn true");
        let containment = complete_containment_after_spawn(child.id(), handle).await;
        finalize_containment_cgroup(&containment);
        let _ = child.wait().await;
    }
}

#[cfg(not(target_os = "linux"))]
mod non_linux {
    #[test]
    fn non_linux_stub_reports_no_half_memory_budget() {
        assert!(crate::acp_memory_containment::half_physical_memory_bytes().is_none());
    }
}
