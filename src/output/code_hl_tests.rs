use super::{recolor_termimad_inline, recolor_termimad_text, show_code_line};

const KEYWORD: &str = "\x1b[38;2;203;166;247m";
const STRING: &str = "\x1b[38;2;166;227;161m";
const COMMENT: &str = "\x1b[38;2;108;112;134m";

#[test]
fn inline_rust_keywords_are_colored_and_backticks_are_removed() {
    let shown = recolor_termimad_inline("call `fn main()` now");
    assert!(!shown.contains('`'), "{shown:?}");
    assert!(shown.contains(KEYWORD), "{shown:?}");
    assert!(shown.contains("fn"), "{shown:?}");
    assert!(shown.contains("call"), "{shown:?}");
}

#[test]
fn list_item_backticks_are_highlighted() {
    let shown = recolor_termimad_text("- run `fn go()`", 80);
    assert!(!shown.contains('`'), "{shown:?}");
    assert!(shown.contains(KEYWORD), "{shown:?}");
    assert!(shown.contains("go"), "{shown:?}");
}

#[test]
fn keyword_phrase_without_punctuation_is_highlighted() {
    let shown = recolor_termimad_inline("use `let mut x`");
    assert!(!shown.contains('`'), "{shown:?}");
    assert!(shown.contains(KEYWORD), "{shown:?}");
    assert!(shown.contains("mut"), "{shown:?}");
}

#[test]
fn plain_backtick_span_uses_code_color() {
    let shown = recolor_termimad_inline("see `src/lib.rs`");
    assert!(!shown.contains('`'), "{shown:?}");
    assert!(
        shown.contains(crate::terminal_palette::ansi_accent()),
        "{shown:?}"
    );
    assert!(!shown.contains(KEYWORD), "{shown:?}");
}

#[test]
fn fenced_language_colors_strings_and_comments() {
    let shown = show_code_line("rust", "let name = \"ada\"; // hi", true);
    assert!(shown.contains(STRING), "{shown:?}");
    assert!(shown.contains(COMMENT), "{shown:?}");
    assert!(shown.contains(KEYWORD), "{shown:?}");
}

#[test]
fn color_off_keeps_the_code_line_plain() {
    let shown = show_code_line("rust", "fn main() {}", false);
    assert_eq!(shown, "fn main() {}");
}
