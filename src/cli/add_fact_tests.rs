use super::*;

fn args(items: &[&str]) -> Vec<String> {
    items.iter().map(|s| (*s).to_string()).collect()
}

#[test]
fn parses_fields_in_any_order_with_repeated_symptoms() {
    let a = parse_args(&args(&[
        "symptoms=s1",
        "fact=Body text = ok",
        "fact_id=my-id",
        "symptoms=s2",
    ]))
    .expect("parse");
    assert_eq!(a.fact_id, "my-id");
    assert_eq!(a.fact, "Body text = ok");
    assert_eq!(a.symptoms, vec!["s1", "s2"]);
}

#[test]
fn rejects_bad_arguments() {
    let cases: &[&[&str]] = &[
        &["fact_id=a1", "fact"],
        &["fact_id=a1", "fact=x", "bogus=1"],
        &["fact_id=a1", "fact_id=a2", "fact=x"],
        &["fact=x"],
        &["fact_id=a1"],
        &["fact_id=A1", "fact=x"],
        &["fact_id=1a", "fact=x"],
        &["fact_id=a1", "fact=  "],
        &["fact_id=a1", "fact=line1\nline2"],
        &["fact_id=a1", "fact=bell\u{7}"],
        &["fact_id=a1", "fact=zero\u{200B}width"],
        &["fact_id=a1", "fact=x", "symptoms="],
    ];
    for case in cases {
        assert!(parse_args(&args(case)).is_err(), "{case:?}");
    }
    let long = format!("fact={}", "x".repeat(FACT_MAX + 1));
    assert!(parse_args(&args(&["fact_id=a1", &long])).is_err());
}

#[test]
fn rejects_too_many_symptoms() {
    let mut v = args(&["fact_id=a1", "fact=x"]);
    v.extend((0..=SYMPTOMS_MAX).map(|i| format!("symptoms=s{i}")));
    assert!(parse_args(&v).is_err());
}

#[test]
fn append_creates_file_enforces_unique_id_and_fixes_missing_newline() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("sub").join("facts.jsonl");
    let first = parse_args(&args(&["fact_id=one", "fact=First \"quoted\""])).expect("parse");
    append_fact_at(&path, &first).expect("create");
    let dup = append_fact_at(&path, &first).expect_err("duplicate");
    assert!(dup.contains("already exists"), "{dup}");
    let text = std::fs::read_to_string(&path).expect("read");
    std::fs::write(&path, text.trim_end()).expect("strip newline");
    let second = parse_args(&args(&["fact_id=two", "fact=Second", "symptoms=s"])).expect("parse");
    append_fact_at(&path, &second).expect("append");
    let text = std::fs::read_to_string(&path).expect("read");
    let lines: Vec<serde_json::Value> = text
        .lines()
        .map(|l| serde_json::from_str(l).expect("json"))
        .collect();
    assert_eq!(lines.len(), 2);
    assert_eq!(lines[0]["fact"], "First \"quoted\"");
    assert_eq!(lines[0]["symptoms"], serde_json::json!([]));
    assert_eq!(lines[1]["symptoms"], serde_json::json!(["s"]));
    assert!(!dir.path().join("sub").join("facts.jsonl.lock").exists());
}

#[test]
fn stale_lock_times_out_with_clear_error() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("facts.jsonl");
    std::fs::write(dir.path().join("facts.jsonl.lock"), "").expect("lock");
    let new = parse_args(&args(&["fact_id=one", "fact=x"])).expect("parse");
    let err = append_fact_at(&path, &new).expect_err("locked");
    assert!(err.contains("locked"), "{err}");
}
