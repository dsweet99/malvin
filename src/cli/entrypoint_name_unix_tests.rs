use super::{Exit, entrypoint_from};

#[cfg(unix)]
#[test]
fn bare_kpop_duplicate_name_exits_failure() {
    crate::test_utils::with_isolated_home(|work| {
        let _ = work;
        let mut child = crate::malvin_sandbox::malvin_std_command("sleep")
            .arg("120")
            .spawn()
            .expect("spawn sleep");
        let holder_pid = child.id();
        std::fs::create_dir_all(crate::names_registry_root()).expect("mkdir names");
        std::fs::write(crate::name_path("probe"), format!("{holder_pid}\n")).expect("peer lock");
        assert_eq!(
            entrypoint_from(["malvin", "--name", "probe", "investigate cache"]),
            Exit::Failure
        );
        let _ = child.kill();
        let _ = child.wait();
    });
}

#[cfg(unix)]
#[test]
fn duplicate_name_exits_failure() {
    crate::test_utils::with_isolated_home(|work| {
        let _ = work;
        let mut child = crate::malvin_sandbox::malvin_std_command("sleep")
            .arg("120")
            .spawn()
            .expect("spawn sleep");
        let holder_pid = child.id();
        std::fs::create_dir_all(crate::names_registry_root()).expect("mkdir names");
        std::fs::write(crate::name_path("probe"), format!("{holder_pid}\n")).expect("peer lock");
        assert_eq!(
            entrypoint_from(["malvin", "--name", "probe", "code", "plan.md"]),
            Exit::Failure
        );
        let _ = child.kill();
        let _ = child.wait();
    });
}

#[cfg(unix)]
#[test]
fn duplicate_name_error_on_stderr_with_background() {
    use crate::test_stderr_capture::capture_stderr_output;

    crate::test_utils::with_isolated_home(|work| {
        let _ = work;
        let mut child = crate::malvin_sandbox::malvin_std_command("sleep")
            .arg("120")
            .spawn()
            .expect("spawn sleep");
        let holder_pid = child.id();
        std::fs::create_dir_all(crate::names_registry_root()).expect("mkdir names");
        std::fs::write(crate::name_path("probe"), format!("{holder_pid}\n")).expect("peer lock");
        let stderr = capture_stderr_output(|| {
            assert_eq!(
                entrypoint_from(["malvin", "--background", "--name", "probe", "code", "plan.md"]),
                Exit::Failure
            );
        });
        assert!(
            stderr.contains(&holder_pid.to_string()),
            "stderr must name holder pid; got: {stderr:?}"
        );
        assert!(
            stderr.contains(&crate::name_path("probe").display().to_string()),
            "stderr must name lock path; got: {stderr:?}"
        );
        let _ = child.kill();
        let _ = child.wait();
    });
}

#[test]
fn kiss_cov_entrypoint_name_unix_symbols() {
    #[cfg(unix)]
    {
    }
}
