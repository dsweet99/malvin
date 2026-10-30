use std::path::Path;
use std::process::Stdio;
use std::time::Duration;

const BRIDGE: &str = r#"
import fs from 'node:fs';
import readline from 'node:readline';
let model;
const emit = obj => process.stdout.write(JSON.stringify(obj) + '\n');
for await (const line of readline.createInterface({input: process.stdin})) {
    const req = JSON.parse(line);
    if (req.op === 'create') {
        model = req.model;
        fs.appendFileSync('attempts', model + '\n');
        emit({event: 'ok', agentId: 'summary'});
    } else if (req.op === 'send') {
        if (model === 'poor') {
            emit({event: 'fatal', message: 'Insufficient funds', retryable: false});
        } else if (model === 'broken') {
            emit({event: 'fatal', message: 'invalid model', retryable: false});
        } else if (model === 'main') {
            while (!fs.existsSync('started')) {
                await new Promise(resolve => setTimeout(resolve, 5));
            }
            emit({event: 'run_done', status: 'finished', result: '__MALVIN_DONE__'});
        } else if (model === 'slow') {
            fs.writeFileSync('started', '');
        } else {
            const prompt = req.prompt || '';
            const result = model === 'echo' ? 'Use 30 characters now'
                : model === 'malformed' ? 'one two three four five six seven eight'
                : model === 'brief' ? 'Fix the parser'
                : model === 'live' && prompt.includes('current status')
                    ? (prompt.includes('EDITING') ? 'Editing the parser' : 'Reading the plan')
                : 'Add cheap request summaries';
            emit({event: 'run_done', status: 'finished', result});
        }
    } else if (req.op === 'close') {
        break;
    }
}
"#;

fn prepare(work: &Path, models: &str) {
    std::fs::create_dir_all(work.join(".malvinconf")).unwrap();
    std::fs::write(
        work.join(".malvinconf/config.toml"),
        format!("[agent]\nmodel = ['cursor:auto']\nmodel_lite = {models}\n"),
    )
    .unwrap();
    std::fs::write(
        work.join("request.md"),
        "Please implement background request summaries for the herdr status line right now.",
    )
    .unwrap();
    std::fs::write(work.join("bridge.mjs"), BRIDGE).unwrap();
}

fn command(work: &Path) -> tokio::process::Command {
    let mut command = tokio::process::Command::new(env!("CARGO_BIN_EXE_malvin"));
    command
        .current_dir(work)
        .env("HOME", work)
        .env("MALVIN_TEST_HOME", work)
        .env("MALVIN_CURSOR_SDK_BRIDGE", work.join("bridge.mjs"))
        .env("CURSOR_API_KEY", "test-key")
        .env("MALVIN_TEST_NO_REAL_AGENT", "1")
        .env("MALVIN_TEST_FAST_TEARDOWN", "1")
        .env("MALVIN_NO_HERDR", "1")
        .env_remove("MALVIN_SUMMARY_WORKER_REQUEST")
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .kill_on_drop(true);
    command
}

fn worker(work: &Path) -> tokio::process::Child {
    command(work)
        .env("MALVIN_SUMMARY_WORKER_REQUEST", work.join("request.md"))
        .spawn()
        .unwrap()
}

async fn finish(child: &mut tokio::process::Child) -> std::process::ExitStatus {
    tokio::time::timeout(Duration::from_secs(3), child.wait())
        .await
        .unwrap()
        .unwrap()
}

async fn run_worker(work: &Path) -> (std::process::ExitStatus, Option<String>) {
    let mut child = worker(work);
    let stdin = child.stdin.take();
    let summary = work.join("request_summary.txt");
    let status = tokio::time::timeout(Duration::from_secs(3), async move {
        loop {
            if let Some(status) = child.try_wait().unwrap() {
                return status;
            }
            if summary.exists() {
                drop(stdin);
                return child.wait().await.unwrap();
            }
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
    })
    .await
    .unwrap();
    (
        status,
        std::fs::read_to_string(work.join("request_summary.txt")).ok(),
    )
}

async fn wait_for_text(path: &Path, expected: &str) {
    tokio::time::timeout(Duration::from_secs(3), async {
        loop {
            if std::fs::read_to_string(path).ok().as_deref() == Some(expected) {
                return;
            }
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
    })
    .await
    .unwrap();
}

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
