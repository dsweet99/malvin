use pi::sdk::AgentEvent;

pub(crate) fn fake_events_for_prompt(prompt: &str) -> Vec<AgentEvent> {
    use pi::model::{AssistantMessage, ContentBlock, Message, TextContent, Usage};

    if prompt.contains("EMPTY_ASSISTANT_RESULT") {
        return vec![empty_agent_end(prompt)];
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
