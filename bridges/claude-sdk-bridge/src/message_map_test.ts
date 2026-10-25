import assert from "node:assert/strict";
import test from "node:test";
import type { SDKMessage } from "@anthropic-ai/claude-agent-sdk";
import { mapMessage } from "./message_map.js";

test("maps tool results to terminal lifecycle events", () => {
  const complete = mapMessage({
    type: "user",
    message: {
      role: "user",
      content: [{ type: "tool_result", tool_use_id: "tool-1", content: "done" }],
    },
    parent_tool_use_id: null,
  } as SDKMessage);
  const error = mapMessage({
    type: "user",
    message: {
      role: "user",
      content: [{ type: "tool_result", tool_use_id: "tool-2", content: "failed", is_error: true }],
    },
    parent_tool_use_id: null,
  } as SDKMessage);

  assert.deepEqual(complete, [{ event: "tool_call", phase: "complete", toolCallId: "tool-1" }]);
  assert.deepEqual(error, [{ event: "tool_call", phase: "error", toolCallId: "tool-2" }]);
});

test("tool_use start and result carry name and summary", () => {
  const start = mapMessage({
    type: "assistant",
    message: {
      role: "assistant",
      content: [{ type: "tool_use", id: "t-9", name: "Bash", input: { command: "ls  -l" } }],
    },
    parent_tool_use_id: null,
  } as unknown as SDKMessage);
  assert.deepEqual(start, [
    { event: "tool_call", phase: "start", name: "Bash", toolCallId: "t-9", summary: "Run ls -l" },
  ]);
  const done = mapMessage({
    type: "user",
    message: { role: "user", content: [{ type: "tool_result", tool_use_id: "t-9", content: "x" }] },
    parent_tool_use_id: null,
  } as SDKMessage);
  assert.deepEqual(done, [
    { event: "tool_call", phase: "complete", toolCallId: "t-9", name: "Bash", summary: "Run ls -l" },
  ]);
});
