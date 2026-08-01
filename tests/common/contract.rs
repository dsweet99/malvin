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
pub struct FakePathGuard {
    old_path: Option<String>,
}

#[cfg(unix)]
impl Drop for FakePathGuard {
    fn drop(&mut self) {
        #[allow(unsafe_code)]
        unsafe {
            match &self.old_path {
                Some(path) => std::env::set_var("PATH", path),
                None => std::env::remove_var("PATH"),
            }
        }
    }
}

#[cfg(unix)]
pub fn prepend_fake_agent_models_to_path(body: &str) -> (tempfile::TempDir, FakePathGuard) {
    use std::os::unix::fs::PermissionsExt;

    let dir = tempfile::tempdir().expect("fake agent dir");
    let agent = dir.path().join("agent");
    std::fs::write(&agent, body).expect("write fake agent");
    let mut perms = std::fs::metadata(&agent).expect("metadata").permissions();
    perms.set_mode(0o755);
    std::fs::set_permissions(&agent, perms).expect("chmod fake agent");
    let old_path = std::env::var("PATH").ok();
    let new_path = format!(
        "{}:{}",
        dir.path().display(),
        old_path.as_deref().unwrap_or("")
    );
    #[allow(unsafe_code)]
    unsafe {
        std::env::set_var("PATH", new_path);
    }
    (
        dir,
        FakePathGuard { old_path },
    )
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
