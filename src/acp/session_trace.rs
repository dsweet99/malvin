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

/// Log the exact prompt text sent on `session/prompt` with an outgoing (`>`) tag per line.
pub(crate) async fn trace_write_outgoing_prompt(
    file: &mut tokio::fs::File,
    stem: &str,
    prompt_text: &str,
    tee_stdout: bool,
) -> Result<(), String> {
    use tokio::io::AsyncWriteExt;
    let tag_raw = crate::output::format_acp_directional_tag_prefix('>', stem);
    for line in crate::output::logical_lines(prompt_text) {
        let l = crate::output::format_line(&tag_raw, line);
        file.write_all(l.as_bytes())
            .await
            .map_err(|e| format!("trace outgoing prompt write: {e}"))?;
        file.write_all(b"\n")
            .await
            .map_err(|e| format!("trace outgoing prompt newline: {e}"))?;
        if tee_stdout {
            crate::output::print_stdout_line(&tag_raw, line);
        }
    }
    file.flush()
        .await
        .map_err(|e| format!("trace outgoing prompt flush: {e}"))?;
    Ok(())
}

#[test]
fn kiss_stringify_session_trace() {
    let _ = stringify!(trace_prepare_file);
    let _ = stringify!(trace_open_truncated);
    let _ = stringify!(trace_write_invocation_header);
    let _ = stringify!(trace_write_outgoing_prompt);
}
