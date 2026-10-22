import assert from "node:assert/strict";
import test from "node:test";
import { claudeUsageRecord } from "./usage.js";

test("maps Claude Agent SDK usage to malvin bridge usage", () => {
  assert.deepEqual(
    claudeUsageRecord({
      input_tokens: 12,
      output_tokens: 5,
      cache_read_input_tokens: 3,
      cache_creation_input_tokens: 2,
      thinking_tokens: 4,
    }),
    {
      inputTokens: 12,
      outputTokens: 5,
      cacheReadTokens: 3,
      cacheWriteTokens: 2,
      reasoningTokens: 4,
    },
  );
});
