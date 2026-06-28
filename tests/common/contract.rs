use std::path::PathBuf;

#[cfg(unix)]
pub fn fresh_workdir(name: &str) -> PathBuf {
    let work = std::env::temp_dir().join(name);
    let _ = std::fs::remove_dir_all(&work);
    std::fs::create_dir_all(&work).expect("mkdir work");
    assert!(
        std::process::Command::new("git")
            .args(["init"])
            .current_dir(&work)
            .status()
            .expect("git init status")
            .success(),
        "git init failed for {}",
        work.display()
    );
    work
}

#[cfg(unix)]
pub fn sleep_child(seconds: &str) -> std::process::Child {
    let mut cmd = malvin::malvin_sandbox::malvin_std_command("sleep");
    cmd.arg(seconds);
    cmd.spawn().expect("spawn sleep")
}

#[cfg(unix)]
pub fn write_peer_acp_lock(
    work: &std::path::Path,
    slot: &str,
    holder_pid: u32,
) -> std::path::PathBuf {
    std::fs::create_dir_all(malvin::malvin_acp_spawn_chamber_dir(work)).expect("mkdir acp_spawn");
    let lock = malvin::malvin_acp_spawn_chamber_dir(work).join(format!("{slot}.lock"));
    std::fs::write(&lock, holder_pid.to_string()).expect("write peer lock");
    lock
}
