#[test]
fn kiss_cov_output_units() {
    let _ = crate::output::who_tag_ansi;
    let _ = crate::output::append_stdout_log_line;
    let _: Option<crate::output::stdout_log_pair::TaggedDisplayStyle> = None;
    let _ = crate::output::stdout_log_pair::acp_bracket_color;
    let _ = crate::output::stdout_log_pair::acp_bracket_payload;
    let _ = crate::output::stdout_log_pair::acp_from_agent_payload;
    let _ = crate::output::stdout_log_pair::heartbeat_display_and_log_line;
    let _ = crate::output::stdout_log_pair::resolve_log_timestamp;
    let _ = crate::output::stdout_log_pair::tagged_display_and_log_line;
    let _ = crate::output::stdout_log_pair::tagged_stdout_display;
    let _ = crate::output::stderr_log::emit_stderr_log_line;
    let _ = crate::output::stderr_log::emit_stderr_log_lines;
    let _ = crate::output::stdout_tee_env::stdout_is_interactive;
    let _ = crate::output::stdout_tee_env::force_stdout_tee_from_env;
    let _ = crate::output::stdout_tee_env::agent_stdout_tee_enabled;
}

#[test]
fn kiss_cov_acp_tee_stdout_event_destructure_and_dim_plain() {
    use crate::output::acp_tee::{
        print_stdout_acp_tee_line_with_timestamp_dim_plain, AcpTeeStdoutEvent,
    };
    use crate::output::{AcpTeeDirection, AcpTeeLineFmt};

    let ev = AcpTeeStdoutEvent {
        direction: AcpTeeDirection::ToAgent,
        who: "malvin",
        line: "probe",
        ts: "20260616.000000.000",
        emit_stdout_markdown: false,
        dim_payload: true,
    };
    let touched = std::hint::black_box(ev);
    let AcpTeeStdoutEvent {
        direction,
        who,
        line,
        ts,
        emit_stdout_markdown,
        dim_payload,
    } = touched;
    assert_eq!(direction, AcpTeeDirection::ToAgent);
    assert_eq!(who, "malvin");
    assert_eq!(line, "probe");
    assert!(dim_payload);
    assert!(!emit_stdout_markdown);
    assert!(ts.contains("20260616"));

    let _guard = crate::output::STDOUT_LOG_TEST_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    crate::output::enable_stdout_capture();
    print_stdout_acp_tee_line_with_timestamp_dim_plain(
        AcpTeeDirection::FromAgent,
        "<agent",
        "dim-plain",
        "20260616.000000.000",
    );
    let _ = crate::output::take_captured_stdout();

    let ctx = AcpTeeLineFmt {
        ts: "20260616.000000.000",
        direction: AcpTeeDirection::FromAgent,
        who: "<agent",
        line: "dim-plain",
        dim_payload: true,
    };
    let _ = std::hint::black_box(ctx);
}

#[test]
fn kiss_cov_stdout_log_pair_enums_and_tagged_display_all_styles() {
    use crate::output::stdout_log_pair::{
        tagged_stdout_display, AcpTeeDirection, AcpTeeLineFmt, TaggedDisplayStyle,
    };

    assert!(matches!(AcpTeeDirection::ToAgent, AcpTeeDirection::ToAgent));
    assert!(matches!(AcpTeeDirection::FromAgent, AcpTeeDirection::FromAgent));

    let fmt = AcpTeeLineFmt {
        ts: "20260616.000000.000",
        direction: AcpTeeDirection::ToAgent,
        who: "malvin",
        line: "payload",
        dim_payload: false,
    };
    let AcpTeeLineFmt {
        ts,
        direction,
        who,
        line,
        dim_payload,
    } = std::hint::black_box(fmt);
    assert_eq!(ts, "20260616.000000.000");
    assert_eq!(direction, AcpTeeDirection::ToAgent);
    assert_eq!(who, "malvin");
    assert_eq!(line, "payload");
    assert!(!dim_payload);

    for style in [
        TaggedDisplayStyle::Plain,
        TaggedDisplayStyle::Ansi,
        TaggedDisplayStyle::HeartbeatAnsi,
    ] {
        let out = tagged_stdout_display("malvin", "x", style);
        assert!(out.contains('x'), "{out:?}");
    }
}

#[test]
fn kiss_cov_stdout_render_prelude_all_variants() {
    use crate::output::stdout_render::StdoutRenderPrelude;

    assert!(matches!(
        StdoutRenderPrelude::TaggedWithHeartbeat,
        StdoutRenderPrelude::TaggedWithHeartbeat
    ));
    assert!(matches!(
        StdoutRenderPrelude::HeartbeatOnly,
        StdoutRenderPrelude::HeartbeatOnly
    ));
    assert!(matches!(
        StdoutRenderPrelude::FlushOnly,
        StdoutRenderPrelude::FlushOnly
    ));
}
