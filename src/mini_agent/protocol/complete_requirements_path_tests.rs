use super::complete_requirements_path::requirements_file_on_disk_is_valid;

#[test]
fn on_disk_rejects_empty_object_and_blank_requirements() {
    let dir = std::env::temp_dir();
    let pid = std::process::id();
    let cases = [
        (
            format!("malvin_req_empty_{pid}.json"),
            r#"{"groups":[{"title":"t","requirements":[]}]}"#,
        ),
        (
            format!("malvin_req_obj_{pid}.json"),
            r#"{"groups":[{"title":"t","requirements":[{"id":"1"}]}]}"#,
        ),
        (
            format!("malvin_req_blank_{pid}.json"),
            r#"{"groups":[{"title":"t","requirements":["  "]}]}"#,
        ),
        (
            format!("malvin_req_nogroups_{pid}.json"),
            r#"{"not_groups":[]}"#,
        ),
    ];
    for (name, body) in cases {
        let path = dir.join(name);
        std::fs::write(&path, body).expect("write");
        let path_s = path.to_string_lossy().into_owned();
        assert!(
            !requirements_file_on_disk_is_valid(&path_s),
            "expected invalid: {body}"
        );
        let _ = std::fs::remove_file(&path);
    }
    assert!(!requirements_file_on_disk_is_valid(
        "/tmp/malvin_req_missing_does_not_exist.json"
    ));
}
