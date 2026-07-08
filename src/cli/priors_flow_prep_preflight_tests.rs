use super::priors_preflight;

#[test]
fn priors_proceeds_when_out_path_missing() {
    crate::test_utils::with_isolated_home(|work| {
        let cwd = std::env::current_dir().expect("cwd");
        std::env::set_current_dir(work).expect("chdir");
        let (text, out, _work_dir) =
            priors_preflight("ground this", "priors.md").expect("ok");
        assert_eq!(text, "ground this");
        assert!(out.ends_with("priors.md"));
        std::env::set_current_dir(cwd).expect("restore");
    });
}

#[test]
fn priors_default_allocates_sibling_when_priors_md_exists() {
    crate::test_utils::with_isolated_home(|work| {
        let cwd = std::env::current_dir().expect("cwd");
        std::env::set_current_dir(work).expect("chdir");
        std::fs::write(work.join("priors.md"), "existing\n").expect("write");
        let (_text, out, _) = priors_preflight("req", "priors.md").expect("ok");
        assert!(
            out.ends_with("priors_1.md"),
            "expected priors_1.md, got {}",
            out.display()
        );
        std::env::set_current_dir(cwd).expect("restore");
    });
}

#[test]
fn priors_custom_out_path_refuses_overwrite() {
    crate::test_utils::with_isolated_home(|work| {
        let cwd = std::env::current_dir().expect("cwd");
        std::env::set_current_dir(work).expect("chdir");
        std::fs::create_dir_all(work.join("reports")).expect("mkdir");
        std::fs::write(work.join("reports/existing.md"), "x\n").expect("write");
        let err = priors_preflight("req", "reports/existing.md").expect_err("exists");
        assert!(err.contains("refusing to overwrite"));
        std::env::set_current_dir(cwd).expect("restore");
    });
}

#[test]
fn priors_preflight_resolves_md_request_file() {
    crate::test_utils::with_isolated_home(|work| {
        let cwd = std::env::current_dir().expect("cwd");
        std::env::set_current_dir(work).expect("chdir");
        std::fs::write(work.join("brief.md"), "brief body\n").expect("write");
        let (text, out, _) = priors_preflight("brief.md", "priors.md").expect("ok");
        assert_eq!(text, "brief body\n");
        assert!(out.ends_with("priors.md"));
        std::env::set_current_dir(cwd).expect("restore");
    });
}
