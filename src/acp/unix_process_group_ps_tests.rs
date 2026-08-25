#![cfg(all(test, unix))]

fn parse_u32_field_parses_integers() {
    assert_eq!(super::parse_u32_field(" 42 "), Some(42));
    assert_eq!(super::parse_u32_field("x"), None);
}

fn list_proc_rows_includes_current_process() {
    let rows = super::list_proc_rows().expect("proc rows");
    assert!(rows.iter().any(|row| row.pid == std::process::id()));
}

fn proc_snapshots_without_self_are_rejected() {
    assert!(!super::proc_pid_snapshot_is_usable(
        &std::collections::HashSet::new()
    ));
    assert!(!super::proc_row_snapshot_is_usable(&[]));
}

fn proc_snapshots_with_self_are_accepted() {
    let me = std::process::id();
    assert!(super::proc_pid_snapshot_is_usable(
        &std::iter::once(me).collect()
    ));
    assert!(super::proc_row_snapshot_is_usable(&[super::ProcRow {
        pid: me,
        pgid: me,
        ppid: 1,
    }]));
}

#[cfg(target_os = "linux")]
fn list_proc_rows_matches_ps_for_self_via_proc_path() {
    let me = std::process::id();
    let via_public = super::list_proc_rows().expect("list_proc_rows");
    let public_row = via_public
        .iter()
        .find(|row| row.pid == me)
        .expect("current pid via list_proc_rows");

    let out = std::process::Command::new("ps")
        .args([
            "-p",
            &me.to_string(),
            "-o",
            "pid=",
            "-o",
            "pgid=",
            "-o",
            "ppid=",
        ])
        .stderr(std::process::Stdio::null())
        .output()
        .expect("ps");
    let from_ps = super::parse_proc_rows(&out.stdout);
    let ps_row = from_ps
        .iter()
        .find(|row| row.pid == me)
        .expect("current pid in ps");

    assert_eq!(public_row.pgid, ps_row.pgid);
    assert_eq!(public_row.ppid, ps_row.ppid);
    assert!(
        std::path::Path::new("/proc").is_dir(),
        "Linux production path should prefer /proc when present"
    );
}

fn parse_pid_list_reads_ps_output() {
    let pids = super::parse_pid_list(b"  42\n19531\n");
    assert_eq!(pids.len(), 2);
    assert!(pids.contains(&42));
    assert!(pids.contains(&19_531));
}

fn parse_proc_rows_reads_ps_output() {
    let rows = super::parse_proc_rows(b"  42  42    1\n19531 19531 42\n");
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].pid, 42);
    assert_eq!(rows[0].pgid, 42);
    assert_eq!(rows[0].ppid, 1);
}

fn list_pids_from_ps_returns_current_process() {
    let pids = super::list_pids_from_ps().expect("ps listing");
    assert!(pids.contains(&std::process::id()));
}

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

fn is_safe_kill_target_rejects_init_and_self() {
    let protected = super::host_protected_pids(&[]);
    assert!(!super::is_safe_kill_target(super::INIT_PID, &protected));
    assert!(!super::is_safe_kill_target(std::process::id(), &protected));
    assert!(super::is_safe_kill_target(
        std::process::id().saturating_add(1),
        &protected
    ));
}

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

fn spawned_pids_since_baseline_excludes_baseline_members() {
    let mut baseline = super::snapshot_pids();
    baseline.insert(std::process::id());
    let spawned = super::spawned_pids_since_baseline(&baseline);
    assert!(!spawned.contains(&std::process::id()));
}

#[cfg(target_os = "linux")]
fn read_proc_cmdline_and_environ_reads_current_process() {
    let me = std::process::id();
    assert!(super::read_proc_cmdline(me).is_some_and(|cmdline| !cmdline.is_empty()));
    assert!(super::read_proc_environ(me).is_some());
}

#[cfg(target_os = "linux")]
fn looks_like_malvin_agent_acp_ignores_inherited_malvin_workspace_on_sleep() {
    let mut child = std::process::Command::new("sh");
    child
        .arg("-c")
        .arg("MALVIN_WORKSPACE=/tmp/cov-test exec sleep 30");
    let mut child = child.spawn().expect("spawn");
    let pid = child.id();
    std::thread::sleep(std::time::Duration::from_millis(100));
    assert!(
        !super::looks_like_malvin_agent_acp(pid),
        "inherited MALVIN_WORKSPACE on sleep must not identify agent acp"
    );
    let _ = child.kill();
    let _ = child.wait();
}

fn signal_pid_is_noop_for_invalid_pid() {
    super::signal_pid(999_999_999, 15);
}

fn pid_alive_reports_self_alive_and_reaped_child_dead() {
    assert!(
        super::pid_alive(std::process::id()),
        "current process must be alive via kill(2) signal 0"
    );

    let mut child = std::process::Command::new("true")
        .spawn()
        .expect("spawn short-lived child");
    let child_pid = child.id();
    let status = child.wait().expect("wait child");
    assert!(status.success(), "child `true` must exit 0");
    assert!(
        !super::pid_alive(child_pid),
        "reaped child pid must be dead; kill(2) must not depend on a `kill` binary on PATH"
    );
}

fn signal_pid_kill_round_trip_without_kill_binary() {
    // Direct libc::kill must stop a child; the old path forked `/bin/kill` for this.
    let mut child = std::process::Command::new("sleep")
        .arg("30")
        .spawn()
        .expect("spawn sleep");
    let child_pid = child.id();
    assert!(super::pid_alive(child_pid), "sleep child must start alive");
    super::signal_pid(child_pid, 9); // SIGKILL
    let status = child.wait().expect("wait signaled child");
    assert!(
        !status.success(),
        "SIGKILL must yield non-success wait status"
    );
    assert!(
        !super::pid_alive(child_pid),
        "after SIGKILL + wait, pid must be dead via kill(2)"
    );
}

#[test]
fn kiss_bundled_acp_unix_process_group_ps_tests() {
    parse_u32_field_parses_integers();
    list_proc_rows_includes_current_process();
    proc_snapshots_without_self_are_rejected();
    proc_snapshots_with_self_are_accepted();
    list_proc_rows_matches_ps_for_self_via_proc_path();
    parse_pid_list_reads_ps_output();
    parse_proc_rows_reads_ps_output();
    list_pids_from_ps_returns_current_process();
    looks_like_agent_acp_cmdline_matches_malvin_argv();
    is_safe_kill_target_rejects_init_and_self();
    process_group_member_pids_includes_self();
    spawned_pids_since_baseline_excludes_baseline_members();
    read_proc_cmdline_and_environ_reads_current_process();
    looks_like_malvin_agent_acp_ignores_inherited_malvin_workspace_on_sleep();
    signal_pid_is_noop_for_invalid_pid();
    pid_alive_reports_self_alive_and_reaped_child_dead();
    signal_pid_kill_round_trip_without_kill_binary();
}
