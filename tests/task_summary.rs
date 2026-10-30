use std::time::Duration;

#[path = "task_summary/support.rs"]
mod support;
use support::*;

#[tokio::test]
async fn summary_worker_falls_back_only_on_quota_and_rejects_malformed_output() {
    for (models, attempts, summary) in [
        (
            "['cursor:poor', 'cursor:cheap']",
            "poor\ncheap\n",
            Some("Add cheap request summaries"),
        ),
        ("['cursor:broken', 'cursor:cheap']", "broken\n", None),
        ("['cursor:malformed']", "malformed\n", None),
        (
            "['cursor:verbose']",
            "verbose\n",
            Some("Add cheap request summaries"),
        ),
        ("['cursor:echo']", "echo\n", None),
        ("['cursor:brief']", "brief\n", Some("Fix the parser")),
    ] {
        let dir = tempfile::tempdir().unwrap();
        prepare(dir.path(), models);
        let (status, stored) = run_worker(dir.path()).await;
        assert_eq!(status.success(), summary.is_some());
        assert_eq!(
            std::fs::read_to_string(dir.path().join("attempts")).unwrap(),
            attempts
        );
        assert_eq!(stored.as_deref(), summary);
    }
}

#[tokio::test]
async fn short_request_is_trimmed_without_calling_the_model() {
    let dir = tempfile::tempdir().unwrap();
    prepare(dir.path(), "['cursor:cheap']");
    std::fs::write(dir.path().join("request.md"), "Fix the parser tonight.").unwrap();
    let (status, stored) = run_worker(dir.path()).await;
    assert!(status.success());
    assert!(!dir.path().join("attempts").exists());
    assert_eq!(stored.as_deref(), Some("Fix the parser tonight."));
}

#[tokio::test]
async fn requests_under_ten_words_use_the_first_thirty_characters() {
    for request in [
        "Investigate extraordinarily complicated parsing failures",
        "One two three four five six seven eight nine",
        "Réparer immédiatement les caractères Unicode du résumé",
    ] {
        let dir = tempfile::tempdir().unwrap();
        prepare(dir.path(), "['cursor:cheap']");
        std::fs::write(dir.path().join("request.md"), request).unwrap();
        let (status, stored) = run_worker(dir.path()).await;
        assert!(status.success());
        assert_eq!(stored, Some(request.chars().take(30).collect()));
        assert!(!dir.path().join("attempts").exists());
    }
}

#[tokio::test]
async fn ten_word_request_calls_the_summary_model() {
    let dir = tempfile::tempdir().unwrap();
    prepare(dir.path(), "['cursor:cheap']");
    std::fs::write(
        dir.path().join("request.md"),
        "One two three four five six seven eight nine ten",
    )
    .unwrap();
    let (status, stored) = run_worker(dir.path()).await;
    assert!(status.success());
    assert_eq!(stored.as_deref(), Some("Add cheap request summaries"));
    assert_eq!(
        std::fs::read_to_string(dir.path().join("attempts")).unwrap(),
        "cheap\n"
    );
}

#[tokio::test]
async fn short_request_works_without_model_lite_configuration() {
    let dir = tempfile::tempdir().unwrap();
    prepare(dir.path(), "['cursor:cheap']");
    std::fs::write(dir.path().join(".malvinconf/config.toml"), "").unwrap();
    let request = "Investigate extraordinarily complicated parsing failures";
    std::fs::write(dir.path().join("request.md"), request).unwrap();
    let (status, stored) = run_worker(dir.path()).await;
    assert!(status.success());
    assert_eq!(stored, Some(request.chars().take(30).collect()));
    assert!(!dir.path().join("attempts").exists());
}

#[tokio::test]
async fn blank_request_does_not_call_the_summary_model() {
    let dir = tempfile::tempdir().unwrap();
    prepare(dir.path(), "['cursor:cheap']");
    std::fs::write(dir.path().join("request.md"), " \n").unwrap();
    let mut child = worker(dir.path());
    let _connection = child.stdin.take();
    assert!(!finish(&mut child).await.success());
    assert!(!dir.path().join("attempts").exists());
    assert!(!dir.path().join("request_summary.txt").exists());
}

#[tokio::test]
async fn disconnected_parent_cancels_a_pending_summary() {
    let dir = tempfile::tempdir().unwrap();
    prepare(dir.path(), "['cursor:slow']");
    let mut child = worker(dir.path());
    tokio::time::timeout(Duration::from_secs(2), async {
        while !dir.path().join("started").exists() {
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
    })
    .await
    .unwrap();
    drop(child.stdin.take());
    assert!(finish(&mut child).await.success());
    assert!(!dir.path().join("request_summary.txt").exists());
}

#[tokio::test]
async fn status_token_tracks_recent_output_on_the_test_schedule() {
    let dir = tempfile::tempdir().unwrap();
    prepare(dir.path(), "['cursor:live']");
    std::fs::write(dir.path().join("stdout.log"), "Gathering context\n").unwrap();
    let mut child = command(dir.path())
        .env(
            "MALVIN_SUMMARY_WORKER_REQUEST",
            dir.path().join("request.md"),
        )
        .env("MALVIN_TEST_STATUS_SCHEDULE_MS", "80,120")
        .spawn()
        .unwrap();
    let stdin = child.stdin.take();
    let status_path = dir.path().join("status_summary.txt");
    wait_for_text(&status_path, "Reading the plan").await;
    std::fs::write(dir.path().join("stdout.log"), "EDITING the parser\n").unwrap();
    wait_for_text(&status_path, "Editing the parser").await;
    drop(stdin);
    assert!(finish(&mut child).await.success());
    assert_eq!(
        std::fs::read_to_string(dir.path().join("request_summary.txt")).unwrap(),
        "Add cheap request summaries"
    );
}

#[tokio::test]
async fn status_token_refreshes_when_a_main_agent_prompt_is_sent() {
    let dir = tempfile::tempdir().unwrap();
    prepare(dir.path(), "['cursor:live']");
    let mut child = command(dir.path())
        .env(
            "MALVIN_SUMMARY_WORKER_REQUEST",
            dir.path().join("request.md"),
        )
        .env("MALVIN_TEST_STATUS_SCHEDULE_MS", "60000,60000")
        .spawn()
        .unwrap();
    let stdin = child.stdin.take();
    let status_path = dir.path().join("status_summary.txt");
    let prompts = dir.path().join("prompt_sends");
    wait_for_text(
        &dir.path().join("request_summary.txt"),
        "Add cheap request summaries",
    )
    .await;
    std::fs::write(dir.path().join("stdout.log"), "Gathering context\n").unwrap();
    tokio::time::sleep(Duration::from_millis(300)).await;
    assert!(!status_path.exists());
    std::fs::write(&prompts, "\n").unwrap();
    wait_for_text(&status_path, "Reading the plan").await;
    std::fs::write(dir.path().join("stdout.log"), "EDITING the parser\n").unwrap();
    tokio::time::sleep(Duration::from_millis(300)).await;
    assert_eq!(
        std::fs::read_to_string(&status_path).unwrap(),
        "Reading the plan"
    );
    let mut prompts_file = std::fs::OpenOptions::new()
        .append(true)
        .open(&prompts)
        .unwrap();
    std::io::Write::write_all(&mut prompts_file, b"\n").unwrap();
    wait_for_text(&status_path, "Editing the parser").await;
    drop(stdin);
    assert!(finish(&mut child).await.success());
}

#[tokio::test]
async fn each_queued_prompt_send_is_summarized() {
    let dir = tempfile::tempdir().unwrap();
    prepare(dir.path(), "['cursor:live']");
    let mut child = command(dir.path())
        .env(
            "MALVIN_SUMMARY_WORKER_REQUEST",
            dir.path().join("request.md"),
        )
        .env("MALVIN_TEST_STATUS_SCHEDULE_MS", "60000,60000")
        .spawn()
        .unwrap();
    let stdin = child.stdin.take();
    let attempts = dir.path().join("attempts");
    wait_for_text(
        &dir.path().join("request_summary.txt"),
        "Add cheap request summaries",
    )
    .await;
    assert_eq!(std::fs::read_to_string(&attempts).unwrap(), "live\n");
    std::fs::write(dir.path().join("stdout.log"), "EDITING the parser\n").unwrap();
    std::fs::write(dir.path().join("prompt_sends"), "\n\n").unwrap();
    wait_for_text(&attempts, "live\nlive\nlive\n").await;
    wait_for_text(
        &dir.path().join("status_summary.txt"),
        "Editing the parser",
    )
    .await;
    drop(stdin);
    assert!(finish(&mut child).await.success());
}

#[tokio::test]
async fn main_agent_finishes_while_summary_is_still_pending() {
    let dir = tempfile::tempdir().unwrap();
    prepare(dir.path(), "['cursor:slow']");
    let mut child = command(dir.path())
        .args([
            "--no-auto-costs",
            "--no-emit",
            "--model=cursor:main",
            "--max-loops=1",
            "Implement herdr task and status tokens for this request now",
        ])
        .spawn()
        .unwrap();
    assert!(finish(&mut child).await.success());
    assert!(dir.path().join("started").exists());
    assert!(!dir.path().join("request_summary.txt").exists());
}
