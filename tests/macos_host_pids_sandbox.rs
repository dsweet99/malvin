#![cfg(target_os = "macos")]

use std::collections::HashSet;

use malvin::agent_process::process_group_rss::pids_sandbox_bytes;

#[test]
fn macos_host_pids_sandbox_bytes_positive() {
    let mut pids = HashSet::new();
    pids.insert(std::process::id());
    let bytes = pids_sandbox_bytes(&pids).expect("macos sandbox");
    assert!(bytes > 0);
}

#[tokio::test]
async fn macos_agent_launch_uses_isolated_process_group_without_cgroups() {
    use malvin::agent_process::agent_scope::{agent_cgroups_enabled, malvin_agent_tokio_command};

    assert!(!agent_cgroups_enabled());
    let mut cmd = malvin_agent_tokio_command("sh");
    assert_eq!(cmd.as_std().get_program(), "sh");
    cmd.args(["-c", "ps -o pgid= -p $$"])
        .stdout(std::process::Stdio::piped());
    let child = cmd.spawn().expect("spawn fallback command");
    let pid = child.id().expect("child pid");
    let output = child.wait_with_output().await.expect("wait for fallback");
    assert!(output.status.success());
    let pgid = String::from_utf8(output.stdout).expect("pgid output");
    assert_eq!(pgid.trim().parse::<u32>().expect("numeric pgid"), pid);
}
