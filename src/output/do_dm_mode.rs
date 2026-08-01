//! Mode flags for DM-only process stdout (`--do`, and default-router `--quiet` / `-q`).

use std::sync::atomic::{AtomicBool, Ordering};

static DO_DM_STDOUT: AtomicBool = AtomicBool::new(false);
static DO_DM_MARKDOWN: AtomicBool = AtomicBool::new(false);

/// Options for [`set_do_dm_stdout_opts`].
#[derive(Debug, Clone, Copy, Default)]
pub struct DoDmStdoutOpts {
    /// When true, process stdout is DM-body-only.
    pub enabled: bool,
    /// When true (and enabled), style DM bodies as markdown on a TTY.
    pub emit_markdown: bool,
}

/// Enable or disable DM-only process stdout (`malvin --do`, or quiet default router).
pub fn set_do_dm_stdout_opts(opts: DoDmStdoutOpts) {
    DO_DM_STDOUT.store(opts.enabled, Ordering::Relaxed);
    DO_DM_MARKDOWN.store(opts.enabled && opts.emit_markdown, Ordering::Relaxed);
    super::do_dm_filter::reset_do_dm_filter();
}

/// Convenience: enable DM-only stdout without markdown styling.
pub fn set_do_dm_stdout_mode(enabled: bool) {
    set_do_dm_stdout_opts(DoDmStdoutOpts {
        enabled,
        emit_markdown: false,
    });
}

#[must_use]
pub fn do_dm_stdout_mode() -> bool {
    DO_DM_STDOUT.load(Ordering::Relaxed)
}

pub(crate) fn do_dm_stdout_markdown() -> bool {
    DO_DM_MARKDOWN.load(Ordering::Relaxed)
}
