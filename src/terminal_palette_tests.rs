use super::{
    active_palette, ansi_tool_amber, ansi_tool_coral, ansi_tool_dark, ansi_tool_navy, ansi_tool_teal,
    ansi_tool_white, init_terminal_theme, Palette, TerminalTheme, ACTIVE_THEME, DARK_PALETTE,
    LIGHT_PALETTE, THEME_DARK,
};
use std::sync::atomic::Ordering;

type SemanticEscape = (&'static str, fn() -> &'static str);

fn all_semantic_escapes() -> [SemanticEscape; 6] {
    [
        ("coral", ansi_tool_coral),
        ("amber", ansi_tool_amber),
        ("navy", ansi_tool_navy),
        ("teal", ansi_tool_teal),
        ("dark", ansi_tool_dark),
        ("white", ansi_tool_white),
    ]
}

#[test]
fn light_palette_darkens_semantic_colors_and_inverts_white_and_cream() {
    init_terminal_theme(TerminalTheme::Light);
    assert!(ansi_tool_navy().contains("55;57;72"));
    assert!(ansi_tool_white().contains("24;24;26"));
    assert!(ansi_tool_dark().contains("48;48;50"));
    init_terminal_theme(TerminalTheme::Dark);
    assert!(ansi_tool_white().contains("235;235;235"));
    assert!(ansi_tool_dark().contains("158;128;78"));
}

#[test]
fn dark_palette_matches_legacy_constants() {
    init_terminal_theme(TerminalTheme::Dark);
    assert_eq!(ansi_tool_coral(), DARK_PALETTE.coral);
    assert_eq!(ansi_tool_amber(), DARK_PALETTE.amber);
    assert_eq!(ansi_tool_navy(), DARK_PALETTE.navy);
    assert_eq!(ansi_tool_teal(), DARK_PALETTE.teal);
    assert_eq!(ansi_tool_dark(), DARK_PALETTE.dark);
    assert_eq!(ansi_tool_white(), DARK_PALETTE.white);
}

#[test]
fn light_palette_exposes_all_semantic_slots() {
    init_terminal_theme(TerminalTheme::Light);
    for (name, escape) in all_semantic_escapes() {
        let seq = escape();
        assert!(seq.starts_with("\x1b[38;2;"), "{name} must be RGB ANSI");
        assert!(seq.ends_with('m'), "{name} must end with m");
    }
    assert_eq!(ansi_tool_coral(), LIGHT_PALETTE.coral);
    assert_eq!(ansi_tool_white(), LIGHT_PALETTE.white);
}

#[test]
fn active_palette_selects_dark_and_light_tables() {
    init_terminal_theme(TerminalTheme::Light);
    let light: Palette = active_palette();
    assert_eq!(light.navy, LIGHT_PALETTE.navy);

    init_terminal_theme(TerminalTheme::Dark);
    let dark: Palette = active_palette();
    assert_eq!(dark.navy, DARK_PALETTE.navy);

    ACTIVE_THEME.store(THEME_DARK, Ordering::Relaxed);
    assert_eq!(active_palette().navy, DARK_PALETTE.navy);
}

#[test]
fn default_theme_before_init_is_dark() {
    ACTIVE_THEME.store(THEME_DARK, Ordering::Relaxed);
    assert_eq!(ansi_tool_navy(), DARK_PALETTE.navy);
}
