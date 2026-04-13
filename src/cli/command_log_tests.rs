use std::os::unix::fs::PermissionsExt;

use malvin::output::{MALVIN_WHO, format_log_tag_inner};
use tempfile::tempdir;

use super::emit_command_line;

#[test]
fn emit_command_line_writes_command_log_when_run_dir_is_writable() {
    let tmp = tempdir().unwrap();
    let run = tmp.path().join("run");
    std::fs::create_dir_all(&run).unwrap();
    emit_command_line(&run, true).unwrap();
    let p = run.join("command.log");
    assert!(p.is_file(), "command.log should record argv beside the run");
    let text = std::fs::read_to_string(&p).expect("read command.log");
    let inner = format_log_tag_inner(MALVIN_WHO);
    assert!(
        text.contains(&format!(":[{inner}]: Command: ")) && text.ends_with('\n'),
        "command.log should match stdout line format; got {text:?}"
    );
}

#[test]
fn emit_command_line_writes_command_log_when_stdout_echo_suppressed() {
    let tmp = tempdir().unwrap();
    let run = tmp.path().join("run");
    std::fs::create_dir_all(&run).unwrap();
    emit_command_line(&run, false).unwrap();
    let p = run.join("command.log");
    assert!(
        p.is_file(),
        "command.log should still record argv when tee is off"
    );
    let text = std::fs::read_to_string(&p).expect("read command.log");
    let inner = format_log_tag_inner(MALVIN_WHO);
    assert!(
        text.contains(&format!(":[{inner}]: Command: ")) && text.ends_with('\n'),
        "command.log should match stdout format when tee is off; got {text:?}"
    );
}

#[test]
fn emit_command_line_returns_error_when_command_log_cannot_be_created() {
    let tmp = tempdir().unwrap();
    let run = tmp.path().join("run");
    std::fs::create_dir_all(&run).unwrap();
    std::fs::set_permissions(&run, PermissionsExt::from_mode(0o555)).unwrap();
    let err = emit_command_line(&run, true).unwrap_err();
    assert!(
        !run.join("command.log").exists(),
        "expected no command.log when write fails; err={err}"
    );
    std::fs::set_permissions(&run, PermissionsExt::from_mode(0o755)).unwrap();
}
