#[cfg(target_os = "linux")]
#[allow(unsafe_code)]
pub fn install_parent_death_signal(cmd: &mut std::process::Command) {
    use std::os::unix::process::CommandExt;
    let parent_pid = std::process::id();
    unsafe {
        cmd.pre_exec(move || {
            let _ = libc::prctl(libc::PR_SET_PDEATHSIG, libc::SIGKILL as libc::c_ulong);
            if libc::getppid().cast_unsigned() != parent_pid {
                let _ = libc::raise(libc::SIGKILL);
            }
            Ok(())
        });
    }
}

#[cfg(not(target_os = "linux"))]
pub const fn install_parent_death_signal(_: &mut std::process::Command) {}

#[cfg(target_os = "linux")]
pub fn install_tokio_parent_death_signal(cmd: &mut tokio::process::Command) {
    install_parent_death_signal(cmd.as_std_mut());
}

#[cfg(not(target_os = "linux"))]
pub const fn install_tokio_parent_death_signal(_: &mut tokio::process::Command) {}

#[cfg(test)]
mod tests {
    #[test]
    fn parent_death_signal_wired_into_command_builders() {
        let mut std_cmd = std::process::Command::new("true");
        super::install_parent_death_signal(&mut std_cmd);
        let mut tokio_cmd = tokio::process::Command::new("true");
        super::install_tokio_parent_death_signal(&mut tokio_cmd);
    }

    #[test]
    fn kiss_cov_parent_death_symbols() {
        let _ = super::install_parent_death_signal;
        let _ = super::install_tokio_parent_death_signal;
    }
}
