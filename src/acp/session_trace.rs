// Trace file setup for [`crate::acp::AcpSession::prompt`].
pub(crate) async fn trace_prepare_file(trace_path: &Path) -> Result<(), String> {
    crate::invocation::init_from_env();
    if let Some(parent) = trace_path.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .map_err(|e| format!("trace mkdir: {e}"))?;
    }
    Ok(())
}

pub(crate) async fn trace_open_truncated(
    trace_path: &Path,
) -> Result<tokio::fs::File, String> {
    tokio::fs::OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(trace_path)
        .await
        .map_err(|e| format!("trace open: {e}"))
}

pub(crate) async fn trace_write_invocation_header(
    file: &mut tokio::fs::File,
) -> Result<(), String> {
    if let Some(cmd) = crate::invocation::command_line() {
        use tokio::io::AsyncWriteExt;
        let header = format!(
            "{}\n",
            crate::output::format_line(crate::output::MALVIN_WHO, &format!("Command: {cmd}"))
        );
        file.write_all(header.as_bytes())
            .await
            .map_err(|e| format!("trace header write: {e}"))?;
        file.flush()
            .await
            .map_err(|e| format!("trace header flush: {e}"))?;
    }
    Ok(())
}

/// Whether to mirror outgoing `session/prompt` lines (`>` tags) to stdout when tee is enabled.
///
/// Always returns `false`: outgoing prompt body text is never echoed line-by-line to stdout.
/// The trace file still records full `>{stem}` lines. A single `[{stem}...]` bracket announcement
/// is printed via [`crate::output::print_outgoing_prompt_log`] after the trace write.
/// See repository root `grounding.md`, section **## Outgoing prompts**.
#[allow(dead_code)]
pub(crate) const fn acp_tee_echo_outgoing_prompt_lines(_tee_stdout: bool, _stem: &str) -> bool {
    false
}

async fn trace_write_tagged_body(
    file: &mut tokio::fs::File,
    stem: &str,
    body: &str,
) -> Result<(), String> {
    use tokio::io::AsyncWriteExt;
    let tag_raw = crate::output::format_acp_directional_tag_prefix('>', stem);
    for line in crate::output::logical_lines(body) {
        let l = crate::output::format_line(&tag_raw, line);
        file.write_all(l.as_bytes())
            .await
            .map_err(|e| format!("trace outgoing prompt write: {e}"))?;
        file.write_all(b"\n")
            .await
            .map_err(|e| format!("trace outgoing prompt newline: {e}"))?;
    }
    Ok(())
}

#[allow(clippy::struct_field_names)]
pub(crate) struct DoOutgoingTraceParts<'a> {
    pub style_text: Option<&'a str>,
    pub header_text: &'a str,
    pub user_text: &'a str,
}

/// `malvin do`: disk trace matches the full prompt (style, then `header.md`, then user request);
/// stdout announces each segment with a `[{stem}...]` bracket line (no body tee).
pub(crate) async fn trace_write_outgoing_prompt_do(
    file: &mut tokio::fs::File,
    parts: DoOutgoingTraceParts<'_>,
) -> Result<(), String> {
    use tokio::io::AsyncWriteExt;
    let DoOutgoingTraceParts {
        style_text,
        header_text,
        user_text,
    } = parts;
    if let Some(s) = style_text.filter(|t| !t.trim().is_empty()) {
        trace_write_tagged_body(file, "style", s.trim()).await?;
        crate::output::print_outgoing_prompt_log("style");
    }
    trace_write_tagged_body(file, "header", header_text).await?;
    crate::output::print_outgoing_prompt_log("header");
    trace_write_tagged_body(file, "prompt", user_text).await?;
    crate::output::print_outgoing_prompt_log("prompt");
    file.flush()
        .await
        .map_err(|e| format!("trace outgoing prompt flush: {e}"))?;
    Ok(())
}

/// Log the exact prompt text sent on `session/prompt` with an outgoing (`>`) tag per line.
pub(crate) async fn trace_write_outgoing_prompt(
    file: &mut tokio::fs::File,
    stem: &str,
    prompt_text: &str,
) -> Result<(), String> {
    use tokio::io::AsyncWriteExt;
    trace_write_tagged_body(file, stem, prompt_text).await?;
    file.flush()
        .await
        .map_err(|e| format!("trace outgoing prompt flush: {e}"))?;
    Ok(())
}

#[test]
fn acp_tee_echo_outgoing_always_false() {
    assert!(!acp_tee_echo_outgoing_prompt_lines(true, "learn"));
    assert!(!acp_tee_echo_outgoing_prompt_lines(true, "implement"));
    assert!(!acp_tee_echo_outgoing_prompt_lines(false, "learn"));
    assert!(!acp_tee_echo_outgoing_prompt_lines(true, "style"));
    assert!(!acp_tee_echo_outgoing_prompt_lines(true, "header"));
    assert!(!acp_tee_echo_outgoing_prompt_lines(true, "prompt"));
}

#[test]
fn kiss_stringify_session_trace() {
    let _ = stringify!(trace_prepare_file);
    let _ = stringify!(trace_open_truncated);
    let _ = stringify!(trace_write_invocation_header);
    let _ = stringify!(acp_tee_echo_outgoing_prompt_lines);
    let _ = stringify!(trace_write_outgoing_prompt);
    let _ = stringify!(trace_write_outgoing_prompt_do);
    let _ = stringify!(DoOutgoingTraceParts);
}
