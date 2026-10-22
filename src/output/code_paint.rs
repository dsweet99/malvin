use crate::terminal_palette::theme_is_light;

#[derive(Clone, Copy)]
pub(super) enum Kind {
    Keyword,
    String,
    Comment,
    Number,
    Code,
}

const DARK: [&str; 5] = [
    "\x1b[38;2;203;166;247m",
    "\x1b[38;2;166;227;161m",
    "\x1b[38;2;108;112;134m",
    "\x1b[38;2;250;179;135m",
    "\x1b[38;2;137;180;250m",
];

const LIGHT: [&str; 5] = [
    "\x1b[38;2;136;57;239m",
    "\x1b[38;2;64;120;48m",
    "\x1b[38;2;108;112;134m",
    "\x1b[38;2;180;90;20m",
    "\x1b[38;2;30;102;175m",
];

pub(super) fn paint(kind: Kind, text: &str) -> String {
    format!(
        "{}{text}{}",
        color(kind),
        crate::terminal_palette::ANSI_RESET
    )
}

fn color(kind: Kind) -> &'static str {
    let palette = if theme_is_light() { &LIGHT } else { &DARK };
    palette[kind_index(kind)]
}

const fn kind_index(kind: Kind) -> usize {
    match kind {
        Kind::Keyword => 0,
        Kind::String => 1,
        Kind::Comment => 2,
        Kind::Number => 3,
        Kind::Code => 4,
    }
}
