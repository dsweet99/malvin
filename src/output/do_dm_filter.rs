
use std::sync::Mutex;

pub const DM_START: &str = "MALVIN_DM_START";
pub const DM_END: &str = "MALVIN_DM_END";

#[derive(Default)]
struct DmFilter {
    inside: bool,
    buf: String,
}

static FILTER: Mutex<DmFilter> = Mutex::new(DmFilter {
    inside: false,
    buf: String::new(),
});

pub(crate) fn reset_do_dm_filter() {
    let mut guard = FILTER
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    *guard = DmFilter::default();
}

pub fn feed_do_dm_stdout_text(text: &str) {
    if !super::do_dm_mode::do_dm_stdout_mode() || text.is_empty() {
        return;
    }
    let mut filter = take_filter_appending(text);
    drain_do_dm_filter(&mut filter);
    store_filter(filter);
}

fn take_filter_appending(text: &str) -> DmFilter {
    let mut guard = FILTER
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    guard.buf.push_str(text);
    std::mem::take(&mut *guard)
}

fn store_filter(mut filter: DmFilter) {
    let mut guard = FILTER
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    if !guard.buf.is_empty() {
        filter.buf.push_str(&guard.buf);
    }
    *guard = filter;
}

fn drain_do_dm_filter(filter: &mut DmFilter) {
    loop {
        let progressed = if filter.inside {
            take_inside_progress(filter)
        } else {
            take_outside_progress(filter)
        };
        if !progressed {
            return;
        }
    }
}

fn take_outside_progress(filter: &mut DmFilter) -> bool {
    let Some(nl) = filter.buf.find('\n') else {
        keep_outside_incomplete(&mut filter.buf);
        return false;
    };
    let line = &filter.buf[..nl];
    let rest = filter.buf[nl + 1..].to_string();
    if line == DM_START {
        filter.buf = rest;
        filter.inside = true;
        return true;
    }
    filter.buf = rest;
    true
}

fn take_inside_progress(filter: &mut DmFilter) -> bool {
    let mut line_start = 0usize;
    while let Some(rel) = filter.buf[line_start..].find('\n') {
        let nl = line_start + rel;
        let line = &filter.buf[line_start..nl];
        if line == DM_END {
            let mut body = filter.buf[..line_start].to_string();
            if body.ends_with('\n') {
                body.pop();
            }
            super::do_dm_emit::emit_do_dm_body(&body);
            filter.buf = filter.buf[nl + 1..].to_string();
            filter.inside = false;
            return true;
        }
        line_start = nl + 1;
    }
    emit_safe_inside_prefix(filter);
    false
}

fn keep_outside_incomplete(buf: &mut String) {
    if !DM_START.starts_with(buf.as_str()) {
        buf.clear();
    }
}

fn emit_safe_inside_prefix(filter: &mut DmFilter) {
    let Some(last_nl) = filter.buf.rfind('\n') else {
        return;
    };
    let body = filter.buf[..=last_nl].to_string();
    filter.buf = filter.buf[last_nl + 1..].to_string();
    super::do_dm_emit::emit_do_dm_body(&body);
}

#[cfg(test)]
mod tests {
    use super::{feed_do_dm_stdout_text, reset_do_dm_filter, DM_END, DM_START};
    use crate::output::{
        do_dm_stdout_mode, enable_stdout_capture, set_do_dm_stdout_mode, take_captured_stdout,
    };

    fn with_dm_capture(f: impl FnOnce()) -> String {
        reset_do_dm_filter();
        set_do_dm_stdout_mode(true);
        enable_stdout_capture();
        f();
        let out = take_captured_stdout();
        set_do_dm_stdout_mode(false);
        reset_do_dm_filter();
        out
    }

    #[test]
    fn extracts_body_between_fences() {
        let out = with_dm_capture(|| {
            feed_do_dm_stdout_text(&format!("{DM_START}\nhello\n{DM_END}\n"));
        });
        assert_eq!(out, "hello");
    }

    #[test]
    fn ignores_text_outside_fences() {
        let out = with_dm_capture(|| {
            feed_do_dm_stdout_text("noise\n");
            feed_do_dm_stdout_text(&format!("{DM_START}\nonly\n{DM_END}\n"));
        });
        assert_eq!(out, "only");
    }

    #[test]
    fn streaming_chunks_across_markers() {
        let out = with_dm_capture(|| {
            feed_do_dm_stdout_text("MALVIN_DM_");
            feed_do_dm_stdout_text("START\nxi\nMALVIN_DM_");
            feed_do_dm_stdout_text("END\n");
        });
        assert_eq!(out, "xi");
    }

    #[test]
    fn mode_off_is_noop() {
        set_do_dm_stdout_mode(false);
        reset_do_dm_filter();
        enable_stdout_capture();
        feed_do_dm_stdout_text(&format!("{DM_START}\nz\n{DM_END}\n"));
        assert!(take_captured_stdout().is_empty());
        assert!(!do_dm_stdout_mode());
    }

    #[test]
    fn streaming_multibyte_outside_fences_does_not_panic() {
        const OUTSIDE: &str = "Obtain the current local date and time (with timezone name or numeric offset) and report it clearly to the user—prefer a";
        let out = with_dm_capture(|| {
            feed_do_dm_stdout_text(OUTSIDE);
            feed_do_dm_stdout_text(" café 日本語\n");
            feed_do_dm_stdout_text(&format!("{DM_START}\nbody—ok\n{DM_END}\n"));
        });
        assert_eq!(out, "body—ok");
    }

    #[test]
    fn streaming_multibyte_inside_incomplete_end_does_not_panic() {
        let out = with_dm_capture(|| {
            feed_do_dm_stdout_text(&format!("{DM_START}\nline—one\n"));
            feed_do_dm_stdout_text("line—two\n");
            feed_do_dm_stdout_text(&format!("{DM_END}\n"));
        });
        assert!(out.contains("line—one"), "out={out:?}");
        assert!(out.contains("line—two"), "out={out:?}");
    }

    #[test]
    fn rejects_same_line_start_and_end() {
        let out = with_dm_capture(|| {
            feed_do_dm_stdout_text(&format!("{DM_START} hello {DM_END}\n"));
            feed_do_dm_stdout_text(&format!("{DM_START}\nreal\n{DM_END}\n"));
        });
        assert_eq!(out, "real");
    }

    #[test]
    fn rejects_non_alone_marker_forms() {
        let out = with_dm_capture(|| {
            feed_do_dm_stdout_text(&format!("x{DM_START}\nnope\n{DM_END}\n"));
            feed_do_dm_stdout_text(&format!("{DM_START}x\nnope\n{DM_END}\n"));
            feed_do_dm_stdout_text(&format!(" {DM_START}\nnope\n{DM_END}\n"));
            feed_do_dm_stdout_text(&format!("{DM_START}\nsee {DM_END}\n{DM_END}\n"));
            feed_do_dm_stdout_text(&format!("{DM_START}\nalone\n{DM_END}\n"));
        });
        assert_eq!(out, format!("see {DM_END}\nalone"));
    }

    #[test]
    fn rejects_start_and_body_on_one_line() {
        let out = with_dm_capture(|| {
            feed_do_dm_stdout_text(&format!("{DM_START} hello\n{DM_END}\n"));
            feed_do_dm_stdout_text(&format!("{DM_START}\nok\n{DM_END}\n"));
        });
        assert_eq!(out, "ok");
    }
}
