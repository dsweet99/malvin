use super::{RequestKind, classify_top_level_requests};
use std::ffi::OsString;

fn os(args: &[&str]) -> Vec<OsString> {
    args.iter().map(OsString::from).collect()
}

#[test]
fn max_loops_value_is_not_a_request() {
    let tagged = classify_top_level_requests(&os(&["malvin", "--max-loops", "2", "task"]))
        .expect("classify");
    assert_eq!(tagged.len(), 1);
    assert_eq!(tagged[0].text, "task");
    let eq =
        classify_top_level_requests(&os(&["malvin", "--max-loops=4", "other"])).expect("classify");
    assert_eq!(eq.len(), 1);
    assert_eq!(eq[0].text, "other");
}

#[test]
fn advice_with_tag_consumes_following_token() {
    let tagged =
        classify_top_level_requests(&os(&["malvin", "--advice", "design"])).expect("classify");
    assert!(tagged.is_empty(), "{tagged:?}");
}
#[test]
fn bare_advice_alone_leaves_no_requests_and_advice_equals_form_does_not_skip_next_request() {
    {
        let tagged = classify_top_level_requests(&os(&["malvin", "--advice"])).expect("ok");
        assert!(tagged.is_empty());
    }
    {
        let tagged =
            classify_top_level_requests(&os(&["malvin", "--advice=design", "Write"])).expect("ok");
        assert_eq!(tagged.len(), 1);
        assert_eq!(tagged[0].text, "Write");
    }
}

#[test]
fn bare_advice_before_flag_does_not_skip_flag_token_and_double_dash_ends_options() {
    let tagged = classify_top_level_requests(&os(&[
        "malvin",
        "--advice",
        "--max-acp-retries",
        "2",
        "Write",
    ]))
    .expect("ok");
    assert_eq!(tagged.len(), 1);
    assert_eq!(tagged[0].text, "Write");
    let dash = classify_top_level_requests(&os(&["malvin", "--", "-n"])).expect("classify");
    assert_eq!(dash.len(), 1);
    assert_eq!(dash[0].text, "-n");
    assert_eq!(dash[0].kind, RequestKind::Router);
    let bare = classify_top_level_requests(&os(&["malvin", "--"])).expect("classify");
    assert!(bare.is_empty());
    let literal =
        classify_top_level_requests(&os(&["malvin", "--", "--do", "task"])).expect("classify");
    assert_eq!(
        literal
            .iter()
            .map(|t| (t.text.as_str(), t.kind))
            .collect::<Vec<_>>(),
        vec![("--do", RequestKind::Router), ("task", RequestKind::Router)]
    );
    let pending =
        classify_top_level_requests(&os(&["malvin", "--do", "--", "-n"])).expect("classify");
    assert_eq!(pending.len(), 1);
    assert_eq!(pending[0].text, "-n");
    assert_eq!(pending[0].kind, RequestKind::Router);
}
