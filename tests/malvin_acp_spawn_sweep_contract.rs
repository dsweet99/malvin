//! Contract: directory-wide stale ACP spawn lock garbage collection.

mod common;

#[cfg(unix)]
use common::{fresh_workdir, prepend_fake_agent_models_to_path, sleep_child, write_peer_acp_lock};
#[cfg(unix)]
use malvin::malvin_sandbox::clear_active_sandbox_session;
#[cfg(unix)]
use std::process::Command;

/// Directory sweep removes dead-PID and invalid-content locks; keeps live-PID lock.
#[cfg(unix)]
#[test]
fn sweep_stale_acp_spawn_locks_contract() {
    clear_active_sandbox_session();
    let work = fresh_workdir("malvin_sweep_stale_contract");
    let chamber = work.join(".malvin/acp_spawn");
    std::fs::create_dir_all(&chamber).expect("mkdir chamber");
    let mut child = sleep_child("120");
    write_peer_acp_lock(&work, "peer", child.id());
    std::fs::write(chamber.join("dead.lock"), "424242").expect("dead lock");
    std::fs::write(chamber.join("bad.lock"), "not-a-pid").expect("invalid lock");
    let removed = malvin::sweep_stale_acp_spawn_locks(&work).expect("sweep");
    assert_eq!(removed, 2);
    assert!(chamber.join("peer.lock").exists(), "live peer lock kept");
    assert!(!chamber.join("dead.lock").exists());
    assert!(!chamber.join("bad.lock").exists());
    let _ = child.kill();
    let _ = child.wait();
}

/// `--doc` exits before entrypoint sweep; `models` runs the sweep (plan Phase 3).
#[cfg(unix)]
#[test]
fn malvin_doc_does_not_sweep_but_models_does() {
    let work = fresh_workdir("malvin_doc_vs_models_sweep");
    let chamber = work.join(".malvin/acp_spawn");
    std::fs::create_dir_all(&chamber).expect("mkdir chamber");
    let stale = chamber.join("dead.lock");
    std::fs::write(&stale, "424242").expect("stale lock");
    let (_fake_dir, _path_guard) = prepend_fake_agent_models_to_path(
        "#!/bin/sh\nif [ \"$1\" = models ]; then printf 'composer-2 — Fast\\n'; exit 0; fi\nexit 1\n",
    );
    let bin = env!("CARGO_BIN_EXE_malvin");
    let doc = Command::new(bin)
        .current_dir(&work)
        .args(["--doc"])
        .output()
        .expect("malvin --doc");
    assert!(doc.status.success(), "stderr={}", String::from_utf8_lossy(&doc.stderr));
    assert!(stale.is_file(), "--doc must not sweep stale locks");
    let models = Command::new(bin)
        .current_dir(&work)
        .arg("models")
        .output()
        .expect("malvin models");
    assert!(
        models.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&models.stderr)
    );
    assert!(!stale.is_file(), "models must sweep stale locks");
}

#[test]
fn kiss_cov_malvin_acp_spawn_sweep_contract_symbols() {
    #[cfg(unix)]
    {
        let _ = stringify!(sweep_stale_acp_spawn_locks_contract);
        let _ = stringify!(malvin_doc_does_not_sweep_but_models_does);
    }
}
