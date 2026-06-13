use crate::session_dotfile_backup::gate_restore_merge::{
    kissconfig_low_coverage_threshold, merge_for_gate_restore,
};
use crate::session_dotfile_backup::{
    DotfileBackupPayload, DotfileBackupState, GitignoreBackup, GitignoreFileBackup,
    SessionDotfileBackups,
};

fn present(bytes: &[u8]) -> DotfileBackupState {
    DotfileBackupState::Present(DotfileBackupPayload {
        backup_path: std::path::PathBuf::from("/tmp/test"),
        bytes: bytes.to_vec(),
    })
}

fn gitignore_present(bytes: &[u8]) -> GitignoreBackup {
    GitignoreBackup::Present {
        backup_root: std::path::PathBuf::from("/tmp/test"),
        files: vec![GitignoreFileBackup {
            rel: std::path::PathBuf::from(".gitignore"),
            bytes: bytes.to_vec(),
        }],
    }
}

fn bundle_with(
    gitignore: GitignoreBackup,
    kissconfig: DotfileBackupState,
    checks: DotfileBackupState,
    kissignore: DotfileBackupState,
) -> SessionDotfileBackups {
    SessionDotfileBackups {
        kissconfig,
        malvin_checks: checks,
        kissignore,
        malvin_config: DotfileBackupState::Missing,
        gitignore,
        malvin_config_workspace: DotfileBackupState::Missing,
    }
}

#[test]
fn merge_rejects_deleted_gitignore_and_bare_kiss_checks() {
    let anchor = bundle_with(
        gitignore_present(b"baseline\n"),
        present(b"[gate]\ntest_coverage_threshold = 90\n"),
        present(b"kiss check .\n"),
        DotfileBackupState::Missing,
    );
    let progress = bundle_with(
        GitignoreBackup::Missing,
        present(b"[gate]\ntest_coverage_threshold = 0\n"),
        present(b"kiss\n"),
        DotfileBackupState::Missing,
    );
    let merged = merge_for_gate_restore(&anchor, &progress);
    assert!(matches!(merged.gitignore, GitignoreBackup::Present { .. }));
    assert!(matches!(merged.malvin_checks, DotfileBackupState::Present(_)));
    let DotfileBackupState::Present(ref payload) = merged.kissconfig else {
        panic!("expected kissconfig present");
    };
    assert!(!kissconfig_low_coverage_threshold(&payload.bytes));
}

#[test]
fn merge_rejects_kissconfig_and_home_config_tamper() {
    let anchor = SessionDotfileBackups {
        kissconfig: present(b"x\n"),
        malvin_checks: present(b"kiss check .\n"),
        kissignore: DotfileBackupState::Missing,
        malvin_config: present(b"mem_limit_gb = 7\n"),
        gitignore: GitignoreBackup::Missing,
        malvin_config_workspace: DotfileBackupState::Missing,
    };
    let progress = SessionDotfileBackups {
        kissconfig: present(b"TAMPERED\n"),
        malvin_checks: present(b"kiss check .\n"),
        kissignore: DotfileBackupState::Missing,
        malvin_config: present(b"TAMPERED\n"),
        gitignore: GitignoreBackup::Missing,
        malvin_config_workspace: DotfileBackupState::Missing,
    };
    let merged = merge_for_gate_restore(&anchor, &progress);
    let DotfileBackupState::Present(ref kiss) = merged.kissconfig else {
        panic!("expected kissconfig");
    };
    assert_eq!(kiss.bytes, b"x\n");
    let DotfileBackupState::Present(ref cfg) = merged.malvin_config else {
        panic!("expected malvin_config");
    };
    assert_eq!(cfg.bytes, b"mem_limit_gb = 7\n");
}

#[test]
fn merge_rejects_tampered_malvin_checks() {
    let anchor = bundle_with(
        GitignoreBackup::Missing,
        DotfileBackupState::Missing,
        present(b"kiss check\n"),
        DotfileBackupState::Missing,
    );
    let progress = bundle_with(
        GitignoreBackup::Missing,
        DotfileBackupState::Missing,
        present(b"TAMPERED\n"),
        DotfileBackupState::Missing,
    );
    let merged = merge_for_gate_restore(&anchor, &progress);
    let DotfileBackupState::Present(ref payload) = merged.malvin_checks else {
        panic!("expected malvin_checks present");
    };
    assert_eq!(payload.bytes, b"kiss check\n");
}

#[test]
fn merge_keeps_agent_expanded_malvin_checks() {
    let anchor = bundle_with(
        GitignoreBackup::Missing,
        DotfileBackupState::Missing,
        present(b"kiss check\n"),
        DotfileBackupState::Missing,
    );
    let progress = bundle_with(
        GitignoreBackup::Missing,
        DotfileBackupState::Missing,
        present(b"kiss check\nruff check\n"),
        DotfileBackupState::Missing,
    );
    let merged = merge_for_gate_restore(&anchor, &progress);
    let DotfileBackupState::Present(ref payload) = merged.malvin_checks else {
        panic!("expected malvin_checks present");
    };
    assert_eq!(payload.bytes, b"kiss check\nruff check\n");
}

#[test]
fn merge_prefers_progress_that_repairs_clamp_damage_when_anchor_corrupted() {
    let anchor = bundle_with(
        GitignoreBackup::Missing,
        present(b"[gate]\ntest_coverage_threshold = 0\n"),
        present(b"kiss\n"),
        DotfileBackupState::Missing,
    );
    let progress = bundle_with(
        GitignoreBackup::Missing,
        present(b"[gate]\ntest_coverage_threshold = 90\n"),
        present(b"kiss check .\n"),
        DotfileBackupState::Missing,
    );
    let merged = merge_for_gate_restore(&anchor, &progress);
    let DotfileBackupState::Present(ref kiss) = merged.kissconfig else {
        panic!("expected kissconfig");
    };
    assert!(!kissconfig_low_coverage_threshold(&kiss.bytes));
    let DotfileBackupState::Present(ref checks) = merged.malvin_checks else {
        panic!("expected malvin_checks");
    };
    assert_eq!(checks.bytes, b"kiss check .\n");
}

#[test]
fn merge_keeps_agent_created_kissignore() {
    let anchor = bundle_with(
        gitignore_present(b"baseline\n"),
        present(b"[gate]\ntest_coverage_threshold = 90\n"),
        present(b"kiss check .\n"),
        DotfileBackupState::Missing,
    );
    let progress = bundle_with(
        gitignore_present(b"baseline\n"),
        present(b"[gate]\ntest_coverage_threshold = 90\n"),
        present(b"kiss check .\n"),
        present(b"target/\n"),
    );
    let merged = merge_for_gate_restore(&anchor, &progress);
    assert!(matches!(merged.kissignore, DotfileBackupState::Present(_)));
}
