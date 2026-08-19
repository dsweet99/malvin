use std::sync::atomic::{AtomicBool, Ordering};

static DO_DM_STDOUT: AtomicBool = AtomicBool::new(false);
static DO_DM_MARKDOWN: AtomicBool = AtomicBool::new(false);

#[derive(Debug, Clone, Copy, Default)]
pub struct DoDmStdoutOpts {
    pub enabled: bool,
    pub emit_markdown: bool,
}

pub fn set_do_dm_stdout_opts(opts: DoDmStdoutOpts) {
    DO_DM_STDOUT.store(opts.enabled, Ordering::Relaxed);
    DO_DM_MARKDOWN.store(opts.enabled && opts.emit_markdown, Ordering::Relaxed);
    super::do_dm_filter::reset_do_dm_filter();
}

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
