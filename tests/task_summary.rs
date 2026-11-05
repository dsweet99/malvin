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
            Some("Add cheap summaries"),
        ),
        ("['cursor:broken', 'cursor:cheap']", "broken\n", None),
        ("['cursor:malformed']", "malformed\n", None),
        (
            "['cursor:verbose']",
            "verbose\n",
            Some("Add cheap request..."),
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
    std::fs::write(dir.path().join("request.md"), "Fix the parser now.").unwrap();
    let (status, stored) = run_worker(dir.path()).await;
    assert!(status.success());
    assert!(!dir.path().join("attempts").exists());
    assert_eq!(stored.as_deref(), Some("Fix the parser now."));
}

#[tokio::test]
async fn requests_under_ten_words_use_the_first_twenty_characters() {
    let dir = tempfile::tempdir().unwrap();
    prepare(dir.path(), "['cursor:cheap']");
    let request = "Investigate extraordinarily complicated parsing failures";
    std::fs::write(dir.path().join("request.md"), request).unwrap();
    let (status, stored) = run_worker(dir.path()).await;
    assert!(status.success());
    assert_eq!(stored.as_deref(), Some("Investigate..."));
    assert!(!dir.path().join("attempts").exists());
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
    assert_eq!(stored.as_deref(), Some("Add cheap summaries"));
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
    assert_eq!(stored.as_deref(), Some("Investigate..."));
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
    std::fs::create_dir_all(dir.path().join("main")).unwrap();
    std::fs::write(dir.path().join("main/stdout.log"), "Gathering context\n").unwrap();
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
    wait_for_text(&status_path, "Reading the plan...").await;
    std::fs::create_dir_all(dir.path().join("main")).unwrap();
    std::fs::write(dir.path().join("main/stdout.log"), "EDITING the parser\n").unwrap();
    wait_for_text(&status_path, "Editing the parser").await;
    drop(stdin);
    assert!(finish(&mut child).await.success());
    assert_eq!(
        std::fs::read_to_string(dir.path().join("request_summary.txt")).unwrap(),
        "Add cheap summaries"
    );
}

#[tokio::test]
async fn status_b_fence_is_forwarded_over_btw_and_traced_in_lite() {
    let dir = tempfile::tempdir().unwrap();
    prepare(dir.path(), "['cursor:btw']");
    std::fs::create_dir_all(dir.path().join("main")).unwrap();
    std::fs::write(dir.path().join("main/stdout.log"), "EDITING the parser\n").unwrap();
    let mut child = command(dir.path())
        .env(
            "MALVIN_SUMMARY_WORKER_REQUEST",
            dir.path().join("request.md"),
        )
        .env("MALVIN_TEST_STATUS_SCHEDULE_MS", "50,50")
        .spawn()
        .unwrap();
    let stdin = child.stdin.take();
    let btw = dir.path().join("main/btw");
    tokio::time::timeout(Duration::from_secs(5), async {
        while !std::fs::read_dir(&btw).is_ok_and(|mut d| d.next().is_some()) {
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
    })
    .await
    .unwrap();
    drop(stdin);
    assert!(finish(&mut child).await.success());
    let note = std::fs::read_dir(&btw)
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .path();
    assert_eq!(
        std::fs::read_to_string(note).unwrap().trim(),
        "Run `malvin --advice kpop`"
    );
}

#[tokio::test]
async fn status_schedule_is_measured_from_worker_start_without_skipping() {
    let dir = tempfile::tempdir().unwrap();
    prepare(dir.path(), "['cursor:lag']");
    std::fs::create_dir_all(dir.path().join("main")).unwrap();
    std::fs::write(dir.path().join("main/stdout.log"), "EDITING the parser\n").unwrap();
    let mut child = command(dir.path())
        .env(
            "MALVIN_SUMMARY_WORKER_REQUEST",
            dir.path().join("request.md"),
        )
        .env("MALVIN_TEST_STATUS_SCHEDULE_MS", "300,300")
        .spawn()
        .unwrap();
    let stdin = child.stdin.take();
    let status_at = dir.path().join("status_at");
    tokio::time::timeout(Duration::from_secs(5), async {
        while std::fs::read_to_string(&status_at).map_or(0, |s| s.lines().count()) < 2 {
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
    })
    .await
    .unwrap();
    drop(stdin);
    assert!(finish(&mut child).await.success());
    let task_done: u64 = std::fs::read_to_string(dir.path().join("task_done_at"))
        .unwrap()
        .trim()
        .parse()
        .unwrap();
    let stamps: Vec<u64> = std::fs::read_to_string(&status_at)
        .unwrap()
        .lines()
        .map(|line| line.parse().unwrap())
        .collect();
    // Offsets are 300ms and 600ms from the worker start. The 400ms task summary
    // overruns the first, so that status run happens right after it (not
    // skipped); the second still lands at 600ms, i.e. under one period after
    // the first run, proving it is not measured from the previous run.
    assert!(stamps[0] >= task_done, "{task_done} {stamps:?}");
    assert!(stamps[0] < task_done + 150, "{task_done} {stamps:?}");
    assert!(stamps[1] > stamps[0], "{stamps:?}");
    assert!(stamps[1] < stamps[0] + 290, "{stamps:?}");
    assert_eq!(
        std::fs::read_to_string(dir.path().join("status_summary.txt")).unwrap(),
        "Editing the parser"
    );
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
