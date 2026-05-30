//! Shared RGB terminal palette for malvin stdout styling (who-tags, tool summaries, ACP brackets).

use std::sync::atomic::{AtomicU8, Ordering};

pub(crate) const ANSI_BOLD: &str = "\x1b[1m";
pub(crate) const ANSI_DIM: &str = "\x1b[90m";
pub(crate) const ANSI_RESET: &str = "\x1b[0m";

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum TerminalTheme {
    #[default]
    Dark,
    Light,
}

#[derive(Clone, Copy)]
pub(crate) struct Palette {
    coral: &'static str,
    amber: &'static str,
    navy: &'static str,
    teal: &'static str,
    /// Sand/cream brackets and verb names (dark theme) or very dark gray (light theme).
    dark: &'static str,
    /// Agent payload text (dark theme) or almost-black (light theme).
    white: &'static str,
}

const DARK_PALETTE: Palette = Palette {
    coral: "\x1b[38;2;224;122;95m",
    amber: "\x1b[38;2;245;158;66m",
    navy: "\x1b[38;2;110;113;142m",
    teal: "\x1b[38;2;129;178;154m",
    dark: "\x1b[38;2;158;128;78m",
    white: "\x1b[38;2;235;235;235m",
};

const LIGHT_PALETTE: Palette = Palette {
    coral: "\x1b[38;2;179;78;61m",
    amber: "\x1b[38;2;196;98;44m",
    navy: "\x1b[38;2;55;57;72m",
    teal: "\x1b[38;2;77;118;98m",
    dark: "\x1b[38;2;48;48;50m",
    white: "\x1b[38;2;24;24;26m",
};

const THEME_DARK: u8 = 0;
const THEME_LIGHT: u8 = 1;

static ACTIVE_THEME: AtomicU8 = AtomicU8::new(THEME_DARK);

pub fn init_terminal_theme(theme: TerminalTheme) {
    let id = match theme {
        TerminalTheme::Dark => THEME_DARK,
        TerminalTheme::Light => THEME_LIGHT,
    };
    ACTIVE_THEME.store(id, Ordering::Relaxed);
}

pub(crate) fn active_palette() -> Palette {
    match ACTIVE_THEME.load(Ordering::Relaxed) {
        THEME_LIGHT => LIGHT_PALETTE,
        _ => DARK_PALETTE,
    }
}

pub(crate) fn ansi_tool_coral() -> &'static str {
    active_palette().coral
}

pub(crate) fn ansi_tool_amber() -> &'static str {
    active_palette().amber
}

pub(crate) fn ansi_tool_navy() -> &'static str {
    active_palette().navy
}

pub(crate) fn ansi_tool_teal() -> &'static str {
    active_palette().teal
}

pub(crate) fn ansi_tool_dark() -> &'static str {
    active_palette().dark
}

pub(crate) fn ansi_tool_white() -> &'static str {
    active_palette().white
}

#[cfg(test)]
#[path = "terminal_palette_tests.rs"]
mod terminal_palette_tests;
