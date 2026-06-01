//! Idempotency and no-overwrite integration tests for `malvin init`.

mod common;

use common::{malvin_init_output, InitOk};

#[test]
fn malvin_init_succeeds_twice_in_temp_directory() {
    let w = InitOk::new(&["python"]);
    let (out, _home) = malvin_init_output(w.path(), &["python"]);
    assert!(
        out.status.success(),
        "second malvin init in same temp repo should succeed: {out:?}"
    );
}

#[test]
fn malvin_init_does_not_overwrite_existing_template_files() {
    let w = InitOk::new(&["python"]);
    let gitignore = w.path().join(".gitignore");
    std::fs::write(&gitignore, "custom-marker\n").expect("write marker");
    let (out, _home) = malvin_init_output(w.path(), &["python"]);
    assert!(out.status.success(), "second init failed: {out:?}");
    assert_eq!(
        std::fs::read_to_string(&gitignore).expect("read gitignore"),
        "custom-marker\n"
    );
}
