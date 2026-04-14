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
/// For stem `learn`, the trace file still records full `>learn` lines; tee does not mirror them to
/// stdout (so learn body text is not echoed as tee lines). The `[learn...]` bracket announcement
/// is still printed via [`crate::acp::AcpSession::prompt_impl`] (`print_outgoing_prompt_log`).
/// See repository root `grounding.md`, section **## Outgoing prompts**.
#[must_use]
pub(crate) fn acp_tee_echo_outgoing_prompt_lines(tee_stdout: bool, stem: &str) -> bool {
    tee_stdout && stem != "learn"
}

#[must_use]
fn collapse_header_for_stdout_one_line(header: &str) -> String {
    header
        .lines()
        .map(str::trim_end)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

/// Tee behavior for one tagged outgoing trace segment.
#[derive(Clone, Copy)]
pub(crate) enum TaggedOutgoingTee {
    Off,
    MirrorEachLine,
    MirrorHeaderCollapsed,
}

pub(crate) struct TaggedBodyWrite<'a> {
    pub stem: &'a str,
    pub body: &'a str,
    pub tee: TaggedOutgoingTee,
}

async fn trace_write_tagged_body(
    file: &mut tokio::fs::File,
    tagged: TaggedBodyWrite<'_>,
) -> Result<(), String> {
    use tokio::io::AsyncWriteExt;
    let TaggedBodyWrite { stem, body, tee } = tagged;
    let tag_raw = crate::output::format_acp_directional_tag_prefix('>', stem);
    let echo_outgoing_to_stdout = match &tee {
        TaggedOutgoingTee::Off => false,
        TaggedOutgoingTee::MirrorEachLine | TaggedOutgoingTee::MirrorHeaderCollapsed => {
            acp_tee_echo_outgoing_prompt_lines(true, stem)
        }
    };
    let collapse_stdout = matches!(tee, TaggedOutgoingTee::MirrorHeaderCollapsed);
    for line in crate::output::logical_lines(body) {
        let l = crate::output::format_line(&tag_raw, line);
        file.write_all(l.as_bytes())
            .await
            .map_err(|e| format!("trace outgoing prompt write: {e}"))?;
        file.write_all(b"\n")
            .await
            .map_err(|e| format!("trace outgoing prompt newline: {e}"))?;
        if echo_outgoing_to_stdout && !collapse_stdout {
            crate::output::print_stdout_acp_tee_line(
                crate::output::AcpTeeDirection::ToAgent,
                &tag_raw,
                line,
            );
        }
    }
    if echo_outgoing_to_stdout && collapse_stdout && !body.trim().is_empty() {
        crate::output::print_stdout_acp_tee_line(
            crate::output::AcpTeeDirection::ToAgent,
            &tag_raw,
            &collapse_header_for_stdout_one_line(body),
        );
    }
    Ok(())
}

pub(crate) struct DoOutgoingTraceParts<'a> {
    pub style_text: Option<&'a str>,
    pub header_text: &'a str,
    pub user_text: &'a str,
    pub tee_stdout: bool,
}

/// `malvin do`: disk trace matches the full prompt (style, then `header.md`, then user request);
/// stdout echoes the `header.md` block as a single timestamped line and the user block line-by-line.
pub(crate) async fn trace_write_outgoing_prompt_do(
    file: &mut tokio::fs::File,
    parts: DoOutgoingTraceParts<'_>,
) -> Result<(), String> {
    use tokio::io::AsyncWriteExt;
    let DoOutgoingTraceParts {
        style_text,
        header_text,
        user_text,
        tee_stdout,
    } = parts;
    let tee_style = if tee_stdout {
        TaggedOutgoingTee::MirrorEachLine
    } else {
        TaggedOutgoingTee::Off
    };
    let tee_header = if tee_stdout {
        TaggedOutgoingTee::MirrorHeaderCollapsed
    } else {
        TaggedOutgoingTee::Off
    };
    let tee_user = tee_style;
    if let Some(s) = style_text.filter(|t| !t.trim().is_empty()) {
        trace_write_tagged_body(
            file,
            TaggedBodyWrite {
                stem: "style",
                body: s.trim(),
                tee: tee_style,
            },
        )
        .await?;
    }
    trace_write_tagged_body(
        file,
        TaggedBodyWrite {
            stem: "header",
            body: header_text,
            tee: tee_header,
        },
    )
    .await?;
    trace_write_tagged_body(
        file,
        TaggedBodyWrite {
            stem: "prompt",
            body: user_text,
            tee: tee_user,
        },
    )
    .await?;
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
    tee_stdout: bool,
) -> Result<(), String> {
    use tokio::io::AsyncWriteExt;
    let tee = if tee_stdout {
        TaggedOutgoingTee::MirrorEachLine
    } else {
        TaggedOutgoingTee::Off
    };
    trace_write_tagged_body(
        file,
        TaggedBodyWrite {
            stem,
            body: prompt_text,
            tee,
        },
    )
    .await?;
    file.flush()
        .await
        .map_err(|e| format!("trace outgoing prompt flush: {e}"))?;
    Ok(())
}

#[test]
fn acp_tee_echo_outgoing_skips_learn_stdout() {
    assert!(!acp_tee_echo_outgoing_prompt_lines(true, "learn"));
    assert!(acp_tee_echo_outgoing_prompt_lines(true, "implement"));
    assert!(!acp_tee_echo_outgoing_prompt_lines(false, "learn"));
}

#[test]
fn kiss_stringify_session_trace() {
    let _ = stringify!(trace_prepare_file);
    let _ = stringify!(trace_open_truncated);
    let _ = stringify!(trace_write_invocation_header);
    let _ = stringify!(acp_tee_echo_outgoing_prompt_lines);
    let _ = stringify!(trace_write_outgoing_prompt);
    let _ = stringify!(trace_write_outgoing_prompt_do);
    let _ = stringify!(TaggedOutgoingTee::Off);
    let _ = stringify!(TaggedOutgoingTee::MirrorEachLine);
    let _ = stringify!(TaggedOutgoingTee::MirrorHeaderCollapsed);
    let _ = stringify!(TaggedBodyWrite);
    let _ = stringify!(DoOutgoingTraceParts);
    let _ = stringify!(collapse_header_for_stdout_one_line);
}

#[test]
fn collapse_header_stdout_joins_nonempty_lines() {
    assert_eq!(
        collapse_header_for_stdout_one_line("a\n\nb\n"),
        "a b"
    );
}
