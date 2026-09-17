use super::{RequestKind, classify_top_level_requests};
use std::ffi::OsString;

fn os(args: &[&str]) -> Vec<OsString> {
    args.iter().map(OsString::from).collect()
}

#[test]
fn each_do_tags_only_the_next_request() {
    let tagged = classify_top_level_requests(&os(&[
        "malvin",
        "Write",
        "--do",
        "What time",
        "Find bug",
        "--do",
        "Summarize",
    ]))
    .expect("classify");
    assert_eq!(
        tagged
            .iter()
            .map(|t| (t.text.as_str(), t.kind))
            .collect::<Vec<_>>(),
        vec![
            ("Write", RequestKind::Router),
            ("What time", RequestKind::Do),
            ("Find bug", RequestKind::Router),
            ("Summarize", RequestKind::Do),
        ]
    );
}

#[test]
fn do_then_trailing_request_is_router() {
    let tagged =
        classify_top_level_requests(&os(&["malvin", "--do", "Hello", "Research"])).expect("ok");
    assert_eq!(tagged[0].kind, RequestKind::Do);
    assert_eq!(tagged[1].kind, RequestKind::Router);
}

#[test]
fn bare_do_without_request_is_ok_when_empty() {
    let tagged = classify_top_level_requests(&os(&["malvin", "--do"])).expect("ok");
    assert!(tagged.is_empty());
}

#[test]
fn do_without_request_after_other_requests_errors() {
    let err = classify_top_level_requests(&os(&["malvin", "A", "--do"])).unwrap_err();
    assert!(err.contains("--do"));
}
