use std::sync::atomic::{AtomicU32, Ordering};

use pi::sdk::AgentEvent;

pub(crate) static FAKE_BACKEND_ERROR_TURNS: AtomicU32 = AtomicU32::new(0);

const FAKE_BACKEND_ERROR_TURN_LIMIT: u32 = 20;

pub(crate) fn fake_events_for_prompt(prompt: &str, provider: &str, model: &str) -> Vec<AgentEvent> {
    use pi::model::{AssistantMessage, ContentBlock, Message, TextContent, Usage};

    if prompt.contains("PERSISTENT_BACKEND_ERROR") {
        return vec![backend_error_agent_end(prompt, false)];
    }
    if prompt.contains("VARYING_BACKEND_ERROR") {
        return vec![backend_error_agent_end(prompt, true)];
    }
    if prompt.contains("EMPTY_ASSISTANT_RESULT") {
        return vec![empty_agent_end(prompt)];
    }
    if prompt.contains("TOOL_ERROR_READ") {
        return tool_error_events(prompt);
    }
    if prompt.contains("OUTPUT_CAP_THINKING") {
        return vec![output_cap_agent_end(prompt)];
    }
    let early = prompt.contains("AGENT_END_BEFORE_ACK");
    let text = if early {
        "early-end".to_string()
    } else {
        format!("echo:{prompt}")
    };
    let usage = if early {
        Usage {
            input: 1,
            output: 1,
            ..Usage::default()
        }
    } else {
        Usage {
            input: 3,
            output: 2,
            ..Usage::default()
        }
    };
    let assistant = AssistantMessage {
        content: vec![ContentBlock::Text(TextContent::new(text.clone()))],
        provider: provider.to_string(),
        model: model.to_string(),
        usage,
        ..AssistantMessage::default()
    };
    let mut events = Vec::new();
    if !early {
        events.extend(streamed_hello_events(&text, &assistant));
    }
    events.push(AgentEvent::AgentEnd {
        session_id: "fake".into(),
        messages: vec![
            Message::User(pi::model::UserMessage {
                content: pi::model::UserContent::Text(prompt.to_string()),
                timestamp: 0,
            }),
            Message::assistant(assistant),
        ],
        error: None,
    });
    events
}

fn backend_error_agent_end(prompt: &str, vary: bool) -> AgentEvent {
    let turn = FAKE_BACKEND_ERROR_TURNS.fetch_add(1, Ordering::SeqCst) + 1;
    let error = if turn > FAKE_BACKEND_ERROR_TURN_LIMIT {
        "you've hit your usage limit (fake backend error turn limit)".to_string()
    } else if vary {
        format!("Compute error (turn {turn})")
    } else {
        "Compute error".to_string()
    };
    AgentEvent::AgentEnd {
        session_id: "fake".into(),
        messages: vec![pi::model::Message::User(pi::model::UserMessage {
            content: pi::model::UserContent::Text(prompt.to_string()),
            timestamp: 0,
        })],
        error: Some(error),
    }
}

fn empty_agent_end(prompt: &str) -> AgentEvent {
    AgentEvent::AgentEnd {
        session_id: "fake".into(),
        messages: vec![pi::model::Message::User(pi::model::UserMessage {
            content: pi::model::UserContent::Text(prompt.to_string()),
            timestamp: 0,
        })],
        error: None,
    }
}

pub(crate) const FAKE_TOOL_ERROR_TEXT: &str = "missing field `path`";

fn tool_error_events(prompt: &str) -> Vec<AgentEvent> {
    use pi::model::{ContentBlock, TextContent};

    vec![
        AgentEvent::ToolExecutionStart {
            tool_call_id: "t1".into(),
            tool_name: "read".into(),
            args: serde_json::json!({ "file": "plan.md" }),
        },
        AgentEvent::ToolExecutionEnd {
            tool_call_id: "t1".into(),
            tool_name: "read".into(),
            result: pi::sdk::ToolOutput {
                content: vec![ContentBlock::Text(TextContent::new(FAKE_TOOL_ERROR_TEXT))],
                details: None,
                is_error: true,
            },
            is_error: true,
        },
        empty_agent_end(prompt),
    ]
}

pub(crate) const FAKE_OUTPUT_CAP_TOKENS: u64 = 4096;

fn output_cap_agent_end(prompt: &str) -> AgentEvent {
    use pi::model::{AssistantMessage, ContentBlock, Message, ThinkingContent, Usage};

    let assistant = AssistantMessage {
        content: vec![ContentBlock::Thinking(ThinkingContent {
            thinking: "hmm".into(),
            thinking_signature: None,
        })],
        usage: Usage {
            input: 10,
            output: FAKE_OUTPUT_CAP_TOKENS,
            ..Usage::default()
        },
        ..AssistantMessage::default()
    };
    AgentEvent::AgentEnd {
        session_id: "fake".into(),
        messages: vec![
            Message::User(pi::model::UserMessage {
                content: pi::model::UserContent::Text(prompt.to_string()),
                timestamp: 0,
            }),
            Message::assistant(assistant),
        ],
        error: None,
    }
}

fn streamed_hello_events(text: &str, assistant: &pi::model::AssistantMessage) -> [AgentEvent; 3] {
    [
        AgentEvent::MessageUpdate {
            message: pi::model::Message::assistant(assistant.clone()),
            assistant_message_event: pi::model::AssistantMessageEvent::TextDelta {
                content_index: 0,
                delta: text.to_string(),
                partial: std::sync::Arc::new(assistant.clone()),
            },
        },
        AgentEvent::ToolExecutionStart {
            tool_call_id: "t1".into(),
            tool_name: "ls".into(),
            args: serde_json::Value::Null,
        },
        AgentEvent::ToolExecutionEnd {
            tool_call_id: "t1".into(),
            tool_name: "ls".into(),
            result: pi::sdk::ToolOutput {
                content: Vec::new(),
                details: None,
                is_error: false,
            },
            is_error: false,
        },
    ]
}
