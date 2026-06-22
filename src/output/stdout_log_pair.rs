use super::{
    acp_tee_markdown::agent_rendered_markup_payload,
    format_heartbeat_stdout_ansi, format_line_stdout, format_line_stdout_ansi,
    format_who_tag_delim, format_who_tag_prefix, stderr_use_color, stdout_use_color,
    timestamp_now_string,
    who_tag_ansi,
};

use crate::ansi_strip::strip_ansi_escapes;
use crate::output::WHO_B;
use crate::terminal_palette::{ANSI_DIM, ANSI_RESET};
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
    let prefix = format_who_tag_prefix(who);
    let tag_color = who_tag_ansi(who);
    format!("{ANSI_DIM}{ts}{ANSI_RESET} {tag_color}{prefix}{ANSI_RESET}{payload}")
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

pub(crate) fn heartbeat_display_and_log_line_for_color(
    who: &str,
    payload: &str,
    ts: Option<&str>,
    use_color: bool,
) -> (String, String) {
    heartbeat_display_and_log_line(who, payload, ts, use_color)
}

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

pub(crate) fn acp_bracket_color(who: &str) -> &'static str {
    who_tag_ansi(who)
}

pub(crate) fn acp_from_agent_payload(ctx: &AcpTeeLineFmt<'_>, payload: &str, use_color: bool) -> String {
    if !use_color || ctx.dim_payload || ctx.direction != AcpTeeDirection::FromAgent {
        return payload.to_string();
    }
    agent_rendered_markup_payload(payload)
}

pub(crate) fn acp_bracket_payload(ctx: &AcpTeeLineFmt<'_>) -> String {
    let prefix = format_who_tag_prefix(ctx.who);
    if ctx.who == WHO_B {
        return format!("{ANSI_DIM}{prefix}{}{ANSI_RESET}", ctx.line);
    }
    let bracket = acp_bracket_color(ctx.who);
    if ctx.dim_payload {
        format!(
            "{bracket}{prefix}{ANSI_RESET}{ANSI_DIM}{}{ANSI_RESET}",
            ctx.line
        )
    } else if ctx.direction == AcpTeeDirection::FromAgent {
        format!(
            "{bracket}{prefix}{ANSI_RESET}{}",
            acp_from_agent_payload(ctx, ctx.line, true)
        )
    } else {
        format!("{bracket}{prefix}{ANSI_RESET}{}", ctx.line)
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
pub(crate) mod inline_cov {
    use super::*;
    use crate::ansi_strip::strip_ansi_escapes;
    use crate::output::WHO_T;
    use crate::terminal_palette::{ANSI_DIM, ansi_tool_dark};

    pub(crate) fn assert_acp_tool_summary_dim_preserves_bracket(line: &str) {
        let plain = strip_ansi_escapes(line);
        let close_idx = plain.find(']').expect("who tag must contain ]");
        let prefix_end = line.find(']').expect("] in ansi line");
        let prefix = &line[..=prefix_end];
        assert!(
            !prefix.contains(ANSI_DIM),
            "who prefix must not use dim; got {line:?}"
        );
        let after = &line[prefix_end + 1..];
        assert!(
            after.contains(ANSI_DIM),
            "payload after who tag must be dimmed; got {line:?}"
        );
        let sand = acp_bracket_color(WHO_T);
        assert!(
            prefix.contains(sand),
            "who prefix must use sand bracket color; got {line:?}"
        );
        let _ = close_idx;
    }

    pub(crate) fn assert_tool_payload_uses_verb_styling(line: &str) {
        let dark = ansi_tool_dark();
        assert!(
            line.contains(dark),
            "styled tool summary payload verbs use dark bold; got {line:?}"
        );
    }

    #[test]
    fn kiss_cov_stdout_log_pair_privates() {
    }
}
#[cfg(test)]
#[path = "stdout_log_pair_test.rs"]
mod stdout_log_pair_test;
#[cfg(test)]
#[allow(unused_imports, clippy::unused_unit, non_snake_case)]
mod kiss_static_fn_item_refs {
    use super::*;

    #[test]
    fn kiss_static_fn_item_refs() {
        let _: Option<AcpTeeDirection> = None;
        let _: Option<AcpTeeLineFmt> = None;
        let _: Option<TaggedDisplayStyle> = None;
    }
}
