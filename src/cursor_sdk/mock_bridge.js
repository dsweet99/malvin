#!/usr/bin/env node
/**
 * Offline mock for cursor-sdk-bridge protocol (unit tests).
 * Speaks the same JSONL ops/events as the real bridge.
 *
 * Concurrent cancel: stdin lines are queued so `cancel`/`close` can run while
 * a slow `send` is in flight (mirrors production bridge main loop).
 */
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

let agentId = "mock-agent";
/** "create" | "resume" — AUTH_ONCE fails only on create-booted agents. */
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
  emit({ event: "ok", agentId });
}

async function handleSend(req) {
  if (!created) {
    emit({ event: "fatal", message: "create before send", retryable: false });
    return;
  }
  const prompt = String(req.prompt || "");

  if (prompt.includes("CLOSE_STDOUT")) {
    emit({ event: "assistant", text: "closing" });
    // Exit so the pipe delivers EOF to Rust (`bridge stdout closed`).
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

  // Idle-auth misclassification: fails on create-booted agents; resume succeeds.
  if (prompt.includes("AUTH_ONCE") && bootKind === "create") {
    emit({
      event: "fatal",
      message: "Authentication",
      retryable: false,
    });
    return;
  }

  // Orphaned Cursor run after hard-kill/resume: AgentBusy on resume-booted agents only.
  // Create-booted sends must succeed so retry-after-forget can recover.
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

  const fenced = prompt.includes("NEED_DM");
  const result = fenced
    ? "MALVIN_DM_START\nHello.\nMALVIN_DM_END"
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
      // best-effort
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

    // Cancel/close must not wait for an in-flight send.
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
    // Do not await `busy` here: that would block reading cancel lines.
  }

  await busy;
}

main().catch((err) => {
  emit({ event: "fatal", message: String(err), retryable: false });
  process.exit(1);
});
