use pi::sdk::{ContentBlock, Message};
use serde_json::{Value, json};

use crate::bridge_protocol::{BridgeEvent, RunDoneStatus};

#[must_use]
pub(crate) fn map_agent_end(messages: &[Message], error: Option<&str>) -> BridgeEvent {
    let status = if error.is_some() {
        RunDoneStatus::Error
    } else {
        RunDoneStatus::Finished
    };
    BridgeEvent::RunDone {
        status,
        result: last_assistant_text(messages),
        usage: aggregate_usage(messages),
        error: error.map(str::to_string),
        duration_ms: None,
    }
}

fn last_assistant_text(messages: &[Message]) -> Option<String> {
    messages.iter().rev().find_map(|msg| match msg {
        Message::Assistant(assistant) => text_from_blocks(&assistant.content),
        _ => None,
    })
}

fn text_from_blocks(blocks: &[ContentBlock]) -> Option<String> {
    let mut text = String::new();
    for block in blocks {
        if let ContentBlock::Text(part) = block {
            text.push_str(&part.text);
        }
    }
    (!text.is_empty()).then_some(text)
}

fn aggregate_usage(messages: &[Message]) -> Option<Value> {
    let mut input = 0_u64;
    let mut output = 0_u64;
    let mut cache_read = 0_u64;
    let mut cache_write = 0_u64;
    let mut cost_in = 0.0_f64;
    let mut cost_out = 0.0_f64;
    let mut cost_read = 0.0_f64;
    let mut cost_write = 0.0_f64;
    let mut cost_total = 0.0_f64;
    let mut seen = false;
    let mut saw_cost = false;
    for msg in messages {
        let Message::Assistant(assistant) = msg else {
            continue;
        };
        seen = true;
        input = input.saturating_add(assistant.usage.input);
        output = output.saturating_add(assistant.usage.output);
        cache_read = cache_read.saturating_add(assistant.usage.cache_read);
        cache_write = cache_write.saturating_add(assistant.usage.cache_write);
        let cost = &assistant.usage.cost;
        if cost.total > 0.0
            || cost.input > 0.0
            || cost.output > 0.0
            || cost.cache_read > 0.0
            || cost.cache_write > 0.0
        {
            saw_cost = true;
            cost_in += cost.input;
            cost_out += cost.output;
            cost_read += cost.cache_read;
            cost_write += cost.cache_write;
            cost_total += cost.total;
        }
    }
    if !seen {
        return None;
    }
    let mut usage = serde_json::Map::from_iter([
        ("inputTokens".into(), json!(input)),
        ("outputTokens".into(), json!(output)),
        ("cacheReadTokens".into(), json!(cache_read)),
        ("cacheWriteTokens".into(), json!(cache_write)),
    ]);
    if saw_cost {
        usage.insert(
            "costUsd".into(),
            json!({
                "input": cost_in,
                "output": cost_out,
                "cacheRead": cost_read,
                "cacheWrite": cost_write,
                "total": cost_total,
            }),
        );
    }
    Some(Value::Object(usage))
}
