#[cfg(target_os = "linux")]
#[allow(unsafe_code)]
mod linux {
    use crate::acp_memory_containment::{
        CONTAINMENT_UNAVAILABLE_WARN, emit_containment_unavailable_warn,
    };

    #[test]
    fn emit_containment_unavailable_warn_must_write_plan_line_to_stderr() {
        use std::io::Read;
        use std::os::fd::FromRawFd;

        let mut fds = [-1i32; 2];
        assert_eq!(unsafe { libc::pipe(fds.as_mut_ptr()) }, 0);
        let read_fd = fds[0];
        let write_fd = fds[1];
        let saved_stderr = unsafe { libc::dup(libc::STDERR_FILENO) };
        assert!(saved_stderr >= 0, "dup stderr");
        assert_eq!(
            unsafe { libc::dup2(write_fd, libc::STDERR_FILENO) },
            libc::STDERR_FILENO
        );
        unsafe {
            libc::close(write_fd);
        }

        emit_containment_unavailable_warn();

        unsafe {
            libc::dup2(saved_stderr, libc::STDERR_FILENO);
            libc::close(saved_stderr);
        }

        let mut captured = String::new();
        unsafe {
            std::fs::File::from_raw_fd(read_fd)
                .read_to_string(&mut captured)
                .expect("read captured stderr");
        }
        assert!(
            captured.contains(CONTAINMENT_UNAVAILABLE_WARN),
            "verify-failure path must write plan warning to stderr, not tracing only"
        );
    }
}
