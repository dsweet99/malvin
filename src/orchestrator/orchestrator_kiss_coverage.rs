use super::check_abort;

#[test]
fn smoke_check_abort_ok_when_no_result_file() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let result = tmp.path().join("result.md");
    assert!(check_abort(&result).expect("missing is ok").is_none());
}
