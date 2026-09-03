#![cfg(all(test, windows))]

use std::fs;
use std::process::Command;

use super::{
    apply_fake_path_if_present, fake_command_dir_for_path_env, run_command_for,
    set_fake_command_dir,
};

#[test]
fn fake_command_dir_resolves_batch_command() {
    let tmp = tempfile::tempdir().unwrap();
    let p = tmp.path().to_path_buf();
    let kiss = p.join("kiss.bat");
    fs::write(&kiss, "@echo off\r\nexit /b 0\r\n").unwrap();
    let _guard = set_fake_command_dir(&p);
    assert_eq!(fake_command_dir_for_path_env(), Some(p.clone()));
    let mut cmd = Command::new("kiss");
    apply_fake_path_if_present(&mut cmd);
    assert_eq!(run_command_for("kiss"), kiss);
}
