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
                : model === 'malformed' ? 'Editing \u001b[31mthe parser'
                : model === 'verbose' ? 'Add cheap request summaries today'
                : model === 'brief' ? 'Fix the parser'
                : model === 'live' && prompt.includes('current status')
                    ? (prompt.includes('EDITING') ? 'Editing the parser' : 'Reading the plan comprehensively')
                : 'Add cheap request summaries';
            emit({event: 'run_done', status: 'finished', result});
        }
    } else if (req.op === 'close') {
        break;
    }
}
"#;

pub(super) fn prepare(work: &Path, models: &str) {
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

pub(super) fn command(work: &Path) -> tokio::process::Command {
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

pub(super) fn worker(work: &Path) -> tokio::process::Child {
    command(work)
        .env("MALVIN_SUMMARY_WORKER_REQUEST", work.join("request.md"))
        .spawn()
        .unwrap()
}

pub(super) async fn finish(child: &mut tokio::process::Child) -> std::process::ExitStatus {
    tokio::time::timeout(Duration::from_secs(3), child.wait())
        .await
        .unwrap()
        .unwrap()
}

pub(super) async fn run_worker(work: &Path) -> (std::process::ExitStatus, Option<String>) {
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

pub(super) async fn wait_for_text(path: &Path, expected: &str) {
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

