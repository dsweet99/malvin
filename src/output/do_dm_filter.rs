//! Streaming extractor for `MALVIN_DM_START` / `MALVIN_DM_END` bodies.

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

/// Feed agent message text into the DM extractor; prints extracted bodies to process stdout.
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
    let Some(idx) = filter.buf.find(DM_START) else {
        keep_marker_prefix(&mut filter.buf, DM_START);
        return false;
    };
    let after = idx + DM_START.len();
    filter.buf = filter.buf[after..].to_string();
    strip_leading_newline(&mut filter.buf);
    filter.inside = true;
    true
}

fn take_inside_progress(filter: &mut DmFilter) -> bool {
    let Some(idx) = filter.buf.find(DM_END) else {
        emit_safe_inside_prefix(filter);
        return false;
    };
    let mut body = filter.buf[..idx].to_string();
    if body.ends_with('\n') {
        body.pop();
    }
    super::do_dm_emit::emit_do_dm_body(&body);
    filter.buf = filter.buf[idx + DM_END.len()..].to_string();
    strip_leading_newline(&mut filter.buf);
    filter.inside = false;
    true
}

fn strip_leading_newline(buf: &mut String) {
    if buf.starts_with('\n') {
        buf.replace_range(..1, "");
    }
}

fn emit_safe_inside_prefix(filter: &mut DmFilter) {
    let keep = marker_prefix_len(&filter.buf, DM_END);
    let emit_len = filter.buf.len().saturating_sub(keep);
    if emit_len == 0 {
        return;
    }
    let prefix = &filter.buf[..emit_len];
    let Some(last_nl) = prefix.rfind('\n') else {
        return;
    };
    let body = filter.buf[..=last_nl].to_string();
    filter.buf = filter.buf[last_nl + 1..].to_string();
    super::do_dm_emit::emit_do_dm_body(&body);
}

fn keep_marker_prefix(buf: &mut String, marker: &str) {
    let keep = marker_prefix_len(buf, marker);
    if keep < buf.len() {
        *buf = buf[buf.len() - keep..].to_string();
    }
}

fn marker_prefix_len(buf: &str, marker: &str) -> usize {
    let max = buf.len().min(marker.len().saturating_sub(1));
    (0..=max)
        .rev()
        .find(|&n| marker.starts_with(&buf[buf.len() - n..]))
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::{feed_do_dm_stdout_text, reset_do_dm_filter, DM_END, DM_START};
    use crate::output::{
        do_dm_stdout_mode, enable_stdout_capture, set_do_dm_stdout_mode, take_captured_stdout,
    };

    #[test]
    fn extracts_body_between_fences() {
        set_do_dm_stdout_mode(true);
        enable_stdout_capture();
        feed_do_dm_stdout_text(&format!("{DM_START}\nhello\n{DM_END}\n"));
        assert_eq!(take_captured_stdout(), "hello");
        set_do_dm_stdout_mode(false);
    }

    #[test]
    fn ignores_text_outside_fences() {
        set_do_dm_stdout_mode(true);
        enable_stdout_capture();
        feed_do_dm_stdout_text("noise\n");
        feed_do_dm_stdout_text(&format!("{DM_START}\nonly\n{DM_END}\n"));
        assert_eq!(take_captured_stdout(), "only");
        set_do_dm_stdout_mode(false);
    }

    #[test]
    fn streaming_chunks_across_markers() {
        set_do_dm_stdout_mode(true);
        enable_stdout_capture();
        feed_do_dm_stdout_text("MALVIN_DM_");
        feed_do_dm_stdout_text("START\nxi\nMALVIN_DM_");
        feed_do_dm_stdout_text("END\n");
        assert_eq!(take_captured_stdout(), "xi");
        set_do_dm_stdout_mode(false);
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
}
