use std::sync::atomic::{AtomicU8, Ordering};

pub(crate) const ANSI_BOLD: &str = "\x1b[1m";
pub(crate) const ANSI_DIM: &str = "\x1b[90m";
pub(crate) const ANSI_ITALIC: &str = "\x1b[3m";
pub(crate) const ANSI_RESET: &str = "\x1b[0m";

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum TerminalTheme {
    #[default]
    Dark,
    Light,
}

#[derive(Clone, Copy)]
pub(crate) struct Palette {
    pub(crate) error: &'static str,
    pub(crate) warning: &'static str,
    pub(crate) who_tag: &'static str,
    pub(crate) accent: &'static str,
    pub(crate) tool_name: &'static str,
    pub(crate) body: &'static str,
    pub(crate) remote: &'static str,
}

const DARK_PALETTE: Palette = Palette {
    error: "\x1b[38;2;224;122;95m",
    warning: "\x1b[38;2;245;158;66m",
    who_tag: "\x1b[38;2;110;113;142m",
    accent: "\x1b[38;2;129;178;154m",
    tool_name: "\x1b[38;2;158;128;78m",
    body: "\x1b[38;2;235;235;235m",
    remote: "\x1b[38;2;240;228;196m",
};

const LIGHT_PALETTE: Palette = Palette {
    error: "\x1b[38;2;179;78;61m",
    warning: "\x1b[38;2;196;98;44m",
    who_tag: "\x1b[38;2;55;57;72m",
    accent: "\x1b[38;2;77;118;98m",
    tool_name: "\x1b[38;2;48;48;50m",
    body: "\x1b[38;2;24;24;26m",
    remote: "\x1b[38;2;128;106;64m",
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

pub(crate) fn ansi_error() -> &'static str {
    active_palette().error
}

pub(crate) fn ansi_warning() -> &'static str {
    active_palette().warning
}

pub(crate) fn ansi_who_tag() -> &'static str {
    active_palette().who_tag
}

pub(crate) fn ansi_accent() -> &'static str {
    active_palette().accent
}

pub(crate) fn ansi_tool_name() -> &'static str {
    active_palette().tool_name
}

pub(crate) fn ansi_body() -> &'static str {
    active_palette().body
}

pub(crate) fn ansi_remote() -> &'static str {
    active_palette().remote
}

#[cfg(test)]
#[path = "terminal_palette_tests.rs"]
mod terminal_palette_tests;
