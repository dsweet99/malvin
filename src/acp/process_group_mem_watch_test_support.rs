use std::collections::HashSet;
use std::os::unix::process::CommandExt;

pub fn spawn_std_sleep_child_in_new_process_group() -> (std::process::Child, u32, HashSet<u32>) {
    crate::test_utils::enable_test_fast_teardown();
    let baseline = crate::acp::snapshot_pids();
    let mut cmd = std::process::Command::new("sleep");
    unsafe {
        cmd.arg("30").pre_exec(|| {
            if libc::setpgid(0, 0) != 0 {
                return Err(std::io::Error::last_os_error());
            }
            Ok(())
        });
    }
    let child = cmd.spawn().expect("spawn sleep");
    let pgid = child.id();
    (child, pgid, baseline)
}
