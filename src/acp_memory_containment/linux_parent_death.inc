pub fn install_parent_death_guard(expected_parent_pid: u32) -> std::io::Result<()> {
    let ret = unsafe { libc::prctl(libc::PR_SET_PDEATHSIG, libc::SIGKILL, 0, 0, 0) };
    if ret != 0 {
        return Err(std::io::Error::last_os_error());
    }
    let ppid = unsafe { libc::getppid() }.cast_unsigned();
    if ppid != expected_parent_pid {
        unsafe { libc::_exit(1) };
    }
    Ok(())
}

#[cfg(test)]
mod linux_parent_death_tests {
    use super::install_parent_death_guard;

    fn wait_for_child_exit(child_pid: libc::pid_t) -> i32 {
        assert!(child_pid > 0, "fork failed");
        let mut status: i32 = 0;
        let waited = unsafe { libc::waitpid(child_pid, &raw mut status, 0) };
        assert_eq!(waited, child_pid);
        assert!(libc::WIFEXITED(status));
        libc::WEXITSTATUS(status)
    }

    #[test]
    fn install_parent_death_guard_runs_in_forked_child() {
        let parent_pid = std::process::id();
        let child_pid = unsafe { libc::fork() };
        if child_pid == 0 {
            install_parent_death_guard(parent_pid).expect("child install");
            unsafe { libc::_exit(0) };
        }
        assert_eq!(wait_for_child_exit(child_pid), 0);
    }

    #[test]
    fn install_parent_death_guard_exits_when_expected_parent_mismatch() {
        let child_pid = unsafe { libc::fork() };
        if child_pid == 0 {
            install_parent_death_guard(0).expect("child install");
            unsafe { libc::_exit(99) };
        }
        assert_eq!(wait_for_child_exit(child_pid), 1);
    }

    #[test]
    fn install_parent_death_guard_exits_when_parent_died_before_ppid_check() {
        let expected_parent = std::process::id();
        let intermediate = unsafe { libc::fork() };
        assert!(intermediate >= 0, "fork failed");
        if intermediate == 0 {
            let intermediate_pid = std::process::id();
            let grandchild = unsafe { libc::fork() };
            if grandchild == 0 {
                for _ in 0..500_000 {
                    if unsafe { libc::getppid() }.cast_unsigned() != intermediate_pid {
                        break;
                    }
                    unsafe {
                        libc::sched_yield();
                    }
                }
                assert_ne!(
                    unsafe { libc::getppid() }.cast_unsigned(),
                    intermediate_pid,
                    "intermediate parent must exit before ppid check"
                );
                install_parent_death_guard(expected_parent).expect("grandchild install");
                unsafe { libc::_exit(99) };
            }
            assert!(grandchild > 0, "grandchild fork failed");
            assert_eq!(
                wait_for_child_exit(grandchild),
                1,
                "must exit when reparented before ppid check"
            );
            unsafe { libc::_exit(0) };
        }
        assert_eq!(wait_for_child_exit(intermediate), 0);
    }
}
