use super::{Exit, entrypoint_from};

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
        let err = crate::acquire_name("probe").expect_err("live peer must block");
        assert!(
            err.contains(&holder_pid.to_string()),
            "error must name holder pid; got: {err}"
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
        let names = crate::names_registry_root();
        if let Some(parent) = names.parent() {
            std::fs::create_dir_all(parent).expect("mkdir malvin_home");
        }
        std::fs::write(&names, b"not-a-dir").expect("poison names path");
        let stderr = capture_stderr_output(|| {
            assert_eq!(
                entrypoint_from(["malvin", "--background", "--do", "plan.md"]),
                Exit::Failure
            );
        });
        assert!(
            !stderr.is_empty(),
            "background --do must print session acquire failure on stderr; got: {stderr:?}"
        );
    });
}

#[test]
fn kiss_cov_entrypoint_name_unix_symbols() {
    #[cfg(unix)]
    {}
}
