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

async fn trace_write_tagged_body(
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

#[allow(clippy::struct_field_names)]
pub(crate) struct DoOutgoingTraceParts<'a> {
    pub style_text: Option<&'a str>,
    pub header_text: &'a str,
    pub user_text: &'a str,
}

pub(crate) fn compose_do_split_prompt_text(parts: &DoOutgoingTraceParts<'_>) -> String {
    let mut sections = Vec::new();
    if let Some(style) = parts.style_text.map(str::trim).filter(|t| !t.is_empty()) {
        sections.push(style.to_string());
    }
    sections.push(parts.header_text.to_string());
    sections.push(parts.user_text.to_string());
    sections.join("\n\n")
}

pub(crate) async fn trace_write_invocation_and_do_split_prompt(
    file: &mut tokio::fs::File,
    split: &outgoing_prompt_trace::DoPromptTraceSplit<'_>,
) -> Result<(), String> {
    trace_write_invocation_header(file).await?;
    trace_write_outgoing_prompt_do(
        file,
        DoOutgoingTraceParts {
            style_text: split.style_text,
            header_text: split.header,
            user_text: split.user,
        },
    )
    .await
}

/// `malvin do`: disk trace matches the full prompt (style, then `header.md`, then user request).
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

pub(crate) const PROMPTS_LOG_FILE_NAME: &str = "prompts.log";

pub(crate) async fn append_prompts_log_uniform(
    run_dir: Option<&std::path::Path>,
    trace_stem: &str,
    bracket_label: &str,
    prompt_text: Option<&str>,
) -> Result<(), String> {
    use tokio::io::AsyncWriteExt;
    let Some(dir) = run_dir else {
        return Ok(());
    };
    let path = dir.join(PROMPTS_LOG_FILE_NAME);
    let mut f = tokio::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .await
        .map_err(|e| format!("prompts.log open: {e}"))?;
    let tag = crate::output::format_acp_directional_tag_prefix('>', trace_stem);
    if let Some(body) = prompt_text {
        for line in crate::output::logical_lines(body) {
            let l = crate::output::format_line(&tag, line);
            f.write_all(l.as_bytes())
                .await
                .map_err(|e| format!("prompts.log write: {e}"))?;
            f.write_all(b"\n")
                .await
                .map_err(|e| format!("prompts.log nl: {e}"))?;
        }
    } else {
        let summary = format!("[{bracket_label}...]");
        let l = crate::output::format_line(&tag, &summary);
        f.write_all(l.as_bytes())
            .await
            .map_err(|e| format!("prompts.log write: {e}"))?;
        f.write_all(b"\n")
            .await
            .map_err(|e| format!("prompts.log nl: {e}"))?;
    }
    f.flush()
        .await
        .map_err(|e| format!("prompts.log flush: {e}"))?;
    Ok(())
}

pub(crate) async fn append_prompts_log_do_plain(
    run_dir: Option<&std::path::Path>,
    parts: &DoOutgoingTraceParts<'_>,
    include_full_combined: bool,
) -> Result<(), String> {
    use tokio::io::AsyncWriteExt;
    let Some(dir) = run_dir else {
        return Ok(());
    };
    let path = dir.join(PROMPTS_LOG_FILE_NAME);
    let mut f = tokio::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .await
        .map_err(|e| format!("prompts.log open: {e}"))?;
    let tag = crate::output::format_acp_directional_tag_prefix('>', "do");
    if include_full_combined {
        let combined = compose_do_split_prompt_text(parts);
        for line in crate::output::logical_lines(&combined) {
            let l = crate::output::format_line(&tag, line);
            f.write_all(l.as_bytes())
                .await
                .map_err(|e| format!("prompts.log write: {e}"))?;
            f.write_all(b"\n")
                .await
                .map_err(|e| format!("prompts.log nl: {e}"))?;
        }
    } else {
        let summary = "[do...]".to_string();
        let l = crate::output::format_line(&tag, &summary);
        f.write_all(l.as_bytes())
            .await
            .map_err(|e| format!("prompts.log write: {e}"))?;
        f.write_all(b"\n")
            .await
            .map_err(|e| format!("prompts.log nl: {e}"))?;
    }
    f.flush()
        .await
        .map_err(|e| format!("prompts.log flush: {e}"))?;
    Ok(())
}

#[test]
fn kiss_stringify_session_trace() {
    let _ = stringify!(trace_prepare_file);
    let _ = stringify!(trace_open_truncated);
    let _ = stringify!(trace_write_invocation_header);
    let _ = stringify!(trace_write_invocation_and_do_split_prompt);
    let _ = stringify!(trace_write_outgoing_prompt);
    let _ = stringify!(trace_write_outgoing_prompt_do);
    let _ = stringify!(DoOutgoingTraceParts);
    let _ = stringify!(file_write_line_with_newline);
    let _ = stringify!(trace_write_tagged_body);
    let _ = stringify!(trace_write_plain_body);
    let _ = stringify!(compose_do_split_prompt_text);
    let _ = stringify!(trace_write_tagged_body_writes_prefixed_lines);
    let _ = stringify!(trace_write_outgoing_prompt_do_writes_plain_lines_without_tags);
    let _ = stringify!(trace_write_outgoing_prompt_do_preserves_header_user_separator);
    let _ = stringify!(PROMPTS_LOG_FILE_NAME);
    let _ = stringify!(append_prompts_log_uniform);
    let _ = stringify!(append_prompts_log_do_plain);
}

#[tokio::test]
async fn trace_write_tagged_body_writes_prefixed_lines() {
    let tmp = tempfile::NamedTempFile::new().unwrap();
    let path = tmp.path();
    let mut file = tokio::fs::OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(path)
        .await
        .unwrap();
    trace_write_tagged_body(&mut file, "test", "line1\nline2").await.unwrap();
    drop(file);
    let content = std::fs::read_to_string(path).unwrap();
    assert!(content.contains("test"), "should include stem");
    assert!(content.contains("line1"), "should include line1");
    assert!(content.contains("line2"), "should include line2");
}

#[tokio::test]
async fn trace_write_outgoing_prompt_do_writes_plain_lines_without_tags() {
    let tmp = tempfile::NamedTempFile::new().unwrap();
    let path = tmp.path();
    let mut file = tokio::fs::OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(path)
        .await
        .unwrap();
    trace_write_outgoing_prompt_do(
        &mut file,
        DoOutgoingTraceParts {
            style_text: Some("STYLE"),
            header_text: "HEADER",
            user_text: "PROMPT",
        },
    )
    .await
    .unwrap();
    drop(file);
    let content = std::fs::read_to_string(path).unwrap();
    assert_eq!(content, "STYLE\n\nHEADER\n\nPROMPT\n");
    assert!(!content.contains(":[>style"));
    assert!(!content.contains(":[>header"));
    assert!(!content.contains(":[>prompt"));
}

#[tokio::test]
async fn append_prompts_log_uniform_appends_tagged_timestamped_lines() {
    let tmp = tempfile::tempdir().unwrap();
    let run_dir = tmp.path().join("_malvin").join("r");
    tokio::fs::create_dir_all(&run_dir).await.unwrap();
    append_prompts_log_uniform(Some(&run_dir), "implement", "implement.md", Some("a\nb"))
        .await
        .unwrap();
    append_prompts_log_uniform(Some(&run_dir), "implement", "implement.md", Some("c"))
        .await
        .unwrap();
    let content = tokio::fs::read_to_string(run_dir.join(PROMPTS_LOG_FILE_NAME))
        .await
        .unwrap();
    assert_eq!(content.matches(">implement").count(), 3);
    assert!(content.contains("]: a"));
    assert!(content.contains("]: b"));
    assert!(content.contains("]: c"));
}

#[tokio::test]
async fn append_prompts_log_do_plain_uses_do_stem_like_stdout() {
    let tmp = tempfile::tempdir().unwrap();
    let run_dir = tmp.path().join("_malvin").join("do_run");
    tokio::fs::create_dir_all(&run_dir).await.unwrap();
    append_prompts_log_do_plain(
        Some(&run_dir),
        &DoOutgoingTraceParts {
            style_text: None,
            header_text: "H",
            user_text: "U",
        },
        true,
    )
    .await
    .unwrap();
    let content = tokio::fs::read_to_string(run_dir.join(PROMPTS_LOG_FILE_NAME))
        .await
        .unwrap();
    assert!(
        content.contains(">do"),
        "prompts.log should match stdout directional stem for do: {content}"
    );
    assert!(content.contains("]: H"));
    assert!(content.contains("]: U"));
}

#[tokio::test]
async fn append_prompts_log_uniform_name_only_writes_one_summary_line() {
    let tmp = tempfile::tempdir().unwrap();
    let run_dir = tmp.path().join("_malvin").join("r");
    tokio::fs::create_dir_all(&run_dir).await.unwrap();
    append_prompts_log_uniform(Some(&run_dir), "learn", "learn.md", None)
        .await
        .unwrap();
    let content = tokio::fs::read_to_string(run_dir.join(PROMPTS_LOG_FILE_NAME))
        .await
        .unwrap();
    assert_eq!(content.matches(">learn").count(), 1);
    assert!(content.contains("]: [learn.md...]"));
}

#[tokio::test]
async fn append_prompts_log_do_plain_name_only_writes_do_summary() {
    let tmp = tempfile::tempdir().unwrap();
    let run_dir = tmp.path().join("_malvin").join("do_run");
    tokio::fs::create_dir_all(&run_dir).await.unwrap();
    append_prompts_log_do_plain(
        Some(&run_dir),
        &DoOutgoingTraceParts {
            style_text: None,
            header_text: "SECRET",
            user_text: "PRIVATE",
        },
        false,
    )
    .await
    .unwrap();
    let content = tokio::fs::read_to_string(run_dir.join(PROMPTS_LOG_FILE_NAME))
        .await
        .unwrap();
    assert!(content.contains(">do"));
    assert!(content.contains("]: [do...]"));
    assert!(!content.contains("SECRET"));
}

#[tokio::test]
async fn trace_write_outgoing_prompt_do_preserves_header_user_separator() {
    let tmp = tempfile::NamedTempFile::new().unwrap();
    let path = tmp.path();
    let mut file = tokio::fs::OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(path)
        .await
        .unwrap();
    trace_write_outgoing_prompt_do(
        &mut file,
        DoOutgoingTraceParts {
            style_text: None,
            header_text: "HEADER",
            user_text: "USER",
        },
    )
    .await
    .unwrap();
    drop(file);
    let content = std::fs::read_to_string(path).unwrap();
    assert_eq!(content, "HEADER\n\nUSER\n");
}
