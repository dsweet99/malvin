fn format_tool_line(
    parsed: &ParsedToolUpdate,
    tracker: &ToolSummaryTracker,
    detail: ToolSummaryDetail,
) -> String {
    let rec = tracker.record(&parsed.id);
    let kind = rec.map_or(parsed.kind.as_str(), |r| r.kind.as_str());
    let mut parts = vec![
        "[tool]".to_string(),
        tool_phase_label(parsed.phase, parsed.status.as_deref()),
        format!("kind={kind}"),
        format!("id={}", parsed.id),
    ];
    if parsed.phase == TOOL_PHASE_RUNNING {
        if let Some(rec) = rec {
            append_elapsed(&mut parts, rec);
        }
    } else if parsed.phase == TOOL_PHASE_START || parsed.phase == TOOL_PHASE_PENDING {
        append_start_title(&mut parts, parsed, rec);
    }
    if parsed.phase == TOOL_PHASE_DONE {
        if let Some(status) = parsed.status.as_deref() {
            parts.push(format!("status={status}"));
        }
        if let Some(rec) = rec {
            append_elapsed(&mut parts, rec);
        }
        append_done_fields(&mut parts, parsed, tracker, detail);
    }
    parts.join(" ")
}

fn start_label<'a>(parsed: &'a ParsedToolUpdate, rec: Option<&'a ToolCallRecord>) -> &'a str {
    if let Some(command) = parsed.command.as_deref() {
        return command;
    }
    if let Some(stripped) = parsed
        .title
        .strip_prefix('`')
        .and_then(|t| t.strip_suffix('`'))
        .filter(|t| !t.is_empty())
    {
        return stripped;
    }
    if !parsed.title.is_empty() {
        return &parsed.title;
    }
    rec.map_or("", |r| r.title.as_str())
}

fn append_elapsed(parts: &mut Vec<String>, rec: &ToolCallRecord) {
    let elapsed = rec.started.elapsed();
    let secs = elapsed.as_secs();
    let tenths = elapsed.subsec_millis() / 100;
    parts.push(format!("elapsed={secs}.{tenths}s"));
}

fn append_start_title(
    parts: &mut Vec<String>,
    parsed: &ParsedToolUpdate,
    rec: Option<&ToolCallRecord>,
) {
    let label = start_label(parsed, rec);
    if label.is_empty() {
        return;
    }
    let short = shorten_middle(label, TOOL_DISPLAY_MAX_WIDTH);
    parts.push(format!("title=\"{}\"", escape_quoted(&short)));
}

fn append_done_fields(
    parts: &mut Vec<String>,
    parsed: &ParsedToolUpdate,
    tracker: &ToolSummaryTracker,
    detail: ToolSummaryDetail,
) {
    let Some(raw) = parsed.raw_output.as_ref() else {
        return;
    };
    let kind = tracker
        .record(&parsed.id)
        .map_or(parsed.kind.as_str(), |r| r.kind.as_str());
    match kind {
        "execute" => append_execute_done(parts, raw, detail),
        "read" => append_read_done(parts, raw, detail),
        "search" => append_search_done(parts, raw),
        "edit" => append_edit_done(parts, raw),
        _ => append_generic_done(parts, raw, detail),
    }
}

fn append_execute_done(parts: &mut Vec<String>, raw: &Value, detail: ToolSummaryDetail) {
    if let Some(code) = raw.get("exitCode").and_then(Value::as_i64) {
        parts.push(format!("exit={code}"));
    }
    if detail == ToolSummaryDetail::Log {
        append_byte_fields(parts, raw);
        append_error_headline(parts, raw);
    }
}

fn append_read_done(parts: &mut Vec<String>, raw: &Value, detail: ToolSummaryDetail) {
    if let Some(content) = raw.get("content").and_then(Value::as_str) {
        parts.push(format!("output={}B", content.len()));
    } else if detail == ToolSummaryDetail::Log {
        append_byte_fields(parts, raw);
    }
}

fn append_search_done(parts: &mut Vec<String>, raw: &Value) {
    if let Some(n) = raw
        .get("totalMatches")
        .or_else(|| raw.get("resultCount"))
        .and_then(json_number)
    {
        parts.push(format!("matches={n}"));
    }
    if let Some(truncated) = raw.get("truncated").and_then(Value::as_bool) {
        parts.push(format!("truncated={truncated}"));
    }
}

fn push_edit_path(parts: &mut Vec<String>, raw: &Value, path: &str) {
    let short = shorten_middle(path, TOOL_DISPLAY_MAX_WIDTH);
    if let Some(line) = raw
        .get("lineNumber")
        .or_else(|| raw.get("startLine"))
        .and_then(json_number)
    {
        parts.push(format!("path={short}:{line}"));
    } else {
        parts.push(format!("path={short}"));
    }
}

fn append_edit_done(parts: &mut Vec<String>, raw: &Value) {
    if let Some(paths) = edit_paths(raw) {
        for path in paths {
            push_edit_path(parts, raw, &path);
        }
    }
    if let Some(files) = raw.get("totalFiles").and_then(json_number) {
        parts.push(format!("files={files}"));
    } else if raw.get("content").is_some() {
        parts.push("files=1".to_string());
    } else if let Some(paths) = edit_paths(raw) {
        parts.push(format!("files={}", paths.len()));
    }
    append_edit_counts(parts, raw);
}

fn append_edit_counts(parts: &mut Vec<String>, raw: &Value) {
    let added = raw
        .get("linesAdded")
        .or_else(|| raw.get("added"))
        .and_then(json_number);
    let removed = raw
        .get("linesRemoved")
        .or_else(|| raw.get("removed"))
        .and_then(json_number);
    match (added, removed) {
        (Some(a), Some(r)) => parts.push(format!("added={a} removed={r}")),
        (Some(a), None) => parts.push(format!("added={a}")),
        (None, Some(r)) => parts.push(format!("removed={r}")),
        (None, None) => {}
    }
}

fn append_generic_done(parts: &mut Vec<String>, raw: &Value, detail: ToolSummaryDetail) {
    if let Some(content) = raw.get("content").and_then(Value::as_str) {
        parts.push(format!("output={}B", content.len()));
    } else if detail == ToolSummaryDetail::Log {
        append_byte_fields(parts, raw);
        append_error_headline(parts, raw);
    }
}

fn edit_paths(raw: &Value) -> Option<Vec<String>> {
    let mut paths = Vec::new();
    if let Some(p) = raw.get("path").or_else(|| raw.get("filePath")).and_then(Value::as_str) {
        paths.push(p.to_string());
    }
    if let Some(arr) = raw.get("paths").and_then(Value::as_array) {
        for item in arr {
            if let Some(s) = item.as_str() {
                paths.push(s.to_string());
            }
        }
    }
    if paths.is_empty() {
        None
    } else {
        Some(paths)
    }
}

fn append_byte_fields(parts: &mut Vec<String>, raw: &Value) {
    if let Some(stdout) = raw.get("stdout").and_then(Value::as_str) {
        parts.push(format!("stdout={}B", stdout.len()));
    }
    if let Some(stderr) = raw.get("stderr").and_then(Value::as_str) {
        parts.push(format!("stderr={}B", stderr.len()));
    }
}

fn append_error_headline(parts: &mut Vec<String>, raw: &Value) {
    let headline = stderr_headline(raw).or_else(|| stdout_headline(raw));
    if let Some(line) = headline {
        let short = shorten_middle(line, TOOL_DISPLAY_MAX_WIDTH);
        parts.push(format!("error=\"{}\"", escape_quoted(&short)));
    }
}

fn stderr_headline(raw: &Value) -> Option<&str> {
    raw.get("stderr")
        .and_then(Value::as_str)
        .and_then(first_non_empty_line)
}

fn stdout_headline(raw: &Value) -> Option<&str> {
    raw.get("stdout")
        .and_then(Value::as_str)
        .and_then(first_non_empty_line)
}

fn first_non_empty_line(s: &str) -> Option<&str> {
    for line in s.lines() {
        let trimmed = line.trim();
        if !trimmed.is_empty() {
            return Some(trimmed);
        }
    }
    None
}

fn json_number(v: &Value) -> Option<u64> {
    if let Some(n) = v.as_u64() {
        return Some(n);
    }
    if let Some(n) = v.as_i64() {
        return u64::try_from(n).ok();
    }
    if let Some(s) = v.as_str() {
        return s.parse().ok();
    }
    None
}

fn escape_quoted(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}
