use crate::run_id::{build_identifier, create_run_dir, random_alnum};

#[test]
fn run_id_helpers() {
    let id = build_identifier();
    assert!(!id.is_empty());
    assert_eq!(random_alnum(4).len(), 4);
    let tmp = tempfile::tempdir().expect("tempdir");
    let dir = create_run_dir(Some(tmp.path())).expect("run dir");
    assert!(dir.is_dir());
}
