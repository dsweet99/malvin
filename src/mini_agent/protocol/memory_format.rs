//! Header / History / Previous / New-request assembly and NEW_HISTORY / RESPONSE parsing.

use crate::llm_transport::{ChatMessage, ChatRole};

pub const NEW_HISTORY_HEADING: &str = "## NEW_HISTORY";
pub const RESPONSE_HEADING: &str = "## RESPONSE";

/// Chat-state History label (not workflow `header.md` log-file History).
pub const CHAT_STATE_HISTORY_LABEL: &str = "### Chat-state History";

pub const PREVIOUS_RESPONSE_LABEL: &str = "### Previous response";

/// Shape cue after a failed NEW_HISTORY / RESPONSE parse.
pub const SECTION_SHAPE_NUDGE: &str = "Emit sections in this exact order only:\n\n## NEW_HISTORY\n\
<consolidated chat-state history>\n\n## RESPONSE\n\
<answer for the new request>\n\nDo not omit either heading.";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedTurn {
    pub new_history: String,
    pub response: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SectionParseError {
    MissingNewHistory,
    MissingResponse,
    WrongOrder,
}

pub(crate) fn section_parse_error_message(err: &SectionParseError) -> String {
    match err {
        SectionParseError::MissingNewHistory => format!("missing {NEW_HISTORY_HEADING} section"),
        SectionParseError::MissingResponse => format!("missing {RESPONSE_HEADING} section"),
        SectionParseError::WrongOrder => {
            format!("{NEW_HISTORY_HEADING} must appear before {RESPONSE_HEADING}")
        }
    }
}

impl SectionParseError {
    #[must_use]
    pub fn as_message(&self) -> String {
        section_parse_error_message(self)
    }
}

/// Parse assistant wire text: `## NEW_HISTORY` then `## RESPONSE` (fixed order).
///
/// Headings must appear at line starts so a History body that mentions the
/// substring `## RESPONSE` mid-line does not steal the section boundary.
pub fn parse_history_response(content: &str) -> Result<ParsedTurn, SectionParseError> {
    let hist_pos = find_line_heading(content, NEW_HISTORY_HEADING);
    let resp_anywhere = find_line_heading(content, RESPONSE_HEADING);
    let (Some(h), Some(r_any)) = (hist_pos, resp_anywhere) else {
        if hist_pos.is_none() {
            return Err(SectionParseError::MissingNewHistory);
        }
        return Err(SectionParseError::MissingResponse);
    };
    if r_any < h {
        return Err(SectionParseError::WrongOrder);
    }
    let search_from = h + NEW_HISTORY_HEADING.len();
    let Some(r) = find_line_heading(&content[search_from..], RESPONSE_HEADING)
        .map(|rel| search_from + rel)
    else {
        return Err(SectionParseError::MissingResponse);
    };
    let after_hist = h + NEW_HISTORY_HEADING.len();
    let history_body = content[after_hist..r].trim();
    let after_resp = r + RESPONSE_HEADING.len();
    let response_body = content[after_resp..].trim();
    Ok(ParsedTurn {
        new_history: history_body.to_string(),
        response: response_body.to_string(),
    })
}

fn find_line_heading(content: &str, heading: &str) -> Option<usize> {
    if content.starts_with(heading) && heading_ends_cleanly(&content[heading.len()..]) {
        return Some(0);
    }
    let mut search_from = 0;
    while let Some(rel) = content[search_from..].find('\n') {
        let line_start = search_from + rel + 1;
        if content[line_start..].starts_with(heading)
            && heading_ends_cleanly(&content[line_start + heading.len()..])
        {
            return Some(line_start);
        }
        search_from = line_start;
        if search_from >= content.len() {
            break;
        }
    }
    None
}

fn heading_ends_cleanly(after: &str) -> bool {
    after.is_empty()
        || after.starts_with('\n')
        || after.starts_with("\r\n")
        || after.starts_with(' ')
        || after.starts_with('\t')
}

/// Build a wire turn for tests / mocks.
#[must_use]
pub fn format_wire_turn(new_history: &str, response: &str) -> String {
    format!("{NEW_HISTORY_HEADING}\n{new_history}\n\n{RESPONSE_HEADING}\n{response}")
}

#[derive(Debug, Clone, Copy)]
pub struct AssembleInput<'a> {
    pub header: &'a str,
    pub study_act_cue: Option<&'a str>,
    pub history: &'a str,
    pub previous_response: &'a str,
    pub new_request: &'a str,
}

/// Ephemeral wire list: Header (+ cue) / optional History / optional Previous / New request.
#[must_use]
pub fn assemble_completion_messages(input: AssembleInput<'_>) -> Vec<ChatMessage> {
    let mut out = Vec::with_capacity(5);
    out.push(ChatMessage {
        role: ChatRole::System,
        content: input.header.to_string(),
    });
    if let Some(cue) = input.study_act_cue.filter(|c| !c.is_empty()) {
        out.push(ChatMessage {
            role: ChatRole::System,
            content: cue.to_string(),
        });
    }
    if !input.history.trim().is_empty() {
        out.push(ChatMessage {
            role: ChatRole::System,
            content: format!("{CHAT_STATE_HISTORY_LABEL}\n{}", input.history.trim()),
        });
    }
    if !input.previous_response.trim().is_empty() {
        out.push(ChatMessage {
            role: ChatRole::Assistant,
            content: input.previous_response.to_string(),
        });
    }
    out.push(ChatMessage {
        role: ChatRole::User,
        content: input.new_request.to_string(),
    });
    out
}

