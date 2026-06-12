use crate::artifacts::{
    create_kpop_run_artifacts, GitignoreBackup, backup_workspace_gitignore_if_present,
    backup_workspace_gitignore_if_present_with_id, restore_workspace_gitignore_backup,
    SessionDotfileBackups,
};
use crate::session_dotfile_backup::DotfileBackupState;
use crate::test_utils::with_isolated_home;

#[test]
fn gitignore_backup_skips_when_workspace_file_missing() {
    let tmp = tempfile::tempdir().unwrap();
    let work = tmp.path().join("empty");
    std::fs::create_dir_all(&work).unwrap();
    assert_eq!(
        backup_workspace_gitignore_if_present(&work).unwrap(),
        GitignoreBackup::Missing
    );
}

#[test]
fn gitignore_backup_round_trip_restores_workspace_file() {
    with_isolated_home(|work| {
        std::fs::write(work.join(".gitignore"), "ORIGINAL\n").unwrap();
        let backup = backup_workspace_gitignore_if_present(work).unwrap();
        let GitignoreBackup::Present(payload) = &backup else {
            panic!("expected backup path");
        };
        assert!(payload.backup_path.is_file());
        std::fs::write(work.join(".gitignore"), "MODIFIED\n").unwrap();
        restore_workspace_gitignore_backup(work, &backup).unwrap();
        assert_eq!(
            std::fs::read_to_string(work.join(".gitignore")).unwrap(),
            "ORIGINAL\n"
        );
    });
}

#[test]
fn gitignore_backup_missing_restores_by_removing_created_workspace_file() {
    let tmp = tempfile::tempdir().unwrap();
    let work = tmp.path().join("repo");
    std::fs::create_dir_all(&work).unwrap();
    let backup = backup_workspace_gitignore_if_present(&work).unwrap();
    std::fs::write(work.join(".gitignore"), "CREATED\n").unwrap();
    restore_workspace_gitignore_backup(&work, &backup).unwrap();
    assert!(!work.join(".gitignore").exists());
}

#[test]
fn gitignore_backup_retries_on_existing_collision() {
    with_isolated_home(|work| {
        let home = std::env::var_os("HOME").unwrap();
        let dir = std::path::Path::new(&home)
            .join(crate::MALVIN_USER_HOME_DIR)
            .join("gitignore_snapshots");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::create_dir_all(dir.join("aaaaa")).unwrap();

        std::fs::write(work.join(".gitignore"), "ORIGINAL\n").unwrap();
        let backup = backup_workspace_gitignore_if_present_with_id(work, |attempt| {
            if attempt == 0 {
                "aaaaa".to_string()
            } else {
                "bbbbb".to_string()
            }
        })
        .unwrap();

        let GitignoreBackup::Present(payload) = &backup else {
            panic!("expected backup path");
        };
        assert_eq!(payload.backup_path.parent(), Some(dir.join("bbbbb").as_path()));
        assert!(dir.join("bbbbb").join(".gitignore").is_file());
    });
}

/// Post-agent snapshot must capture agent-created dotfiles before intra-session restore wipes them.
#[test]
fn post_agent_snapshot_preserves_agent_created_kissignore_for_gate_restore() {
    with_isolated_home(|work| {
        std::fs::create_dir_all(work.join(".malvin")).unwrap();
        std::fs::write(work.join(".malvin/checks"), "kiss check .\n").unwrap();
        let iteration_start =
            SessionDotfileBackups::snapshot_after_ensuring_home_config(work).unwrap();
        assert!(matches!(iteration_start.kissignore, DotfileBackupState::Missing));
        std::fs::write(work.join(".kissignore"), "target/\nops/\n").unwrap();
        let post_agent =
            SessionDotfileBackups::snapshot_after_ensuring_home_config(work).unwrap();
        iteration_start
            .restore_excluding_malvin_checks(work)
            .unwrap();
        assert!(!work.join(".kissignore").exists(), "intra-session restore wipes agent repair");
        post_agent
            .restore_excluding_malvin_checks(work)
            .unwrap();
        assert_eq!(
            std::fs::read_to_string(work.join(".kissignore")).unwrap(),
            "target/\nops/\n"
        );
    });
}

/// `KPop` outer loops snapshot dotfiles at each iteration start; restore must undo tamper before the next snapshot.
#[test]
fn outer_loop_resnapshot_restores_gitignore_between_iterations() {
    const BASELINE: &str = "baseline\n";
    with_isolated_home(|work| {
        let gitignore = work.join(".gitignore");
        std::fs::write(&gitignore, BASELINE).unwrap();
        let _artifacts = create_kpop_run_artifacts("kpop", Some(work)).expect("artifacts");
        let loop1 = SessionDotfileBackups::snapshot_after_ensuring_home_config(work).unwrap();
        std::fs::write(&gitignore, "tampered\n").unwrap();
        loop1.restore_excluding_malvin_checks(work).unwrap();
        assert_eq!(std::fs::read_to_string(&gitignore).unwrap(), BASELINE);
        let loop2 = SessionDotfileBackups::snapshot_after_ensuring_home_config(work).unwrap();
        std::fs::write(&gitignore, "tampered-again\n").unwrap();
        loop2.restore_excluding_malvin_checks(work).unwrap();
        assert_eq!(std::fs::read_to_string(&gitignore).unwrap(), BASELINE);
    });
}
