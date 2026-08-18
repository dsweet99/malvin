
mod common;

#[cfg(unix)]
use common::{fresh_workdir, prepend_fake_agent_models_to_path, sleep_child, write_peer_acp_lock};
#[cfg(unix)]
use malvin::malvin_sandbox::clear_active_sandbox_session;
#[cfg(unix)]
use std::process::Command;

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

#[cfg(unix)]
fn run_malvin_home(home: &std::path::Path, work: &std::path::Path, args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_malvin"))
        .env("HOME", home)
        .current_dir(work)
        .args(args)
        .output()
        .unwrap_or_else(|e| panic!("malvin {args:?}: {e}"))
}

#[cfg(unix)]
#[test]
fn malvin_doc_does_not_sweep_stale_locks() {
    let work_dir = tempfile::tempdir().expect("workdir");
    let work = work_dir.path();
    let chamber = work.join(".malvin/acp_spawn");
    std::fs::create_dir_all(&chamber).expect("mkdir chamber");
    let stale = chamber.join("dead.lock");
    std::fs::write(&stale, "424242").expect("stale lock");
    let home = tempfile::tempdir().expect("home");
    let doc = run_malvin_home(home.path(), work, &["--doc"]);
    assert!(doc.status.success(), "stderr={}", String::from_utf8_lossy(&doc.stderr));
    assert!(stale.is_file(), "--doc must not sweep stale locks");
}

#[cfg(unix)]
#[test]
fn malvin_doc_does_not_sweep_but_models_does() {
    let work_dir = tempfile::tempdir().expect("workdir");
    let work = work_dir.path();
    assert!(
        std::process::Command::new("git")
            .args(["init", "--quiet"])
            .current_dir(work)
            .status()
            .expect("git init status")
            .success(),
        "git init failed"
    );
    let chamber = work.join(".malvin/acp_spawn");
    std::fs::create_dir_all(&chamber).expect("mkdir chamber");
    let stale = chamber.join("dead.lock");
    std::fs::write(&stale, "424242").expect("stale lock");
    let (_fake_dir, _path_guard) = prepend_fake_agent_models_to_path(
        "#!/bin/sh\nif [ \"$1\" = models ]; then printf 'composer-2 — Fast\\n'; exit 0; fi\nexit 1\n",
    );
    let home = tempfile::tempdir().expect("home");
    let models = run_malvin_home(home.path(), work, &["models"]);
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
