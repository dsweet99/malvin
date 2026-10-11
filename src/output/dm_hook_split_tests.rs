use super::{bind_body, split_on_emit, substitute_body};

#[test]
fn splits_a_simple_command_and_keeps_body_as_one_word() {
    let tokens = split_on_emit("slack send {body}").expect("split");
    assert_eq!(tokens, ["slack", "send", "{body}"]);
    let (argv, used) = bind_body(&tokens, "hello\nthere");
    assert!(used);
    assert_eq!(argv, ["slack", "send", "hello\nthere"]);
}

#[test]
fn quotes_keep_spaces_and_a_partial_body_token_stays_literal() {
    let tokens = split_on_emit("sh -c 'slack send \"$1\"' dm {body}").expect("split");
    assert_eq!(
        tokens,
        ["sh", "-c", "slack send \"$1\"", "dm", "{body}"]
    );
    let (argv, used) = bind_body(&["pre-{body}".into()], "x");
    assert!(!used);
    assert_eq!(argv, ["pre-{body}"]);
}

#[test]
fn body_word_is_shell_quoted_and_the_rest_of_the_command_stays() {
    let (script, used) = substitute_body("echo {body} > /tmp/dm", "a'b $(id)").expect("sub");
    assert!(used);
    assert_eq!(script, "echo 'a'\\''b $(id)' > /tmp/dm");
    let (script, used) = substitute_body("cat > /tmp/dm", "x").expect("stdin");
    assert!(!used);
    assert_eq!(script, "cat > /tmp/dm");
    let (script, used) = substitute_body("echo pre-{body}", "x").expect("literal");
    assert!(!used);
    assert_eq!(script, "echo pre-{body}");
}

fn empty_string_has_no_tokens_and_a_missing_quote_is_an_error() {
    assert!(split_on_emit("   ").expect("empty").is_empty());
    assert!(split_on_emit("sh -c 'unterminated").is_err());
    let tokens = split_on_emit("\"a b\" c").expect("double");
    assert_eq!(tokens, ["a b", "c"]);
}
