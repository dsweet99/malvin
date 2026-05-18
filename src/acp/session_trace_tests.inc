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
