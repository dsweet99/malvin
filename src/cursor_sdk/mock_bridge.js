#!/usr/bin/env node

import * as fs from "node:fs";
import * as path from "node:path";
import * as readline from "node:readline";

let created = false;
let closing = false;
let sendInFlight = null;

function emit(obj) {
  process.stdout.write(`${JSON.stringify(obj)}\n`);
}

function sleep(ms) {
  return new Promise((r) => setTimeout(r, ms));
}

function noteBoot(kind) {
  const dir = process.env.MOCK_BRIDGE_ONCE_DIR;
  if (!dir) {
    return;
  }
  try {
    fs.appendFileSync(path.join(dir, "boots"), `${kind}\n`);
  } catch {
    // ignore logging failures in mock
  }
}

function consumeOnceFlag(name) {
  const dir = process.env.MOCK_BRIDGE_ONCE_DIR;
  if (!dir) {
    return false;
  }
  const flag = path.join(dir, name);
  if (fs.existsSync(flag)) {
    return false;
  }
  try {
    fs.writeFileSync(flag, "1");
    return true;
  } catch {
    return false;
  }
}

let agentId = "mock-agent";

let bootKind = null;

async function handleCreate(req) {
  if (created) {
    emit({
      event: "fatal",
      message: "agent already created",
      retryable: false,
    });
    return;
  }
  if (process.env.MOCK_BRIDGE_HANG_CREATE === "1") {
    await sleep(60_000);
    return;
  }
  if (req.noForcePolicy === "fail_fast") {
    emit({
      event: "fatal",
      message: "--no-force is not supported with the Cursor SDK backend",
      retryable: false,
    });
    return;
  }
  created = true;
  bootKind = "create";
  agentId = "mock-agent";
  noteBoot("create");
  emit({ event: "ok", agentId });
}

async function handleResume(req) {
  if (created) {
    emit({
      event: "fatal",
      message: "agent already created",
      retryable: false,
    });
    return;
  }
  if (req.noForcePolicy === "fail_fast") {
    emit({
      event: "fatal",
      message: "--no-force is not supported with the Cursor SDK backend",
      retryable: false,
    });
    return;
  }
  if (!req.agentId) {
    emit({ event: "fatal", message: "resume requires agentId", retryable: false });
    return;
  }
  created = true;
  bootKind = "resume";
  agentId = String(req.agentId);
  noteBoot("resume");
  emit({ event: "ok", agentId });
}

async function handleSend(req) {
  if (!created) {
    emit({ event: "fatal", message: "create before send", retryable: false });
    return;
  }
  const prompt = String(req.prompt || "");

  if (prompt.includes("NON_TEARDOWN_TIMEOUT_ONCE") && consumeOnceFlag("non_teardown_timeout_once")) {
    emit({
      event: "fatal",
      message: "request timed out",
      retryable: true,
    });
    return;
  }

  if (prompt.includes("CLOSE_STDOUT")) {
    emit({ event: "assistant", text: "closing" });
    
    process.exit(0);
  }

  if (prompt.includes("STREAM_FATAL_ONLY")) {
    emit({
      event: "fatal",
      message: "stream error: injected-only",
      retryable: true,
    });
    return;
  }

  
  if (prompt.includes("AUTH_ONCE") && bootKind === "create") {
    emit({
      event: "fatal",
      message: "Authentication",
      retryable: false,
    });
    return;
  }

  
  
  if (prompt.includes("AGENT_BUSY_ON_RESUME") && bootKind === "resume") {
    emit({
      event: "fatal",
      message: `Agent ${agentId} already has active run`,
      retryable: false,
    });
    return;
  }

  if (prompt.includes("FATAL_THEN_RUN_DONE")) {
    emit({
      event: "fatal",
      message: "stream error: paired",
      retryable: false,
    });
    emit({
      event: "run_done",
      status: "finished",
      result: "STALE_SUCCESS",
      durationMs: 1,
    });
    return;
  }

  if (prompt.includes("CANCELLED_RUN")) {
    emit({ event: "assistant", text: "partial" });
    emit({
      event: "run_done",
      status: "cancelled",
      result: "partial reply",
      usage: {
        inputTokens: 3,
        outputTokens: 1,
        cacheReadTokens: 0,
        cacheWriteTokens: 0,
      },
      durationMs: 1,
    });
    return;
  }

  if (prompt.includes("SLOW_SEND")) {
    await sleep(400);
    if (closing) {
      emit({
        event: "run_done",
        status: "cancelled",
        result: "slow cancelled",
        durationMs: 400,
      });
      return;
    }
  }

  
  if (prompt.includes("NEVER_RUN_DONE")) {
    emit({ event: "assistant", text: "partial before hang" });
    await sleep(60_000);
    return;
  }

  
  if (prompt.includes("EMPTY_RESULT_RUN_DONE")) {
    emit({ event: "assistant", text: "" });
    emit({
      event: "run_done",
      status: "finished",
      durationMs: 1,
    });
    return;
  }

  
  if (prompt.includes("KEEP_ALIVE_THEN_DONE")) {
    for (let i = 0; i < 6; i++) {
      emit({ event: "assistant", text: `tick-${i}` });
      await sleep(80);
    }
    emit({
      event: "run_done",
      status: "finished",
      result: "kept-alive",
      usage: {
        inputTokens: 1,
        outputTokens: 1,
        cacheReadTokens: 0,
        cacheWriteTokens: 0,
      },
      durationMs: 500,
    });
    return;
  }

  if (prompt.includes("LONG_TOOL_TURN_THEN_DONE")) {
    const periodMs = Math.max(
      1,
      Number.parseInt(process.env.MOCK_BRIDGE_TOOL_PERIOD_MS || "60", 10) || 60,
    );
    const pulses = Math.max(
      4,
      Number.parseInt(process.env.MOCK_BRIDGE_TOOL_PULSES || "8", 10) || 8,
    );
    const toolId = "mock-long-shell";
    emit({
      event: "tool_call",
      phase: "start",
      name: "shell",
      summary: "Run long compile",
      toolCallId: toolId,
    });
    for (let i = 0; i < pulses; i++) {
      await sleep(periodMs);
      emit({ event: "progress", kind: "heartbeat", detail: `tool-pulse-${i}` });
    }
    emit({
      event: "tool_call",
      phase: "complete",
      name: "shell",
      summary: "Run long compile",
      toolCallId: toolId,
    });
    emit({
      event: "run_done",
      status: "finished",
      result: "long-tool-turn-done",
      usage: {
        inputTokens: 1,
        outputTokens: 1,
        cacheReadTokens: 0,
        cacheWriteTokens: 0,
      },
      durationMs: periodMs * pulses,
    });
    return;
  }

  if (prompt.includes("PROGRESS_THEN_DONE")) {
    const periodMs = Math.max(
      1,
      Number.parseInt(process.env.MOCK_BRIDGE_PROGRESS_MS || "40", 10) || 40,
    );
    const pulses = Math.max(
      1,
      Number.parseInt(process.env.MOCK_BRIDGE_PROGRESS_COUNT || "6", 10) || 6,
    );
    for (let i = 0; i < pulses; i++) {
      emit({ event: "progress", kind: "heartbeat", detail: `pulse-${i}` });
      await sleep(periodMs);
    }
    emit({
      event: "run_done",
      status: "finished",
      result: "progressed",
      usage: {
        inputTokens: 1,
        outputTokens: 1,
        cacheReadTokens: 0,
        cacheWriteTokens: 0,
      },
      durationMs: periodMs * pulses,
    });
    return;
  }

  const fenced = prompt.includes("NEED_DM");
  const result = fenced
    ? "__MALVIN_DM_START__\nHello.\n__MALVIN_DM_END__"
    : "mock reply";
  emit({ event: "assistant", text: fenced ? "Hello" : "mock reply" });
  emit({ event: "step", kind: "onStep" });
  emit({
    event: "run_done",
    status: "finished",
    result,
    usage: {
      inputTokens: 11,
      outputTokens: 7,
      cacheReadTokens: 0,
      cacheWriteTokens: 0,
    },
    durationMs: 1,
  });
}

async function handleCancel() {
  closing = true;
}

async function handleClose() {
  closing = true;
  if (sendInFlight) {
    try {
      await sendInFlight;
    } catch {
      
    }
  }
  process.exit(0);
}

async function dispatch(req) {
  switch (req.op) {
    case "create":
      await handleCreate(req);
      break;
    case "resume":
      await handleResume(req);
      break;
    case "send":
      sendInFlight = handleSend(req);
      await sendInFlight;
      sendInFlight = null;
      break;
    case "cancel":
      await handleCancel();
      break;
    case "close":
      await handleClose();
      break;
    default:
      emit({ event: "fatal", message: `unknown op ${req.op}`, retryable: false });
  }
}

async function main() {
  const rl = readline.createInterface({
    input: process.stdin,
    crlfDelay: Infinity,
  });
  const queue = [];
  let wake = null;
  let stdinClosed = false;

  rl.on("line", (line) => {
    queue.push(line);
    wake?.();
  });
  rl.on("close", () => {
    stdinClosed = true;
    wake?.();
  });

  async function nextLine() {
    for (;;) {
      if (queue.length > 0) return queue.shift();
      if (stdinClosed) return null;
      await new Promise((r) => {
        wake = r;
      });
      wake = null;
    }
  }

  let busy = Promise.resolve();

  for (;;) {
    const line = await nextLine();
    if (line === null) break;
    const trimmed = line.trim();
    if (!trimmed) continue;
    let req;
    try {
      req = JSON.parse(trimmed);
    } catch (e) {
      emit({ event: "fatal", message: String(e), retryable: false });
      continue;
    }

    
    if (req.op === "cancel" || req.op === "close") {
      try {
        await dispatch(req);
      } catch (err) {
        emit({
          event: "fatal",
          message: `bad request: ${err}`,
          retryable: false,
        });
      }
      if (req.op === "close") break;
      continue;
    }

    busy = busy.then(() => dispatch(req)).catch((err) => {
      emit({
        event: "fatal",
        message: `bad request: ${err}`,
        retryable: false,
      });
    });
    
  }

  await busy;
}

main().catch((err) => {
  emit({ event: "fatal", message: String(err), retryable: false });
  process.exit(1);
});
