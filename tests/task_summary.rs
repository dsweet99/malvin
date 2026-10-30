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
            const result = model === 'malformed' ? 'too short' : 'Add cheap request summary models';
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
        "Please implement request summaries.",
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

#[tokio::test]
async fn summary_worker_falls_back_only_on_quota_and_rejects_malformed_output() {
    for (models, attempts, success) in [
        ("['cursor:poor', 'cursor:cheap']", "poor\ncheap\n", true),
        ("['cursor:broken', 'cursor:cheap']", "broken\n", false),
        ("['cursor:malformed']", "malformed\n", false),
    ] {
        let dir = tempfile::tempdir().unwrap();
        prepare(dir.path(), models);
        let mut child = worker(dir.path());
        let _connection = child.stdin.take();
        assert_eq!(finish(&mut child).await.success(), success);
        assert_eq!(
            std::fs::read_to_string(dir.path().join("attempts")).unwrap(),
            attempts
        );
        let summary = std::fs::read_to_string(dir.path().join("request_summary.txt"));
        if success {
            assert_eq!(summary.unwrap(), "Add cheap request summary models");
        } else {
            assert!(summary.is_err());
        }
    }
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
    assert!(child.try_wait().unwrap().is_none());
    drop(child.stdin.take());
    assert!(finish(&mut child).await.success());
    assert!(!dir.path().join("request_summary.txt").exists());
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
            "Implement summaries",
        ])
        .spawn()
        .unwrap();
    assert!(finish(&mut child).await.success());
    assert!(
        dir.path().join("started").exists(),
        "main waited for the summary to start"
    );
    assert!(!dir.path().join("request_summary.txt").exists());
}
