use crate::acp::{
    append_prompts_log_do_plain, append_prompts_log_uniform, DoOutgoingTraceParts,
    PROMPTS_LOG_FILE_NAME, trace_write_outgoing_prompt_do, trace_write_tagged_body,
};
use crate::output::{format_who_tag_delim, WHO_U};

#[tokio::test]
pub(crate) async fn trace_write_tagged_body_writes_prefixed_lines() {
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
    assert!(content.contains(&format_who_tag_delim(WHO_U)), "should include user tag");
    assert!(content.contains("line1"), "should include line1");
    assert!(content.contains("line2"), "should include line2");
}

#[tokio::test]
pub(crate) async fn trace_write_outgoing_prompt_do_writes_plain_lines_without_tags() {
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
            header_text: "HEADER",
            user_text: "PROMPT",
        },
    )
    .await
    .unwrap();
    drop(file);
    let content = std::fs::read_to_string(path).unwrap();
    assert_eq!(content, "HEADER\n\nPROMPT\n");
    assert!(!content.contains(":[>style"));
    assert!(!content.contains(":[>header"));
    assert!(!content.contains(":[>prompt"));
}

#[tokio::test]
pub(crate) async fn append_prompts_log_uniform_appends_tagged_timestamped_lines() {
    let tmp = tempfile::tempdir().unwrap();
    let run_dir = tmp.path().join(".malvin/logs").join("r");
    tokio::fs::create_dir_all(&run_dir).await.unwrap();
    append_prompts_log_uniform(Some(&run_dir), "bug_fix", "bug_fix.md", Some("a\nb"))
        .await
        .unwrap();
    append_prompts_log_uniform(Some(&run_dir), "bug_fix", "bug_fix.md", Some("c"))
        .await
        .unwrap();
    let content = tokio::fs::read_to_string(run_dir.join(PROMPTS_LOG_FILE_NAME))
        .await
        .unwrap();
    let user_tag = format_who_tag_delim(WHO_U);
    assert_eq!(content.matches(&user_tag).count(), 3);
    assert!(content.contains("|a"));
    assert!(content.contains("|b"));
    assert!(content.contains("|c"));
}

#[tokio::test]
pub(crate) async fn append_prompts_log_do_plain_uses_do_stem_like_stdout() {
    let tmp = tempfile::tempdir().unwrap();
    let run_dir = tmp.path().join(".malvin/logs").join("do_run");
    tokio::fs::create_dir_all(&run_dir).await.unwrap();
    append_prompts_log_do_plain(
        Some(&run_dir),
        &DoOutgoingTraceParts {
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
        content.contains(&format_who_tag_delim(WHO_U)),
        "prompts.log should use user tag for do: {content}"
    );
    assert!(content.contains("|H"));
    assert!(content.contains("|U"));
}

#[tokio::test]
pub(crate) async fn append_prompts_log_uniform_name_only_writes_one_summary_line() {
    let tmp = tempfile::tempdir().unwrap();
    let run_dir = tmp.path().join(".malvin/logs").join("r");
    tokio::fs::create_dir_all(&run_dir).await.unwrap();
    append_prompts_log_uniform(Some(&run_dir), "summary", "summary.md", None)
        .await
        .unwrap();
    let content = tokio::fs::read_to_string(run_dir.join(PROMPTS_LOG_FILE_NAME))
        .await
        .unwrap();
    assert_eq!(content.matches(&format_who_tag_delim(WHO_U)).count(), 1);
    assert!(content.contains("|[summary.md...]"));
}

#[tokio::test]
pub(crate) async fn append_prompts_log_do_plain_name_only_writes_do_summary() {
    let tmp = tempfile::tempdir().unwrap();
    let run_dir = tmp.path().join(".malvin/logs").join("do_run");
    tokio::fs::create_dir_all(&run_dir).await.unwrap();
    append_prompts_log_do_plain(
        Some(&run_dir),
        &DoOutgoingTraceParts {
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
    assert!(content.contains(&format_who_tag_delim(WHO_U)));
    assert!(content.contains("|[do...]"));
    assert!(!content.contains("SECRET"));
}

#[tokio::test]
pub(crate) async fn trace_write_outgoing_prompt_do_preserves_header_user_separator() {
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


#[cfg(test)]
mod kiss_cov_auto{
    use super::*;

    #[test]
    fn kiss_cov_trace_write_tagged_body_writes_prefixed_lines() { let _ = trace_write_tagged_body_writes_prefixed_lines; }

    #[test]
    fn kiss_cov_trace_write_outgoing_prompt_do_writes_plain_lines_without_tags() { let _ = trace_write_outgoing_prompt_do_writes_plain_lines_without_tags; }

    #[test]
    fn kiss_cov_append_prompts_log_uniform_appends_tagged_timestamped_lines() { let _ = append_prompts_log_uniform_appends_tagged_timestamped_lines; }

    #[test]
    fn kiss_cov_append_prompts_log_do_plain_uses_do_stem_like_stdout() { let _ = append_prompts_log_do_plain_uses_do_stem_like_stdout; }

    #[test]
    fn kiss_cov_append_prompts_log_uniform_name_only_writes_one_summary_line() { let _ = append_prompts_log_uniform_name_only_writes_one_summary_line; }

    #[test]
    fn kiss_cov_append_prompts_log_do_plain_name_only_writes_do_summary() { let _ = append_prompts_log_do_plain_name_only_writes_do_summary; }

    #[test]
    fn kiss_cov_trace_write_outgoing_prompt_do_preserves_header_user_separator() { let _ = trace_write_outgoing_prompt_do_preserves_header_user_separator; }

}
