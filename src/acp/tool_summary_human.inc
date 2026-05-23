const TOOL_STDOUT_FAST_COLLAPSE_MS: u128 = 300;
const TOOL_STDOUT_RUNNING_MIN_MS: u128 = 1000;

const ANSI_BOLD: &str = "\x1b[1m";
const ANSI_CYAN: &str = "\x1b[36m";
const ANSI_DIM: &str = "\x1b[90m";
const ANSI_GREEN: &str = "\x1b[32m";
const ANSI_RED: &str = "\x1b[31m";
const ANSI_RESET: &str = "\x1b[0m";

fn format_tool_stdout(
    parsed: &ParsedToolUpdate,
    tracker: &mut ToolSummaryTracker,
    stdout_deferred: &mut Option<String>,
) -> Option<String> {
    let elapsed = tracker.record(&parsed.id)?.started.elapsed();
    let elapsed_ms = elapsed.as_millis();
    if !tool_stdout_should_emit(parsed, elapsed_ms) {
        return None;
    }
    let kind = tracker
        .record(&parsed.id)
        .map_or(parsed.kind.as_str(), |r| r.kind.as_str());
    if parsed.phase == TOOL_PHASE_DONE {
        let collapsed = elapsed_ms < TOOL_STDOUT_FAST_COLLAPSE_MS;
        if !collapsed {
            let rec = tracker.record(&parsed.id)?;
            if !rec.stdout_start_emitted {
                *stdout_deferred = human_start_line(parsed, tracker, kind);
                if let Some(rec) = tracker.record_mut(&parsed.id) {
                    rec.stdout_start_emitted = true;
                }
            }
        }
    }
    let line = format_tool_line_human(parsed, tracker, elapsed)?;
    if parsed.phase == TOOL_PHASE_RUNNING || parsed.phase == TOOL_PHASE_PENDING {
        if let Some(rec) = tracker.record_mut(&parsed.id) {
            rec.stdout_start_emitted = true;
        }
    }
    Some(line)
}

const fn tool_stdout_should_emit(parsed: &ParsedToolUpdate, elapsed_ms: u128) -> bool {
    match parsed.phase {
        TOOL_PHASE_DONE => true,
        TOOL_PHASE_RUNNING => elapsed_ms >= TOOL_STDOUT_RUNNING_MIN_MS,
        TOOL_PHASE_PENDING => true,
        TOOL_PHASE_START | TOOL_PHASE_NAMED_STATUS => false,
        _ => false,
    }
}

fn format_tool_line_human(
    parsed: &ParsedToolUpdate,
    tracker: &ToolSummaryTracker,
    elapsed: std::time::Duration,
) -> Option<String> {
    let kind = tracker
        .record(&parsed.id)
        .map_or(parsed.kind.as_str(), |r| r.kind.as_str());
    match parsed.phase {
        TOOL_PHASE_DONE => {
            let collapsed = elapsed.as_millis() < TOOL_STDOUT_FAST_COLLAPSE_MS;
            human_done_line(parsed, tracker, kind, elapsed, collapsed)
        }
        TOOL_PHASE_RUNNING => human_running_line(parsed, tracker, kind, elapsed),
        TOOL_PHASE_START | TOOL_PHASE_PENDING => human_start_line(parsed, tracker, kind),
        _ => None,
    }
}

fn human_start_line(
    parsed: &ParsedToolUpdate,
    tracker: &ToolSummaryTracker,
    kind: &str,
) -> Option<String> {
    match kind {
        "read" => human_read_subject(parsed, tracker, false).map(|subject| format!("Reading {subject}…")),
        "search" => Some(human_search_start(parsed)),
        "execute" => {
            let cmd = human_execute_command(parsed, tracker);
            Some(format!("Run {cmd}…"))
        }
        "edit" => human_edit_subject(parsed, tracker, false).map(|subject| format!("Editing {subject}…")),
        _ => None,
    }
}

fn human_running_line(
    parsed: &ParsedToolUpdate,
    tracker: &ToolSummaryTracker,
    kind: &str,
    elapsed: std::time::Duration,
) -> Option<String> {
    let rec = tracker.record(&parsed.id)?;
    if !rec.stdout_start_emitted {
        return human_start_line(parsed, tracker, kind);
    }
    let dur = humanize_duration(elapsed);
    match kind {
        "read" => human_read_subject(parsed, tracker, false)
            .map(|subject| format!("Reading {subject} · {dur}")),
        "search" => Some(format!("Searching · {dur}")),
        "execute" => Some(format!(
            "Run {} · {dur}",
            human_execute_command(parsed, tracker)
        )),
        "edit" => human_edit_subject(parsed, tracker, false)
            .map(|subject| format!("Editing {subject} · {dur}")),
        _ => None,
    }
}

fn human_done_line(
    parsed: &ParsedToolUpdate,
    tracker: &ToolSummaryTracker,
    kind: &str,
    elapsed: std::time::Duration,
    _collapsed: bool,
) -> Option<String> {
    match kind {
        "read" => Some(human_read_done(parsed, tracker, elapsed)),
        "search" => Some(human_search_done(parsed, elapsed)),
        "execute" => Some(human_execute_done(parsed, tracker, elapsed)),
        "edit" => Some(human_edit_done(parsed, tracker, elapsed)),
        _ => None,
    }
}

fn human_read_done(
    parsed: &ParsedToolUpdate,
    tracker: &ToolSummaryTracker,
    elapsed: std::time::Duration,
) -> String {
    let subject = human_read_subject(parsed, tracker, true).unwrap_or_else(|| "file".to_string());
    let dur = humanize_duration(elapsed);
    let Some(raw) = parsed.raw_output.as_ref() else {
        return format!("Read {subject} · {dur}");
    };
    let size = raw
        .get("content")
        .and_then(Value::as_str)
        .map(|c| humanize_bytes(c.len()))
        .or_else(|| raw_byte_size(raw).map(humanize_bytes));
    size.map_or_else(
        || format!("Read {subject} · {dur}"),
        |size| format!("Read {subject} · {size} · {dur}"),
    )
}

fn human_search_start(parsed: &ParsedToolUpdate) -> String {
    if let Some(q) = parsed.search_query.as_deref().filter(|s| !s.is_empty()) {
        return format!("Searching {}…", shorten_middle(q, TOOL_DISPLAY_MAX_WIDTH));
    }
    "Searching…".to_string()
}

fn human_search_done(parsed: &ParsedToolUpdate, _elapsed: std::time::Duration) -> String {
    let Some(raw) = parsed.raw_output.as_ref() else {
        return "Search · matches".to_string();
    };
    let mut line = raw
        .get("totalMatches")
        .or_else(|| raw.get("resultCount"))
        .and_then(json_number)
        .map_or_else(
            || "Search · matches".to_string(),
            |n| format!("Search · {n} matches"),
        );
    if raw.get("truncated").and_then(Value::as_bool) == Some(true) {
        line.push_str(" (truncated)");
    }
    line
}

fn execute_effective_exit(parsed: &ParsedToolUpdate, raw: Option<&Value>) -> i64 {
    if let Some(code) = raw.and_then(|r| r.get("exitCode")).and_then(Value::as_i64) {
        return code;
    }
    match parsed.status.as_deref() {
        Some("failed" | "cancelled") => 1,
        _ => 0,
    }
}

fn execute_stdout_failed(parsed: &ParsedToolUpdate, exit: i64, raw: Option<&Value>) -> bool {
    exit != 0
        || matches!(parsed.status.as_deref(), Some("failed" | "cancelled"))
        || raw.and_then(stderr_headline).is_some()
}

fn human_execute_done(
    parsed: &ParsedToolUpdate,
    tracker: &ToolSummaryTracker,
    elapsed: std::time::Duration,
) -> String {
    let cmd = human_execute_command(parsed, tracker);
    let dur = humanize_duration(elapsed);
    let raw = parsed.raw_output.as_ref();
    let exit = execute_effective_exit(parsed, raw);
    if execute_stdout_failed(parsed, exit, raw) {
        let mut line = format!("Run {cmd} · {dur} · ✗ exit {exit}");
        if let Some(r) = raw {
            if let Some(err) = stderr_headline(r).or_else(|| stdout_headline(r)) {
                let short = shorten_middle(err, TOOL_DISPLAY_MAX_WIDTH);
                line.push_str(" · ");
                line.push_str(&short);
            }
        }
        return line;
    }
    format!("Run {cmd} · {dur} · ✓")
}

fn human_edit_done(
    parsed: &ParsedToolUpdate,
    tracker: &ToolSummaryTracker,
    elapsed: std::time::Duration,
) -> String {
    let subject = human_edit_subject(parsed, tracker, true).unwrap_or_else(|| "file".to_string());
    let dur = humanize_duration(elapsed);
    let Some(raw) = parsed.raw_output.as_ref() else {
        return format!("Edit {subject} · {dur}");
    };
    let counts = human_edit_counts(raw);
    if counts.is_empty() {
        return format!("Edit {subject} · {dur}");
    }
    format!("Edit {subject} · {counts} · {dur}")
}

fn human_edit_counts(raw: &Value) -> String {
    let added = raw
        .get("linesAdded")
        .or_else(|| raw.get("added"))
        .and_then(json_number);
    let removed = raw
        .get("linesRemoved")
        .or_else(|| raw.get("removed"))
        .and_then(json_number);
    match (added, removed) {
        (Some(a), Some(r)) => format!("+{a}/−{r}"),
        (Some(a), None) => format!("+{a}"),
        (None, Some(r)) => format!("−{r}"),
        (None, None) => String::new(),
    }
}

fn human_read_subject(
    parsed: &ParsedToolUpdate,
    tracker: &ToolSummaryTracker,
    allow_generic: bool,
) -> Option<String> {
    let rec = tracker.record(&parsed.id);
    if let Some(path) = parsed
        .input_path
        .as_deref()
        .or_else(|| rec.and_then(|r| r.input_path.as_deref()))
        .or_else(|| parsed.raw_output.as_ref().and_then(read_output_path))
    {
        let line_range = parsed
            .input_line_range
            .or_else(|| rec.and_then(|r| r.input_line_range));
        return Some(shorten_subject_path(path, line_range));
    }
    if let Some(label) = read_or_edit_title_label(parsed, rec, "Read") {
        return Some(shorten_middle(&label, TOOL_DISPLAY_MAX_WIDTH));
    }
    allow_generic.then(|| "file".to_string())
}

fn read_output_path(raw: &Value) -> Option<&str> {
    raw.get("path")
        .or_else(|| raw.get("filePath"))
        .and_then(Value::as_str)
}

fn human_edit_subject(
    parsed: &ParsedToolUpdate,
    tracker: &ToolSummaryTracker,
    allow_generic: bool,
) -> Option<String> {
    let rec = tracker.record(&parsed.id);
    if let Some(paths) = parsed.raw_output.as_ref().and_then(edit_paths) {
        if paths.len() == 1 {
            let line_range = parsed
                .input_line_range
                .or_else(|| rec.and_then(|r| r.input_line_range))
                .or_else(|| parsed.raw_output.as_ref().and_then(|raw| acp_line_range_field(Some(raw))));
            return Some(shorten_subject_path(&paths[0], line_range));
        }
        return Some(format!("{} files", paths.len()));
    }
    if let Some(path) = parsed
        .input_path
        .as_deref()
        .or_else(|| rec.and_then(|r| r.input_path.as_deref()))
    {
        let line_range = parsed
            .input_line_range
            .or_else(|| rec.and_then(|r| r.input_line_range));
        return Some(shorten_subject_path(path, line_range));
    }
    if let Some(label) = read_or_edit_title_label(parsed, rec, "Edit") {
        return Some(shorten_middle(&label, TOOL_DISPLAY_MAX_WIDTH));
    }
    allow_generic.then(|| "file".to_string())
}

fn human_execute_command(parsed: &ParsedToolUpdate, tracker: &ToolSummaryTracker) -> String {
    let rec = tracker.record(&parsed.id);
    let label = parsed
        .command
        .as_deref()
        .or_else(|| rec.and_then(|r| r.command.as_deref()))
        .unwrap_or_else(|| start_label(parsed, rec));
    let stripped = strip_execute_cd_prefix(label);
    shorten_middle(&escape_tool_subject_fragment(stripped), TOOL_DISPLAY_MAX_WIDTH)
}

fn shorten_subject_path(path: &str, line_range: Option<LineRange>) -> String {
    let suffix = format_line_range_suffix(line_range);
    let suffix_chars = suffix.chars().count();
    let min_path = 8usize;
    let path_width = TOOL_DISPLAY_MAX_WIDTH.saturating_sub(suffix_chars).max(min_path);
    let short_path = shorten_middle(path, path_width);
    format!("{short_path}{suffix}")
}

fn format_line_range_suffix(line_range: Option<LineRange>) -> String {
    let Some(range) = line_range else {
        return String::new();
    };
    range.end.map_or_else(
        || format!(":{}", range.start),
        |end| format!(":{}-{}", range.start, end),
    )
}

fn read_or_edit_title_label(
    parsed: &ParsedToolUpdate,
    rec: Option<&ToolCallRecord>,
    verb: &str,
) -> Option<String> {
    let label = start_label(parsed, rec).trim();
    if label.is_empty() {
        return None;
    }
    let mut stripped = label;
    if let Some(rest) = stripped.strip_prefix(verb) {
        stripped = rest.trim_start();
    }
    if stripped.is_empty() {
        return None;
    }
    if looks_like_path_label(stripped) {
        return Some(stripped.to_string());
    }
    None
}

fn looks_like_path_label(s: &str) -> bool {
    s.contains('/') || s.contains('\\')
}

fn escape_tool_subject_fragment(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            _ => out.push(ch),
        }
    }
    out
}

fn strip_execute_cd_prefix(cmd: &str) -> &str {
    let trimmed = cmd.trim();
    let Some(rest) = trimmed.strip_prefix("cd ") else {
        return trimmed;
    };
    if let Some(idx) = rest.find(" && ") {
        return rest[idx + 4..].trim_start();
    }
    if let Some(idx) = rest.find(';') {
        return rest[idx + 1..].trim_start();
    }
    trimmed
}

fn raw_byte_size(raw: &Value) -> Option<usize> {
    raw.get("content")
        .and_then(Value::as_str)
        .map(str::len)
        .or_else(|| raw.get("stdout").and_then(Value::as_str).map(str::len))
}

fn humanize_bytes(n: usize) -> String {
    if n < 1024 {
        format!("{n} B")
    } else if n < 1024 * 1024 {
        format!("{} KB", n.div_ceil(1024))
    } else {
        format!("{} MB", n.div_ceil(1024 * 1024))
    }
}

fn humanize_duration(elapsed: std::time::Duration) -> String {
    let ms = elapsed.as_millis();
    if ms < 1000 {
        return format!("{ms}ms");
    }
    let secs = elapsed.as_secs();
    let tenths = elapsed.subsec_millis() / 100;
    format!("{secs}.{tenths}s")
}

pub fn tool_summary_stdout_display(plain: &str) -> String {
    if !crate::output::stdout_use_color() {
        return plain.to_string();
    }
    apply_tool_summary_ansi(plain)
}

fn apply_tool_summary_ansi(plain: &str) -> String {
    use std::fmt::Write as _;
    let mut out = String::new();
    let mut rest = plain;
    while let Some(idx) = rest.find('·') {
        let (left, right) = rest.split_at(idx);
        out.push_str(&ansi_style_tool_segment(left));
        let _ = write!(out, "{ANSI_DIM}·{ANSI_RESET}");
        rest = right.trim_start_matches('·').trim_start();
    }
    out.push_str(&ansi_style_tool_segment(rest));
    out
}

fn ansi_style_tool_segment(seg: &str) -> String {
    let seg = seg.trim();
    if seg.is_empty() {
        return String::new();
    }
    if seg.contains('✓') {
        return seg.replace('✓', &format!("{ANSI_GREEN}✓{ANSI_RESET}"));
    }
    if seg.contains('✗') {
        return seg.replace('✗', &format!("{ANSI_RED}✗{ANSI_RESET}"));
    }
    if seg.ends_with('…') || seg.starts_with("Reading ") || seg.starts_with("Run ") || seg.starts_with("Editing ") || seg == "Searching…"
    {
        let verb_end = seg.find(' ').unwrap_or(seg.len());
        let (verb, tail) = seg.split_at(verb_end);
        return format!(
            "{ANSI_BOLD}{verb}{ANSI_RESET}{}",
            ansi_style_path_tail(tail)
        );
    }
    if seg.starts_with("Read ") || seg.starts_with("Edit ") || seg.starts_with("Search ") {
        let rest = seg.split_once(' ').map_or(seg, |(_, r)| r);
        return format!("{}{}", &seg[..seg.len() - rest.len()], ansi_style_path_tail(rest));
    }
    if seg.contains("matches") || seg.contains("exit ") {
        return format!("{ANSI_DIM}{seg}{ANSI_RESET}");
    }
    ansi_style_path_tail(seg)
}

fn ansi_style_path_tail(seg: &str) -> String {
    if seg.chars().any(|c| c == '/' || c == '.') {
        return format!("{ANSI_CYAN}{seg}{ANSI_RESET}");
    }
    format!("{ANSI_DIM}{seg}{ANSI_RESET}")
}
