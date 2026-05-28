#![cfg(all(test, unix))]

#[test]
fn parse_u32_field_parses_integers() {
    assert_eq!(super::parse_u32_field(" 42 "), Some(42));
    assert_eq!(super::parse_u32_field("x"), None);
}

#[test]
fn list_proc_rows_includes_current_process() {
    let rows = super::list_proc_rows().expect("proc rows");
    assert!(rows.iter().any(|row| row.pid == std::process::id()));
}

#[test]
fn parse_pid_list_reads_ps_output() {
    let pids = super::parse_pid_list(b"  42\n19531\n");
    assert_eq!(pids.len(), 2);
    assert!(pids.contains(&42));
    assert!(pids.contains(&19_531));
}

#[test]
fn parse_proc_rows_reads_ps_output() {
    let rows = super::parse_proc_rows(b"  42  42    1\n19531 19531 42\n");
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].pid, 42);
    assert_eq!(rows[0].pgid, 42);
    assert_eq!(rows[0].ppid, 1);
}

#[test]
fn list_pids_from_ps_returns_current_process() {
    let pids = super::list_pids_from_ps().expect("ps listing");
    assert!(pids.contains(&std::process::id()));
}

#[test]
fn looks_like_agent_acp_cmdline_matches_malvin_argv() {
    assert!(super::looks_like_agent_acp_cmdline(
        b"agent\0--force\0--model\0auto\0acp\0"
    ));
    assert!(super::looks_like_agent_acp_cmdline(
        b"/home/user/.local/bin/agent\0acp\0"
    ));
    assert!(!super::looks_like_agent_acp_cmdline(b"sleep\x00120\0"));
    assert!(!super::looks_like_agent_acp_cmdline(b"agent\0serve\0"));
}

#[test]
fn is_safe_kill_target_rejects_init_and_self() {
    let protected = super::host_protected_pids(&[]);
    assert!(!super::is_safe_kill_target(super::INIT_PID, &protected));
    assert!(!super::is_safe_kill_target(std::process::id(), &protected));
    assert!(super::is_safe_kill_target(
        std::process::id().saturating_add(1),
        &protected
    ));
}

#[test]
fn process_group_member_pids_includes_self() {
    let me = std::process::id();
    let rows = super::list_proc_rows().expect("proc rows");
    let pgid = rows
        .iter()
        .find(|row| row.pid == me)
        .map(|row| row.pgid)
        .expect("current process row");
    let members = super::process_group_member_pids(pgid);
    assert!(members.contains(&me));
}

#[test]
fn spawned_pids_since_baseline_excludes_baseline_members() {
    let mut baseline = super::snapshot_pids();
    baseline.insert(std::process::id());
    let spawned = super::spawned_pids_since_baseline(&baseline);
    assert!(!spawned.contains(&std::process::id()));
}

#[test]
fn read_proc_cmdline_and_environ_reads_current_process() {
    let me = std::process::id();
    assert!(
        super::read_proc_cmdline(me).is_some_and(|cmdline| !cmdline.is_empty())
    );
    assert!(super::read_proc_environ(me).is_some());
}

#[test]
fn looks_like_malvin_agent_acp_matches_environ_marker() {
    let mut child = std::process::Command::new("sh");
    child.arg("-c").arg("MALVIN_WORKSPACE=/tmp/cov-test exec sleep 30");
    let mut child = child.spawn().expect("spawn");
    let pid = child.id();
    std::thread::sleep(std::time::Duration::from_millis(100));
    assert!(super::looks_like_malvin_agent_acp(pid));
    let _ = child.kill();
    let _ = child.wait();
}

#[test]
fn signal_pid_is_noop_for_invalid_pid() {
    super::signal_pid(999_999_999, 15);
}
