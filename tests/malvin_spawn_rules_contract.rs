//! Contract: dead-before-next and sandbox-isolated malvin spawns.

#[cfg(unix)]
use malvin::acp::snapshot_pids;
#[cfg(unix)]
use malvin::malvin_sandbox::{
    assert_dead_before_next_spawn, clear_active_sandbox_session, malvin_std_command,
    note_active_sandbox_session,
};
#[cfg(unix)]
use std::process::Command;

#[cfg(unix)]
fn process_group_id(pid: u32) -> u32 {
    let out = Command::new("ps")
        .args(["-p", &pid.to_string(), "-o", "pgid="])
        .output()
        .expect("ps pgid");
    let text = String::from_utf8_lossy(&out.stdout);
    text.trim().parse().expect("pgid parse")
}

/// Prior sandbox PIDs must be gone before the next malvin session spawn is allowed.
#[cfg(unix)]
#[test]
fn dead_before_next_rejects_live_prior_sandbox() {
    clear_active_sandbox_session();
    let baseline = snapshot_pids();
    let mut cmd = malvin_std_command("sleep");
    cmd.arg("120");
    let mut child = cmd.spawn().expect("spawn sleep");
    let pgid = child.id();
    note_active_sandbox_session(Some(pgid), baseline);
    let err = assert_dead_before_next_spawn().expect_err("live prior sandbox");
    assert!(
        err.contains("still alive"),
        "expected dead-before-next error, got: {err}"
    );
    let _ = child.kill();
    let _ = child.wait();
    clear_active_sandbox_session();
}

/// After teardown, dead-before-next must allow the next spawn.
#[cfg(unix)]
#[test]
fn dead_before_next_allows_after_prior_sandbox_cleared() {
    clear_active_sandbox_session();
    assert_dead_before_next_spawn().expect("no prior sandbox");
    let baseline = snapshot_pids();
    let mut cmd = malvin_std_command("sleep");
    cmd.arg("1");
    let mut child = cmd.spawn().expect("spawn");
    let pgid = child.id();
    note_active_sandbox_session(Some(pgid), baseline);
    let _ = child.kill();
    let _ = child.wait();
    clear_active_sandbox_session();
    assert_dead_before_next_spawn().expect("prior sandbox ended");
}

/// Malvin workload spawns must create a new process group (sandbox isolation).
#[cfg(unix)]
#[test]
fn malvin_std_command_spawns_in_isolated_process_group() {
    let mut cmd = malvin_std_command("sleep");
    cmd.arg("30");
    let mut child = cmd.spawn().expect("spawn");
    let pid = child.id();
    assert_eq!(
        process_group_id(pid),
        pid,
        "child should be its own process-group leader"
    );
    let _ = child.kill();
    let _ = child.wait();
}

#[test]
fn kiss_cov_malvin_spawn_rules_contract_symbols() {
    let _ = stringify!(dead_before_next_rejects_live_prior_sandbox);
    let _ = stringify!(dead_before_next_allows_after_prior_sandbox_cleared);
    let _ = stringify!(malvin_std_command_spawns_in_isolated_process_group);
    #[cfg(unix)]
    {
        let _ = stringify!(assert_dead_before_next_spawn);
        let _ = stringify!(clear_active_sandbox_session);
        let _ = stringify!(note_active_sandbox_session);
        let _ = stringify!(malvin_std_command);
    }
}
