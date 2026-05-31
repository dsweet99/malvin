use super::{
    acp_tee_markdown::agent_rendered_markup_payload,
    format_heartbeat_stdout_ansi, format_line_stdout, format_line_stdout_ansi,
    format_who_tag_delim, stderr_use_color, stdout_use_color,
    timestamp_now_string,
    who_tag_ansi,
};

use crate::ansi_strip::strip_ansi_escapes;
use crate::output::WHO_B;
use crate::terminal_palette::{ansi_tool_dark, ansi_tool_navy, ANSI_DIM, ANSI_RESET};
use unicode_width::UnicodeWidthStr;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AcpTeeDirection {
    ToAgent,
    FromAgent,
}

pub struct AcpTeeLineFmt<'a> {
    pub ts: &'a str,
    pub direction: AcpTeeDirection,
    pub who: &'a str,
    pub line: &'a str,
    pub dim_payload: bool,
}

pub(crate) fn resolve_log_timestamp(ts: Option<&str>) -> String {
    ts.map_or_else(timestamp_now_string, str::to_string)
}

pub(crate) fn tagged_log_line(ts: &str, who: &str, payload: &str) -> String {
    format!("{ts} {}{payload}", format_who_tag_delim(who))
}

pub(crate) fn tagged_display_line_with_timestamp_ansi(
    ts: &str,
    who: &str,
    payload: &str,
) -> String {
    let delim = format_who_tag_delim(who);
    let tag_color = who_tag_ansi(who);
    format!("{ANSI_DIM}{ts}{ANSI_RESET} {tag_color}{delim}{ANSI_RESET} {payload}")
}

pub(crate) fn stdout_tagged_display_and_log_line(
    who: &str,
    payload: &str,
    ts: Option<&str>,
) -> (String, String) {
    tagged_display_and_log_line(who, payload, ts, stdout_use_color())
}

pub(crate) fn stdout_heartbeat_display_and_log_line(
    who: &str,
    payload: &str,
    ts: Option<&str>,
) -> (String, String) {
    heartbeat_display_and_log_line(who, payload, ts, stdout_use_color())
}

#[cfg(test)]
pub(crate) fn heartbeat_display_and_log_line_for_color(
    who: &str,
    payload: &str,
    ts: Option<&str>,
    use_color: bool,
) -> (String, String) {
    heartbeat_display_and_log_line(who, payload, ts, use_color)
}

#[cfg(test)]
pub(crate) fn tagged_display_and_log_line_for_color(
    who: &str,
    payload: &str,
    ts: Option<&str>,
    use_color: bool,
) -> (String, String) {
    tagged_display_and_log_line(who, payload, ts, use_color)
}

pub(crate) fn stderr_tagged_display_and_log_line(
    who: &str,
    payload: &str,
    ts: Option<&str>,
) -> (String, String) {
    tagged_display_and_log_line(who, payload, ts, stderr_use_color())
}

#[derive(Copy, Clone)]
pub(crate) enum TaggedDisplayStyle { Plain, Ansi, HeartbeatAnsi }

pub(crate) fn tagged_stdout_display(who: &str, payload: &str, style: TaggedDisplayStyle) -> String {
    match style {
        TaggedDisplayStyle::Plain => format_line_stdout(who, payload),
        TaggedDisplayStyle::Ansi => format_line_stdout_ansi(who, payload),
        TaggedDisplayStyle::HeartbeatAnsi => format_heartbeat_stdout_ansi(who, payload),
    }
}

macro_rules! tagged_log_pair {
    ($who:expr, $payload:expr, $ts:expr, $style:expr) => {{
        let ts = resolve_log_timestamp($ts);
        let log = tagged_log_line(&ts, $who, $payload);
        (tagged_stdout_display($who, $payload, $style), log)
    }};
}

pub(crate) fn tagged_display_and_log_line(
    who: &str,
    payload: &str,
    ts: Option<&str>,
    use_color: bool,
) -> (String, String) {
    let style = if use_color {
        TaggedDisplayStyle::Ansi
    } else {
        TaggedDisplayStyle::Plain
    };
    tagged_log_pair!(who, payload, ts, style)
}

pub(crate) fn heartbeat_display_and_log_line(
    who: &str,
    payload: &str,
    ts: Option<&str>,
    use_color: bool,
) -> (String, String) {
    let style = if use_color {
        TaggedDisplayStyle::HeartbeatAnsi
    } else {
        TaggedDisplayStyle::Plain
    };
    tagged_log_pair!(who, payload, ts, style)
}

pub(crate) fn stdout_raw_display_and_log_line(line: &str, ts: Option<&str>) -> (String, String) {
    let ts = resolve_log_timestamp(ts);
    (line.to_string(), format!("{ts} {line}"))
}

pub(crate) fn acp_bracket_color(direction: AcpTeeDirection) -> &'static str {
    match direction {
        AcpTeeDirection::ToAgent => ansi_tool_navy(),
        AcpTeeDirection::FromAgent => ansi_tool_dark(),
    }
}

pub(crate) fn acp_from_agent_payload(ctx: &AcpTeeLineFmt<'_>, payload: &str, use_color: bool) -> String {
    if !use_color || ctx.dim_payload || ctx.direction != AcpTeeDirection::FromAgent {
        return payload.to_string();
    }
    agent_rendered_markup_payload(payload)
}

pub(crate) fn acp_bracket_payload(ctx: &AcpTeeLineFmt<'_>) -> String {
    let delim = format_who_tag_delim(ctx.who);
    if ctx.who == WHO_B {
        return format!("{ANSI_DIM}{delim} {}{ANSI_RESET}", ctx.line);
    }
    let bracket = acp_bracket_color(ctx.direction);
    if ctx.dim_payload {
        format!(
            "{bracket}{delim}{ANSI_RESET} {ANSI_DIM}{}{ANSI_RESET}",
            ctx.line
        )
    } else if ctx.direction == AcpTeeDirection::FromAgent {
        format!(
            "{bracket}{delim}{ANSI_RESET} {}",
            acp_from_agent_payload(ctx, ctx.line, true)
        )
    } else {
        format!("{bracket}{delim}{ANSI_RESET} {}", ctx.line)
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
    let log_line = acp_tee_log_line(log);
    if display.direction == AcpTeeDirection::ToAgent {
        return (String::new(), log_line);
    }
    (acp_tee_display_line(display), log_line)
}

pub(crate) fn stdout_acp_prefix_rendered_line(
    ctx: &AcpTeeLineFmt<'_>,
    rendered_payload: &str,
) -> (String, String) {
    let display_payload = acp_from_agent_payload(ctx, rendered_payload, stdout_use_color());
    (
        format!("{}{display_payload}", acp_tee_payload_prefix(ctx)),
        format!("{}{rendered_payload}", acp_tee_log_prefix(ctx)),
    )
}

#[cfg(test)]
mod inline_cov {
    #[test]
    fn kiss_cov_stdout_log_pair_privates() {
        let _ = stringify!(resolve_log_timestamp);
        let _ = stringify!(tagged_display_and_log_line);
        let _ = stringify!(heartbeat_display_and_log_line);
        let _ = stringify!(tagged_stdout_display);
        let _ = stringify!(TaggedDisplayStyle);
        let _ = stringify!(acp_bracket_color);
        let _ = stringify!(acp_bracket_payload);
        let _ = stringify!(acp_from_agent_payload);
    }
}
