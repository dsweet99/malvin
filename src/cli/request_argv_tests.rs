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
            .map(|t| (t.text.as_str(), t.kind, t.creative))
            .collect::<Vec<_>>(),
        vec![
            ("Write", RequestKind::Router, None),
            ("What time", RequestKind::Do, None),
            ("Find bug", RequestKind::Router, None),
            ("Summarize", RequestKind::Do, None),
        ]
    );
}

#[test]
fn each_creative_tags_only_the_next_request() {
    let tagged = classify_top_level_requests(&os(&[
        "malvin",
        "Write",
        "--creative",
        "Explore",
        "Find bug",
        "--creative=0.4",
        "Summarize",
    ]))
    .expect("classify");
    assert_eq!(
        tagged
            .iter()
            .map(|t| (t.text.as_str(), t.kind, t.creative))
            .collect::<Vec<_>>(),
        vec![
            ("Write", RequestKind::Router, None),
            ("Explore", RequestKind::Router, Some(1.0)),
            ("Find bug", RequestKind::Router, None),
            ("Summarize", RequestKind::Router, Some(0.4)),
        ]
    );
}

#[test]
fn creative_survives_intervening_global_flags() {
    let tagged = classify_top_level_requests(&os(&[
        "malvin",
        "--creative=0.7",
        "--max-loops",
        "3",
        "task",
    ]))
    .expect("classify");
    assert_eq!(tagged.len(), 1);
    assert_eq!(tagged[0].creative, Some(0.7));
    assert_eq!(tagged[0].kind, RequestKind::Router);
}

#[test]
fn creative_then_do_without_request_errors() {
    let err = classify_top_level_requests(&os(&["malvin", "--creative", "--do", "x"])).unwrap_err();
    assert!(err.contains("--creative"), "{err}");
}

#[test]
fn do_then_creative_without_request_errors() {
    let err = classify_top_level_requests(&os(&["malvin", "--do", "--creative", "x"])).unwrap_err();
    assert!(err.contains("--do"), "{err}");
}

#[test]
fn do_then_trailing_request_is_router() {
    let tagged =
        classify_top_level_requests(&os(&["malvin", "--do", "Hello", "Research"])).expect("ok");
    assert_eq!(tagged[0].kind, RequestKind::Do);
    assert_eq!(tagged[1].kind, RequestKind::Router);
    assert!(tagged[0].creative.is_none());
    assert!(tagged[1].creative.is_none());
}

#[test]
fn bare_do_without_request_is_ok_when_empty() {
    let tagged = classify_top_level_requests(&os(&["malvin", "--do"])).expect("ok");
    assert!(tagged.is_empty());
}

#[test]
fn bare_creative_without_request_is_ok_when_empty() {
    let tagged = classify_top_level_requests(&os(&["malvin", "--creative"])).expect("ok");
    assert!(tagged.is_empty());
}

#[test]
fn do_without_request_after_other_requests_errors() {
    let err = classify_top_level_requests(&os(&["malvin", "A", "--do"])).unwrap_err();
    assert!(err.contains("--do"));
}

#[test]
fn creative_without_request_after_other_requests_errors() {
    let err = classify_top_level_requests(&os(&["malvin", "A", "--creative"])).unwrap_err();
    assert!(err.contains("--creative"));
}

#[test]
fn do_then_creative_on_next_router_request() {
    let tagged = classify_top_level_requests(&os(&[
        "malvin",
        "--do",
        "shot",
        "--creative=0.5",
        "route me",
    ]))
    .expect("ok");
    assert_eq!(tagged[0].kind, RequestKind::Do);
    assert!(tagged[0].creative.is_none());
    assert_eq!(tagged[1].kind, RequestKind::Router);
    assert_eq!(tagged[1].creative, Some(0.5));
}

#[test]
fn advice_with_tag_consumes_following_token() {
    let tagged = classify_top_level_requests(&os(&["malvin", "--advice", "design"]))
        .expect("classify");
    assert!(tagged.is_empty(), "{tagged:?}");
}

#[test]
fn bare_advice_alone_leaves_no_requests() {
    let tagged = classify_top_level_requests(&os(&["malvin", "--advice"])).expect("ok");
    assert!(tagged.is_empty());
}

#[test]
fn advice_equals_form_does_not_skip_next_request() {
    let tagged =
        classify_top_level_requests(&os(&["malvin", "--advice=design", "Write"])).expect("ok");
    assert_eq!(tagged.len(), 1);
    assert_eq!(tagged[0].text, "Write");
}

#[test]
fn bare_advice_before_flag_does_not_skip_flag_token() {
    let tagged =
        classify_top_level_requests(&os(&["malvin", "--advice", "--max-loops", "2", "Write"]))
            .expect("ok");
    assert_eq!(tagged.len(), 1);
    assert_eq!(tagged[0].text, "Write");
}
