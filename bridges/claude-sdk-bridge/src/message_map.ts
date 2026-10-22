import type { SDKMessage } from "@anthropic-ai/claude-agent-sdk";
import { type BridgeEvent } from "./protocol.js";
import { claudeUsageRecord } from "./usage.js";

export function mapMessage(message: SDKMessage): BridgeEvent[] {
  const events: BridgeEvent[] = [];
  mapAssistant(message, events);
  mapToolResults(message, events);
  if (message.type === "system" && message.subtype === "init") {
    events.push({ event: "step", kind: "initialized" });
    return events;
  }
  if (message.type !== "result") return events;
  const status = message.subtype === "success" && !message.is_error ? "finished" : "error";
  const usage = claudeUsageRecord(message.usage as unknown as Record<string, unknown>);
  const result = message.subtype === "success" ? message.result : message.errors.join("\n");
  events.push({ event: "usage", usage });
  events.push({
    event: "run_done",
    status,
    result,
    usage,
    error: message.is_error ? result : undefined,
    durationMs: message.duration_ms,
  });
  return events;
}

function mapAssistant(message: SDKMessage, events: BridgeEvent[]): void {
  if (message.type !== "assistant") return;
  for (const block of message.message.content) {
    if (block.type === "text" && block.text) events.push({ event: "assistant", text: block.text });
    if (block.type === "thinking" && block.thinking) events.push({ event: "thinking", text: block.thinking });
    if (block.type === "tool_use") {
      events.push({ event: "tool_call", phase: "start", name: block.name, toolCallId: block.id, summary: block.name });
    }
  }
}

function mapToolResults(message: SDKMessage, events: BridgeEvent[]): void {
  if (message.type !== "user" || !Array.isArray(message.message.content)) return;
  for (const block of message.message.content) {
    if (block.type !== "tool_result") continue;
    events.push({
      event: "tool_call",
      phase: block.is_error ? "error" : "complete",
      toolCallId: block.tool_use_id,
    });
  }
}
