//! Shared RGB terminal palette for malvin stdout styling (who-tags, tool summaries, ACP brackets).

pub(crate) const ANSI_BOLD: &str = "\x1b[1m";
pub(crate) const ANSI_DIM: &str = "\x1b[90m";
pub(crate) const ANSI_RESET: &str = "\x1b[0m";
pub(crate) const ANSI_TOOL_CREAM: &str = "\x1b[38;2;244;241;222m";
pub(crate) const ANSI_TOOL_CORAL: &str = "\x1b[38;2;224;122;95m";
pub(crate) const ANSI_TOOL_AMBER: &str = "\x1b[38;2;245;158;66m";
pub(crate) const ANSI_TOOL_NAVY: &str = "\x1b[38;2;91;94;121m";
pub(crate) const ANSI_TOOL_TEAL: &str = "\x1b[38;2;129;178;154m";
pub(crate) const ANSI_TOOL_SAND: &str = "\x1b[38;2;242;204;143m";
