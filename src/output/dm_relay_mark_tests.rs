use super::{mark_line, parse_dm_emit_mark};

#[test]
fn mark_round_trips_a_multiline_body() {
    let body = "hello\nthere — café";
    let line = mark_line(body);
    assert!(line.starts_with("__MALVIN_DM_EMIT__"));
    assert_eq!(parse_dm_emit_mark(&line), Some(Ok(body.to_string())));
}

#[test]
fn ordinary_text_is_not_a_mark_and_broken_hex_is_an_error() {
    assert_eq!(parse_dm_emit_mark("hello"), None);
    assert_eq!(parse_dm_emit_mark("__MALVIN_DM_EMIT__zz"), Some(Err(())));
    assert_eq!(
        parse_dm_emit_mark("__MALVIN_DM_EMIT__"),
        Some(Ok(String::new()))
    );
}
