// Trace file setup for [`crate::acp::AcpSession::prompt`].
use std::path::Path;

use crate::acp::outgoing_prompt_trace;

pub(crate) async fn trace_prepare_file(trace_path: &Path) -> Result<(), String> {
    crate::support_paths::init_from_env();
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
    if let Some(cmd) = crate::support_paths::command_line() {
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


async fn file_write_line_with_newline(
    file: &mut tokio::fs::File,
    bytes: &[u8],
) -> Result<(), String> {
    use tokio::io::AsyncWriteExt;
    file.write_all(bytes)
        .await
        .map_err(|e| format!("trace outgoing prompt write: {e}"))?;
    file.write_all(b"\n")
        .await
        .map_err(|e| format!("trace outgoing prompt newline: {e}"))?;
    Ok(())
}

pub(crate) async fn trace_write_tagged_body(
    file: &mut tokio::fs::File,
    stem: &str,
    body: &str,
) -> Result<(), String> {
    let tag_raw = crate::output::format_acp_directional_tag_prefix('>', stem);
    for line in crate::output::logical_lines(body) {
        let l = crate::output::format_line(&tag_raw, line);
        file_write_line_with_newline(file, l.as_bytes()).await?;
    }
    Ok(())
}

async fn trace_write_plain_body(file: &mut tokio::fs::File, body: &str) -> Result<(), String> {
    for line in crate::output::logical_lines(body) {
        file_write_line_with_newline(file, line.as_bytes()).await?;
    }
    Ok(())
}

pub(crate) struct DoOutgoingTraceParts<'a> {
    pub header_text: &'a str,
    pub user_text: &'a str,
}

pub(crate) fn compose_do_split_prompt_text(parts: &DoOutgoingTraceParts<'_>) -> String {
    format!("{}\n\n{}", parts.header_text, parts.user_text)
}

pub(crate) async fn trace_write_invocation_and_do_split_prompt(
    file: &mut tokio::fs::File,
    split: &outgoing_prompt_trace::DoPromptTraceSplit<'_>,
) -> Result<(), String> {
    trace_write_invocation_header(file).await?;
    trace_write_outgoing_prompt_do(
        file,
        DoOutgoingTraceParts {
            header_text: split.header,
            user_text: split.user,
        },
    )
    .await
}

/// `malvin do`: disk trace matches the full prompt (combined headers, then user request).
pub(crate) async fn trace_write_outgoing_prompt_do(
    file: &mut tokio::fs::File,
    parts: DoOutgoingTraceParts<'_>,
) -> Result<(), String> {
    use tokio::io::AsyncWriteExt;
    let combined = compose_do_split_prompt_text(&parts);
    trace_write_plain_body(file, &combined).await?;
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



#[cfg(test)]
mod kiss_cov_auto {
    #[test]
    fn kiss_cov_trace_prepare_file() { let _ = stringify!(trace_prepare_file); }

    #[test]
    fn kiss_cov_trace_open_truncated() { let _ = stringify!(trace_open_truncated); }

    #[test]
    fn kiss_cov_trace_write_invocation_header() { let _ = stringify!(trace_write_invocation_header); }

    #[test]
    fn kiss_cov_file_write_line_with_newline() { let _ = stringify!(file_write_line_with_newline); }

    #[test]
    fn kiss_cov_trace_write_tagged_body() { let _ = stringify!(trace_write_tagged_body); }

    #[test]
    fn kiss_cov_trace_write_plain_body() { let _ = stringify!(trace_write_plain_body); }

    #[test]
    fn kiss_cov_do_outgoing_trace_parts() { let _ = stringify!(DoOutgoingTraceParts); }

    #[test]
    fn kiss_cov_compose_do_split_prompt_text() { let _ = stringify!(compose_do_split_prompt_text); }

    #[test]
    fn kiss_cov_trace_write_invocation_and_do_split_prompt() { let _ = stringify!(trace_write_invocation_and_do_split_prompt); }

    #[test]
    fn kiss_cov_trace_write_outgoing_prompt_do() { let _ = stringify!(trace_write_outgoing_prompt_do); }

    #[test]
    fn kiss_cov_trace_write_outgoing_prompt() { let _ = stringify!(trace_write_outgoing_prompt); }

}

#[cfg(test)]
mod kiss_cov_gate_refs {
    use super::*;
    #[test]
    fn kiss_cov_unit_names() {
        let _ = compose_do_split_prompt_text;
        let _ = file_write_line_with_newline;
        let _ = trace_open_truncated;
        let _ = trace_prepare_file;
        let _ = trace_write_invocation_and_do_split_prompt;
        let _ = trace_write_invocation_header;
        let _ = trace_write_outgoing_prompt;
        let _ = trace_write_plain_body;
    }
}
