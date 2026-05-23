use crate::output::acp_tee::{
    AcpTeeDirection, AcpTeeLineFmt, AcpTeeStdoutEvent, acp_tee_display_line, acp_tee_log_line,
    print_stdout_acp_tee_line_with_timestamp,
};
use crate::output::terminal_wrap::{line_wrap_for_prefix_len, wrap_words_bounded};
use crate::output::{
    STDOUT_LOG_TEST_LOCK, init_stdout_style, is_log_timestamp_token, print_stdout_line,
    print_stdout_raw_line, print_stdout_raw_line_with_ts, set_stdout_log_path,
};

struct StdoutLogCapture {
    _guard: std::sync::MutexGuard<'static, ()>,
    _tmp: tempfile::TempDir,
    path: std::path::PathBuf,
}

impl StdoutLogCapture {
    fn open() -> Self {
        let guard = STDOUT_LOG_TEST_LOCK
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let tmp = tempfile::tempdir().expect("tempdir");
        let path = tmp.path().join("stdout.log");
        set_stdout_log_path(Some(path.clone()));
        Self {
            _guard: guard,
            _tmp: tmp,
            path,
        }
    }

    fn finish(self) -> String {
        set_stdout_log_path(None);
        std::fs::read_to_string(self.path).expect("read")
    }
}

fn assert_wrapped_log_shares_timestamp(text: &str) {
    let lines: Vec<&str> = text.lines().collect();
    assert!(
        lines.len() > 1,
        "expected wrapped output to produce multiple stdout.log lines; got {text:?}"
    );
    let first_ts = lines[0].split_whitespace().next().expect("timestamp token");
    assert!(is_log_timestamp_token(first_ts));
    for line in &lines[1..] {
        let ts = line.split_whitespace().next().expect("timestamp token");
        assert_eq!(
            ts, first_ts,
            "wrapped segments of one logical line must share one timestamp; got {text:?}"
        );
    }
}

#[test]
fn print_stdout_raw_line_honors_shared_trace_timestamp() {
    let trace_ts = "20260413.121314.015";
    let (_, expected_log) =
        crate::output::stdout_log_pair::stdout_raw_display_and_log_line("probe", Some(trace_ts));
    let text = {
        let cap = StdoutLogCapture::open();
        print_stdout_raw_line_with_ts("probe", Some(trace_ts));
        cap.finish()
    };
    assert_eq!(
        text.trim_end(),
        expected_log,
        "raw trace tee should reuse trace timestamp on stdout.log; got {text:?}"
    );
}

#[test]
#[allow(unsafe_code)]
fn raw_unprefixed_wrap_shares_one_timestamp_per_logical_line() {
    let trace_ts = "20260413.121314.015";
    let line = "x".repeat(200);
    let (max_payload, wrap) = line_wrap_for_prefix_len(0, &line, true);
    assert!(wrap, "fixture line should wrap");
    let segments = wrap_words_bounded(max_payload, &line);
    assert!(
        segments.len() > 1,
        "expected multiple wrap segments; got {}",
        segments.len()
    );
    let text = {
        let cap = StdoutLogCapture::open();
        for seg in segments {
            print_stdout_raw_line_with_ts(&seg, Some(trace_ts));
        }
        cap.finish()
    };
    assert_wrapped_log_shares_timestamp(&text);
}

#[test]
fn raw_stdout_log_line_is_timestamped() {
    let text = {
        let cap = StdoutLogCapture::open();
        print_stdout_raw_line("raw probe line");
        cap.finish()
    };
    let line = text.lines().next().expect("one log line");
    assert!(
        is_log_timestamp_token(line.split_whitespace().next().unwrap_or("")),
        "stdout.log raw lines must carry wall-clock prefix per plan; got {line:?}"
    );
}

#[test]
#[allow(unsafe_code)]
fn wrapped_tagged_stdout_log_shares_one_timestamp_per_logical_line() {
    let prev_cols = std::env::var("COLUMNS").ok();
    unsafe {
        std::env::set_var("COLUMNS", "40");
    }
    let text = {
        let cap = StdoutLogCapture::open();
        init_stdout_style(false);
        print_stdout_line("wrap", "segment ".repeat(20).trim());
        cap.finish()
    };
    unsafe {
        match prev_cols {
            Some(v) => std::env::set_var("COLUMNS", v),
            None => std::env::remove_var("COLUMNS"),
        }
    }
    assert_wrapped_log_shares_timestamp(&text);
}

#[test]
fn acp_tee_live_display_and_stdout_log_split_timestamp_prefix() {
    let ts = "20260413.121314.015";
    let ctx = AcpTeeLineFmt {
        ts,
        direction: AcpTeeDirection::FromAgent,
        who: "<review",
        line: "hello",
        dim_payload: false,
    };
    let log_line = {
        let cap = StdoutLogCapture::open();
        init_stdout_style(false);
        print_stdout_acp_tee_line_with_timestamp(&AcpTeeStdoutEvent {
            direction: ctx.direction,
            who: ctx.who,
            line: ctx.line,
            ts: ctx.ts,
            emit_stdout_markdown: false,
            dim_payload: false,
        });
        cap.finish().trim_end().to_string()
    };
    assert_eq!(log_line, acp_tee_log_line(&ctx));
    assert!(
        !acp_tee_display_line(&ctx).starts_with("20260413"),
        "live display must omit wall-clock prefix"
    );
    assert!(
        is_log_timestamp_token(log_line.split_whitespace().next().unwrap_or("")),
        "stdout.log must be timestamped; got {log_line:?}"
    );
}

#[test]
fn acp_tee_markdown_prefix_rendered_line_splits_display_and_log_timestamps() {
    use crate::ansi_strip::strip_ansi_escapes;

    let ts = "20260413.121314.015";
    let ctx = AcpTeeLineFmt {
        ts,
        direction: AcpTeeDirection::FromAgent,
        who: "<md",
        line: "",
        dim_payload: false,
    };
    init_stdout_style(true);
    let (display, log) =
        crate::output::stdout_log_pair::stdout_acp_prefix_rendered_line(&ctx, "**bold**");
    assert!(log.starts_with(ts));
    assert!(
        !strip_ansi_escapes(&display).starts_with(ts),
        "markdown tee display must omit wall-clock prefix; got {display:?}"
    );
}
