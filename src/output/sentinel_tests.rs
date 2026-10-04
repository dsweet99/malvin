use super::{DM_END, DM_START, MALVIN_DONE, is_sentinel_line, may_become_sentinel_line};

const FIXTURE: &str = include_str!("../../tests/fixtures/sentinel_lines.json");

fn fixture() -> serde_json::Value {
    serde_json::from_str(FIXTURE).expect("sentinel fixture is valid JSON")
}

#[test]
fn fixture_markers_match_constants() {
    let markers = &fixture()["markers"];
    assert_eq!(markers["done"], MALVIN_DONE);
    assert_eq!(markers["dm_start"], DM_START);
    assert_eq!(markers["dm_end"], DM_END);
}

#[test]
fn fixture_cases_match_rule() {
    let fixture = fixture();
    let cases = fixture["cases"].as_array().expect("cases array");
    assert!(!cases.is_empty());
    for case in cases {
        let marker_key = case["marker"].as_str().expect("marker");
        let marker = fixture["markers"][marker_key].as_str().expect("known marker");
        let line = case["line"].as_str().expect("line");
        let expected = case["matches"].as_bool().expect("matches");
        assert_eq!(
            is_sentinel_line(line, marker),
            expected,
            "line={line:?} marker={marker}"
        );
    }
}

#[test]
fn partial_lines_that_may_still_become_markers() {
    assert!(may_become_sentinel_line("", DM_START));
    assert!(may_become_sentinel_line("__MALVIN_DM_", DM_START));
    assert!(may_become_sentinel_line("  __MALVIN_DM_", DM_START));
    assert!(may_become_sentinel_line("__MALVIN_DM_START__  ", DM_START));
    assert!(!may_become_sentinel_line("x__MALVIN", DM_START));
    assert!(!may_become_sentinel_line("__MALVIN_DM_START__ x", DM_START));
}
