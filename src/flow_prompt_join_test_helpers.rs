
use crate::artifacts::RunArtifacts;

pub fn flow_test_artifacts_no_checks(tmp: &tempfile::TempDir) -> RunArtifacts {
    let plan = tmp.path().join("plan.md");
    std::fs::write(&plan, "ignored").expect("plan");
    let run_dir = tmp.path().join(".malvin/logs").join("r");
    std::fs::create_dir_all(&run_dir).expect("run");
    RunArtifacts {
        run_dir,
        plan_path: plan,
        work_dir: tmp.path().to_path_buf(),
    }
}

pub fn flow_test_artifacts(tmp: &tempfile::TempDir) -> RunArtifacts {
    std::process::Command::new("git")
        .args(["init"])
        .current_dir(tmp.path())
        .status()
        .expect("git init");
    crate::seed_malvin_checks(tmp.path(), "true\n");
    let plan = tmp.path().join("plan.md");
    std::fs::write(&plan, "ignored").expect("plan");
    let run_dir = tmp.path().join(".malvin/logs").join("r");
    std::fs::create_dir_all(&run_dir).expect("run");
    RunArtifacts {
        run_dir,
        plan_path: plan,
        work_dir: tmp.path().to_path_buf(),
    }
}

pub fn assert_header_user_join(combined: &str, header: &str, user: &str) {
    assert_eq!(combined, format!("{header}\n\n{user}"));
    assert_eq!(combined.split("\n\n").count(), 2);
    assert_eq!(combined.matches(header).count(), 1);
    assert_eq!(combined.matches(user).count(), 1);
}

pub fn assert_dual_workflow_header_join(
    combined: &str,
    coding_header: &str,
    mode_header: &str,
    user: &str,
) {
    assert_eq!(
        combined,
        format!("{coding_header}\n\n{mode_header}\n\n{user}")
    );
    assert_eq!(combined.split("\n\n").count(), 3);
    assert!(combined.contains(coding_header));
    assert!(combined.contains(mode_header));
    assert!(combined.ends_with(user));
    assert_eq!(combined.matches(user).count(), 1);
}
