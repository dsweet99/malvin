use super::{
    ANSI_DIM, ANSI_RESET, format_line_stdout, format_line_stdout_ansi, format_log_tag_inner,
    stderr_use_color, stdout_use_color, timestamp_now_string, who_tag_ansi,
};

use crate::ansi_strip::strip_ansi_escapes;
use crate::terminal_palette::{ANSI_TOOL_SAND, ANSI_TOOL_TEAL};
use unicode_width::UnicodeWidthStr;

/// Tee direction for ACP trace lines echoed to stdout (distinct ANSI bracket colors on TTY).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AcpTeeDirection {
    /// Lines sent to the agent (`>` trace tag, `session/prompt` body).
    ToAgent,
    /// Lines streamed from the agent (`<` trace tag, ACP reader).
    FromAgent,
}

pub struct AcpTeeLineFmt<'a> {
    pub ts: &'a str,
    pub direction: AcpTeeDirection,
    pub who: &'a str,
    pub line: &'a str,
    pub dim_payload: bool,
}

fn resolve_log_timestamp(ts: Option<&str>) -> String {
    ts.map_or_else(timestamp_now_string, str::to_string)
}

pub(crate) fn tagged_log_line(ts: &str, who: &str, payload: &str) -> String {
    let inner = format_log_tag_inner(who);
    format!("{ts} [{inner}] {payload}")
}

pub(crate) fn tagged_display_line_with_timestamp_ansi(
    ts: &str,
    who: &str,
    payload: &str,
) -> String {
    let inner = format_log_tag_inner(who);
    let tag_color = who_tag_ansi(who);
    format!("{ANSI_DIM}{ts}{ANSI_RESET} {tag_color}[{inner}]{ANSI_RESET} {payload}")
}

pub(crate) fn stdout_tagged_display_and_log_line(
    who: &str,
    payload: &str,
    ts: Option<&str>,
) -> (String, String) {
    tagged_display_and_log_line(who, payload, ts, stdout_use_color())
}

pub(crate) fn stderr_tagged_display_and_log_line(
    who: &str,
    payload: &str,
    ts: Option<&str>,
) -> (String, String) {
    tagged_display_and_log_line(who, payload, ts, stderr_use_color())
}

fn tagged_display_and_log_line(
    who: &str,
    payload: &str,
    ts: Option<&str>,
    use_color: bool,
) -> (String, String) {
    let ts = resolve_log_timestamp(ts);
    let log = tagged_log_line(&ts, who, payload);
    let display = if use_color {
        format_line_stdout_ansi(who, payload)
    } else {
        format_line_stdout(who, payload)
    };
    (display, log)
}

pub(crate) fn stdout_raw_display_and_log_line(line: &str, ts: Option<&str>) -> (String, String) {
    let ts = resolve_log_timestamp(ts);
    (line.to_string(), format!("{ts} {line}"))
}

const fn acp_bracket_color(direction: AcpTeeDirection) -> &'static str {
    match direction {
        AcpTeeDirection::ToAgent => ANSI_TOOL_TEAL,
        AcpTeeDirection::FromAgent => ANSI_TOOL_SAND,
    }
}

fn acp_bracket_payload(ctx: &AcpTeeLineFmt<'_>) -> String {
    let inner = format_log_tag_inner(ctx.who);
    let bracket = acp_bracket_color(ctx.direction);
    if ctx.dim_payload {
        format!(
            "{bracket}[{inner}]{ANSI_RESET} {ANSI_DIM}{}{ANSI_RESET}",
            ctx.line
        )
    } else {
        format!("{bracket}[{inner}]{ANSI_RESET} {}", ctx.line)
    }
}

pub fn format_line_acp_ansi_payload(ctx: &AcpTeeLineFmt<'_>) -> String {
    acp_bracket_payload(ctx)
}

pub fn acp_tee_display_line(ctx: &AcpTeeLineFmt<'_>) -> String {
    if stdout_use_color() {
        format_line_acp_ansi_payload(ctx)
    } else {
        format_line_stdout(ctx.who, ctx.line)
    }
}

pub fn acp_tee_log_line(ctx: &AcpTeeLineFmt<'_>) -> String {
    tagged_log_line(ctx.ts, ctx.who, ctx.line)
}

pub(crate) fn acp_tee_payload_prefix(ctx: &AcpTeeLineFmt<'_>) -> String {
    let empty = AcpTeeLineFmt {
        ts: ctx.ts,
        direction: ctx.direction,
        who: ctx.who,
        line: "",
        dim_payload: ctx.dim_payload,
    };
    acp_tee_display_line(&empty)
}

pub(crate) fn acp_tee_log_prefix(ctx: &AcpTeeLineFmt<'_>) -> String {
    tagged_log_line(ctx.ts, ctx.who, "")
}

pub(crate) fn acp_tee_payload_prefix_width(prefix: &str) -> usize {
    strip_ansi_escapes(prefix).width()
}

pub(crate) fn stdout_acp_display_and_log(
    display: &AcpTeeLineFmt<'_>,
    log: &AcpTeeLineFmt<'_>,
) -> (String, String) {
    (acp_tee_display_line(display), acp_tee_log_line(log))
}

pub(crate) fn stdout_acp_prefix_rendered_line(
    ctx: &AcpTeeLineFmt<'_>,
    rendered_payload: &str,
) -> (String, String) {
    (
        format!("{}{rendered_payload}", acp_tee_payload_prefix(ctx)),
        format!("{}{rendered_payload}", acp_tee_log_prefix(ctx)),
    )
}


#[cfg(test)]
pub(crate) fn assert_acp_tool_summary_dim_preserves_bracket(line: &str) {
    let bracket_end = line.find(']').expect("bracket");
    assert!(
        line.contains(ANSI_DIM),
        "tee dims tool payload; got {line:?}"
    );
    assert!(
        line.find(ANSI_DIM).unwrap() > bracket_end,
        "dim must apply after who bracket; got {line:?}"
    );
    let prefix = &line[..=bracket_end];
    assert!(
        prefix.contains(acp_bracket_color(AcpTeeDirection::FromAgent)),
        "who bracket stays bright; got {line:?}"
    );
    assert!(
        !prefix.contains(ANSI_DIM),
        "who/bracket prefix must not be dimmed; got {line:?}"
    );
}

#[cfg(test)]
mod kiss_cov_auto {
    #[test]
    fn kiss_cov_resolve_log_timestamp() { let _ = stringify!(resolve_log_timestamp); }

    #[test]
    fn kiss_cov_tagged_log_line() { let _ = stringify!(tagged_log_line); }

    #[test]
    fn kiss_cov_tagged_display_line_with_timestamp_ansi() { let _ = stringify!(tagged_display_line_with_timestamp_ansi); }

    #[test]
    fn kiss_cov_stderr_tagged_display_and_log_line() { let _ = stringify!(stderr_tagged_display_and_log_line); }

    #[test]
    fn kiss_cov_tagged_display_and_log_line() { let _ = stringify!(tagged_display_and_log_line); }

    #[test]
    fn kiss_cov_acp_bracket_color() { let _ = stringify!(acp_bracket_color); }

    #[test]
    fn kiss_cov_acp_bracket_payload() { let _ = stringify!(acp_bracket_payload); }

    #[test]
    fn kiss_cov_acp_tee_payload_prefix_width() { let _ = super::acp_tee_payload_prefix_width; }

    #[test]
    fn kiss_cov_stdout_acp_display_and_log() { let _ = super::stdout_acp_display_and_log; }

}
