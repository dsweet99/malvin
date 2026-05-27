//! Shared RGB terminal palette for malvin stdout styling (who-tags, tool summaries, ACP brackets).

pub(crate) const ANSI_BOLD: &str = "\x1b[1m";
pub(crate) const ANSI_DIM: &str = "\x1b[90m";
pub(crate) const ANSI_RESET: &str = "\x1b[0m";
pub(crate) const ANSI_TOOL_CORAL: &str = "\x1b[38;2;224;122;95m";
pub(crate) const ANSI_TOOL_AMBER: &str = "\x1b[38;2;245;158;66m";
pub(crate) const ANSI_TOOL_NAVY: &str = "\x1b[38;2;91;94;121m";
pub(crate) const ANSI_TOOL_TEAL: &str = "\x1b[38;2;129;178;154m";
/// Darker sand for tool brackets and verb names (was bright sand).
pub(crate) const ANSI_TOOL_DARK: &str = "\x1b[38;2;158;128;78m";
pub(crate) const ANSI_TOOL_WHITE: &str = "\x1b[38;2;235;235;235m";
