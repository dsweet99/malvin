use std::thread;
use std::time::{Duration, Instant};

use crate::agent_process::{AgentIoOptions, SessionUpdateChunkKind};
use crate::output::{enable_stdout_capture, take_captured_stdout};

use super::{
    STDOUT_COALESCE_IDLE, StdoutCoalesceBuf, build, feed, flush, idle_of, pending, recorded,
    write_coalesced_emissions,
};

fn sample_io() -> AgentIoOptions {
    AgentIoOptions {
        no_tee: false,
        raw_output: false,
        show_thoughts_on_stdout: false,
        emit_stdout_markdown: false,
        log_full_outgoing_prompts: false,
    }
}

fn message(
    text: &str,
) -> (
    SessionUpdateChunkKind,
    String,
    Option<crate::agent_process::IterableClosedStream>,
    bool,
) {
    (
        SessionUpdateChunkKind::Message,
        text.to_string(),
        None,
        false,
    )
}

#[test]
fn default_idle_is_250ms() {
    let buf = StdoutCoalesceBuf::new(sample_io());
    assert_eq!(idle_of(&buf), STDOUT_COALESCE_IDLE);
    assert_eq!(STDOUT_COALESCE_IDLE, Duration::from_millis(250));
}

#[test]
fn newline_emits_immediately_and_leaves_nothing_pending() {
    let buf = build(sample_io(), Duration::from_secs(30));
    let got = feed(&buf, SessionUpdateChunkKind::Message, "Hi\n");
    assert_eq!(got.len(), 1);
    assert_eq!(got[0].1, "Hi");
    assert!(!pending(&buf));
}

#[test]
fn flush_publishes_partial_line_before_idle() {
    let buf = build(sample_io(), Duration::from_secs(30));
    assert!(feed(&buf, SessionUpdateChunkKind::Message, "Hi").is_empty());
    let got = flush(&buf);
    assert_eq!(got.len(), 1);
    assert_eq!(got[0].1, "Hi");
    assert!(!pending(&buf));
    assert!(recorded(&buf).is_empty());
}

#[test]
fn partial_line_flushes_once_idle_elapses() {
    let idle = Duration::from_millis(80);
    let buf = build(sample_io(), idle);
    assert!(feed(&buf, SessionUpdateChunkKind::Message, "Hello").is_empty());
    thread::sleep(Duration::from_millis(30));
    assert!(pending(&buf), "30ms is inside the idle window");
    let started = Instant::now();
    while pending(&buf) {
        assert!(
            started.elapsed() < Duration::from_millis(400),
            "buffer still held after {:?}",
            started.elapsed()
        );
        thread::sleep(Duration::from_millis(10));
    }
    assert_eq!(recorded(&buf), vec!["Hello".to_string()]);
}

#[test]
fn latest_character_restarts_idle_window() {
    let buf = build(sample_io(), Duration::from_millis(200));
    feed(&buf, SessionUpdateChunkKind::Message, "ab");
    thread::sleep(Duration::from_millis(80));
    assert!(pending(&buf), "still inside the idle window");
    feed(&buf, SessionUpdateChunkKind::Message, "c");
    thread::sleep(Duration::from_millis(80));
    assert!(
        pending(&buf),
        "the idle window starts again at the latest character"
    );
    let started = Instant::now();
    while pending(&buf) {
        assert!(
            started.elapsed() < Duration::from_millis(500),
            "held {:?}",
            started.elapsed()
        );
        thread::sleep(Duration::from_millis(15));
    }
    assert_eq!(recorded(&buf).concat(), "abc");
}

#[test]
fn feed_and_write_prints_a_finished_line() {
    enable_stdout_capture();
    let buf = StdoutCoalesceBuf::new(sample_io());
    buf.feed_and_write(SessionUpdateChunkKind::Message, "Hi\n");
    let out = take_captured_stdout();
    assert!(out.contains("Hi"), "{out}");
    assert!(!pending(&buf));
}

#[test]
fn flush_and_write_prints_a_partial_line() {
    let buf = build(sample_io(), Duration::from_secs(30));
    enable_stdout_capture();
    buf.feed_and_write(SessionUpdateChunkKind::Message, "Yo");
    assert!(take_captured_stdout().is_empty());
    enable_stdout_capture();
    buf.flush_and_write();
    let out = take_captured_stdout();
    assert!(out.contains("Yo"), "{out}");
    assert!(!pending(&buf));
}

#[test]
fn drop_does_not_wait_out_the_idle_window() {
    let buf = build(sample_io(), Duration::from_secs(30));
    feed(&buf, SessionUpdateChunkKind::Message, "x");
    let started = Instant::now();
    drop(buf);
    assert!(started.elapsed() < Duration::from_millis(500));
}

#[test]
fn cap_still_emits_without_waiting() {
    let buf = build(sample_io(), Duration::from_secs(30));
    let chunk = "a".repeat(125);
    let got = feed(&buf, SessionUpdateChunkKind::Message, &chunk);
    assert_eq!(got.len(), 1);
    assert_eq!(got[0].1, chunk);
    assert!(!pending(&buf));
}

#[test]
fn write_coalesced_emissions_prints_message() {
    enable_stdout_capture();
    write_coalesced_emissions(sample_io(), vec![message("Hi")]);
    let shown = take_captured_stdout();
    assert!(shown.contains("Hi"), "{shown}");
}

#[test]
fn write_coalesced_emissions_skips_hidden_thought() {
    enable_stdout_capture();
    write_coalesced_emissions(
        sample_io(),
        vec![(
            SessionUpdateChunkKind::Thought,
            "secret".into(),
            None,
            false,
        )],
    );
    assert!(take_captured_stdout().is_empty());
}

#[test]
fn write_coalesced_emissions_skips_when_not_teeing() {
    let mut io = sample_io();
    io.no_tee = true;
    enable_stdout_capture();
    write_coalesced_emissions(io, vec![message("hidden")]);
    assert!(take_captured_stdout().is_empty());
}

#[test]
fn write_coalesced_emissions_prints_raw_message() {
    let mut io = sample_io();
    io.raw_output = true;
    enable_stdout_capture();
    write_coalesced_emissions(io, vec![message("raw")]);
    let raw = take_captured_stdout();
    assert!(raw.contains("raw"), "{raw}");
}

#[test]
fn write_coalesced_emissions_skips_empty_line() {
    enable_stdout_capture();
    write_coalesced_emissions(sample_io(), vec![message("")]);
    assert!(take_captured_stdout().is_empty());
}

#[test]
fn shown_thought_is_written() {
    let mut io = sample_io();
    io.show_thoughts_on_stdout = true;
    enable_stdout_capture();
    write_coalesced_emissions(
        io,
        vec![(
            SessionUpdateChunkKind::Thought,
            "ponder".into(),
            None,
            false,
        )],
    );
    let out = take_captured_stdout();
    assert!(out.contains("ponder"), "{out}");
}
