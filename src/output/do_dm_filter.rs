use std::sync::Mutex;

use super::sentinel::{DM_END, DM_START, is_sentinel_line, may_become_sentinel_line};

#[derive(Default)]
struct DmFilter {
    inside: bool,
    buf: String,
    captured: String,
}

static FILTER: Mutex<DmFilter> = Mutex::new(DmFilter {
    inside: false,
    buf: String::new(),
    captured: String::new(),
});

pub(crate) fn reset_do_dm_filter() {
    let mut guard = FILTER
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    *guard = DmFilter::default();
}

pub fn feed_do_dm_stdout_text(text: &str) {
    if text.is_empty() {
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
    if is_sentinel_line(line, DM_START) {
        filter.buf = rest;
        filter.inside = true;
        filter.captured.clear();
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
        if is_sentinel_line(line, DM_END) {
            finish_captured_dm(filter, line_start);
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
    if !may_become_sentinel_line(buf, DM_START) {
        buf.clear();
    }
}

fn emit_safe_inside_prefix(filter: &mut DmFilter) {
    let Some(last_nl) = filter.buf.rfind('\n') else {
        return;
    };
    let body = filter.buf[..=last_nl].to_string();
    filter.buf = filter.buf[last_nl + 1..].to_string();
    filter.captured.push_str(&body);
    super::do_dm_emit::emit_do_dm_body(&body);
}

fn finish_captured_dm(filter: &mut DmFilter, line_start: usize) {
    filter.captured.push_str(&filter.buf[..line_start]);
    let mut body = std::mem::take(&mut filter.captured);
    let mut shown = filter.buf[..line_start].to_string();
    if body.ends_with('\n') {
        body.pop();
    }
    if shown.ends_with('\n') {
        shown.pop();
    }
    super::do_dm_emit::emit_do_dm_body(&shown);
    super::dm_hook::deliver_finished_dm(&body);
}

#[cfg(test)]
#[path = "do_dm_filter_tests.rs"]
mod tests;
