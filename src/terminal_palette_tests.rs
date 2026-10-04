use super::{
    ACTIVE_THEME, DARK_PALETTE, LIGHT_PALETTE, Palette, THEME_DARK, TerminalTheme, active_palette,
    ansi_accent, ansi_body, ansi_error, ansi_tool_name, ansi_warning, ansi_who_tag,
    init_terminal_theme,
};
use std::sync::atomic::Ordering;

type SemanticEscape = (&'static str, fn() -> &'static str);

fn all_semantic_escapes() -> [SemanticEscape; 6] {
    [
        ("error", ansi_error),
        ("warning", ansi_warning),
        ("who_tag", ansi_who_tag),
        ("accent", ansi_accent),
        ("tool_name", ansi_tool_name),
        ("body", ansi_body),
    ]
}

#[test]
fn light_palette_darkens_semantic_colors_and_inverts_body_and_tool_name() {
    init_terminal_theme(TerminalTheme::Light);
    assert!(ansi_who_tag().contains("55;57;72"));
    assert!(ansi_body().contains("24;24;26"));
    assert!(ansi_tool_name().contains("48;48;50"));
    init_terminal_theme(TerminalTheme::Dark);
    assert!(ansi_body().contains("235;235;235"));
    assert!(ansi_tool_name().contains("158;128;78"));
}

#[test]
fn dark_palette_matches_legacy_constants() {
    init_terminal_theme(TerminalTheme::Dark);
    assert_eq!(ansi_error(), DARK_PALETTE.error);
    assert_eq!(ansi_warning(), DARK_PALETTE.warning);
    assert_eq!(ansi_who_tag(), DARK_PALETTE.who_tag);
    assert_eq!(ansi_accent(), DARK_PALETTE.accent);
    assert_eq!(ansi_tool_name(), DARK_PALETTE.tool_name);
    assert_eq!(ansi_body(), DARK_PALETTE.body);
}

#[test]
fn light_palette_exposes_all_semantic_slots() {
    init_terminal_theme(TerminalTheme::Light);
    for (name, escape) in all_semantic_escapes() {
        let seq = escape();
        assert!(seq.starts_with("\x1b[38;2;"), "{name} must be RGB ANSI");
        assert!(seq.ends_with('m'), "{name} must end with m");
    }
    assert_eq!(ansi_error(), LIGHT_PALETTE.error);
    assert_eq!(ansi_body(), LIGHT_PALETTE.body);
    init_terminal_theme(TerminalTheme::Dark);
}

#[test]
fn active_palette_selects_dark_and_light_tables() {
    init_terminal_theme(TerminalTheme::Light);
    let light: Palette = active_palette();
    assert_eq!(light.who_tag, LIGHT_PALETTE.who_tag);

    init_terminal_theme(TerminalTheme::Dark);
    let dark: Palette = active_palette();
    assert_eq!(dark.who_tag, DARK_PALETTE.who_tag);

    ACTIVE_THEME.store(THEME_DARK, Ordering::Relaxed);
    assert_eq!(active_palette().who_tag, DARK_PALETTE.who_tag);
}

#[test]
fn default_theme_before_init_is_dark() {
    ACTIVE_THEME.store(THEME_DARK, Ordering::Relaxed);
    assert_eq!(ansi_who_tag(), DARK_PALETTE.who_tag);
}
