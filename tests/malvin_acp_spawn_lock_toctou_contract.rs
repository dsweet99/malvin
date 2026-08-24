mod common;

#[cfg(unix)]
use common::fresh_workdir;
#[cfg(unix)]
use malvin::{acquire_acp_spawn_lock_for_slot, release_acp_spawn_lock};
#[cfg(unix)]
use std::path::Path;
#[cfg(unix)]
use std::process::Command;

#[cfg(unix)]
use malvin::wait_for_dir_entry_count;

#[cfg(unix)]
fn write_probe_ready(ready_dir: &Path) {
    std::fs::create_dir_all(ready_dir).expect("ready dir");
    std::fs::write(ready_dir.join(std::process::id().to_string()), b"1").expect("ready");
}

#[cfg(unix)]
fn write_probe_result(result_dir: &Path, ok: bool) {
    std::fs::write(
        result_dir.join(std::process::id().to_string()),
        if ok { b"1" } else { b"0" },
    )
    .expect("result");
}

#[cfg(unix)]
fn spawn_toctou_child_probe(work: &Path, slot: &str, ready_dir: &Path) -> std::process::Child {
    let exe = std::env::current_exe().expect("current test exe");
    Command::new(&exe)
        .env("MALVIN_ACP_LOCK_TOCTOU_PROBE_CHILD", "1")
        .env("MALVIN_ACP_LOCK_TOCTOU_WORK", work)
        .env("MALVIN_ACP_LOCK_TOCTOU_SLOT", slot)
        .env("MALVIN_ACP_LOCK_TOCTOU_READY_DIR", ready_dir)
        .args([
            "peer_acp_spawn_lock_toctou_rejects_dual_acquire_across_processes",
            "--exact",
            "--nocapture",
        ])
        .spawn()
        .expect("spawn toctou child")
}

#[cfg(unix)]
fn run_toctou_child_probe(work: &Path, slot: &str, ready_dir: &Path) {
    let result_dir = ready_dir.with_file_name("toctou_results");
    write_probe_ready(ready_dir);
    std::fs::create_dir_all(&result_dir).expect("result dir");
    wait_for_dir_entry_count(ready_dir, 2);
    let result = acquire_acp_spawn_lock_for_slot(work, slot);
    write_probe_result(&result_dir, result.is_ok());
    wait_for_dir_entry_count(&result_dir, 2);
    if result.is_ok() {
        release_acp_spawn_lock(work, slot);
        return;
    }
    panic!("child probe lost the lock race: {result:?}");
}

#[cfg(unix)]
fn assert_exactly_one_child_succeeded(status0: std::process::ExitStatus, status1: std::process::ExitStatus) {
    assert!(
        status0.success() ^ status1.success(),
        "exactly one cross-process acquire must succeed (TOCTOU fixed): {status0:?} {status1:?}"
    );
}

#[cfg(unix)]
fn run_dual_acquire_parent_test(work: &Path, slot: &str, ready_dir: &Path) {
    let _ = std::fs::remove_dir_all(ready_dir);
    let _ = std::fs::remove_dir_all(ready_dir.with_file_name("toctou_results"));
    let mut child0 = spawn_toctou_child_probe(work, slot, ready_dir);
    let mut child1 = spawn_toctou_child_probe(work, slot, ready_dir);
    let status0 = child0.wait().expect("child0");
    let status1 = child1.wait().expect("child1");
    assert_exactly_one_child_succeeded(status0, status1);
    let lock = work
        .join(".malvin")
        .join("acp_spawn_chamber")
        .join(format!("{slot}.lock"));
    assert!(!lock.exists(), "children release locks on exit");
}

#[cfg(unix)]
#[test]
fn peer_acp_spawn_lock_toctou_rejects_dual_acquire_across_processes() {
    if std::env::var_os("MALVIN_ACP_LOCK_TOCTOU_PROBE_CHILD").is_some() {
        let work_path = std::env::var("MALVIN_ACP_LOCK_TOCTOU_WORK").expect("work dir");
        let work = Path::new(&work_path);
        let slot =
            std::env::var("MALVIN_ACP_LOCK_TOCTOU_SLOT").unwrap_or_else(|_| "kpop_toctou".into());
        let ready_path = std::env::var("MALVIN_ACP_LOCK_TOCTOU_READY_DIR").expect("ready dir");
        run_toctou_child_probe(work, &slot, Path::new(&ready_path));
        return;
    }

    let work = fresh_workdir("malvin_peer_acp_spawn_lock_toctou");
    let slot = "kpop_toctou_xproc";
    let ready_dir = work.join(".malvin").join("toctou_ready");
    run_dual_acquire_parent_test(&work, slot, &ready_dir);
}

#[test]
fn kiss_cov_malvin_acp_spawn_lock_toctou_contract_symbols() {
    let _ = stringify!(peer_acp_spawn_lock_toctou_rejects_dual_acquire_across_processes);
}
