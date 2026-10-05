use std::sync::Arc;

use tokio::process::{Child, ChildStdin, ChildStdout};

use super::session::NpmPiSession;
use crate::backends::bridge_sdk::turn_timeout_tests::{
    assert_dead_session_error, assert_turn_timeout_contract, cat_child, script_child,
    stdio_child_from,
};
use crate::backends::bridge_sdk::{DrainIdleTurn, JsonLineSession};

fn npm_pi_session_from(process: (Child, ChildStdin, ChildStdout)) -> NpmPiSession {
    NpmPiSession {
        stdio: stdio_child_from(process),
        pi_model: None,
        local_hold: false,
        output_cap: None,
    }
}

pub(super) fn cat_npm_pi_session() -> NpmPiSession {
    npm_pi_session_from(cat_child())
}

#[tokio::test]
async fn npm_pi_session_extends_turn_and_reports_tools_in_flight() {
    let session = cat_npm_pi_session();
    assert_turn_timeout_contract(&session);
}

#[tokio::test]
async fn npm_pi_json_line_round_trips_through_child() {
    let session = cat_npm_pi_session();
    let sent = serde_json::json!({"type": "ping", "n": 1});
    session.write_json(&sent).await.expect("write");
    let mut turn = DrainIdleTurn::new();
    let got = session
        .read_json_waiting("echo", &mut turn)
        .await
        .expect("read");
    assert_eq!(got, sent);
}

#[tokio::test]
async fn npm_pi_json_line_accepts_crlf() {
    let session = npm_pi_session_from(script_child("printf '{\"a\":1}\\r\\n'; exec cat"));
    let mut turn = DrainIdleTurn::new();
    let got = session
        .read_json_waiting("crlf", &mut turn)
        .await
        .expect("read");
    assert_eq!(got, serde_json::json!({"a": 1}));
}

#[tokio::test]
async fn npm_pi_json_line_failures_are_dead_session_errors() {
    let closed = npm_pi_session_from(script_child("exit 0"));
    let mut turn = DrainIdleTurn::new();
    let err = closed
        .read_json_waiting("eof", &mut turn)
        .await
        .unwrap_err();
    assert_dead_session_error(&err, "npm pi stdout closed");

    let garbled = npm_pi_session_from(script_child("echo nope; exec cat"));
    let err = garbled
        .read_json_waiting("parse", &mut turn)
        .await
        .unwrap_err();
    assert_dead_session_error(&err, "npm pi JSONL parse:");
}

#[tokio::test]
async fn npm_pi_session_drop_marks_reader_dead() {
    let session = cat_npm_pi_session();
    let reader_dead = Arc::clone(&session.stdio.reader_dead);
    drop(session);
    assert!(reader_dead.load(std::sync::atomic::Ordering::SeqCst));
}
