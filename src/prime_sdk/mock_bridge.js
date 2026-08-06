#!/usr/bin/env node
/**
 * Offline mock for prime-sdk-bridge protocol (unit tests).
 * Speaks create / send / cancel / close (no resume).
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

async function handleCreate(req) {
  if (created) {
    emit({ event: "fatal", message: "agent already created", retryable: false });
    return;
  }
  if (req.noForcePolicy === "fail_fast") {
    emit({
      event: "fatal",
      message: "--no-force is not supported with the Prime SDK backend",
      retryable: false,
    });
    return;
  }
  created = true;
  emit({ event: "ok", agentId: "mock-prime-session" });
}

async function handleSend(req) {
  if (!created) {
    emit({ event: "fatal", message: "create before send", retryable: false });
    return;
  }
  const prompt = String(req.prompt ?? "");
  emit({ event: "thinking", text: "mock-think" });
  emit({ event: "assistant", text: `echo:${prompt}` });
  emit({
    event: "tool_call",
    phase: "start",
    name: "ipython",
    summary: "ipython",
    toolCallId: "t1",
  });
  emit({
    event: "tool_call",
    phase: "end",
    name: "ipython",
    summary: "ipython",
    toolCallId: "t1",
  });
  emit({
    event: "usage",
    usage: { inputTokens: 3, outputTokens: 2 },
  });
  emit({
    event: "run_done",
    status: "ok",
    result: `echo:${prompt}`,
    usage: { inputTokens: 3, outputTokens: 2 },
  });
}

async function handleCancel() {
  if (sendInFlight) {
    // best-effort
  }
  emit({ event: "run_done", status: "cancelled", error: "cancelled" });
}

async function handleClose() {
  closing = true;
  process.exit(0);
}

async function dispatch(req) {
  switch (req.op) {
    case "create":
      await handleCreate(req);
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
  const rl = readline.createInterface({ input: process.stdin, crlfDelay: Infinity });
  const queue = [];
  let pumping = false;

  async function pump() {
    if (pumping) return;
    pumping = true;
    while (queue.length > 0 && !closing) {
      const line = queue.shift();
      let req;
      try {
        req = JSON.parse(line);
      } catch (e) {
        emit({ event: "fatal", message: `bad json: ${e}`, retryable: false });
        continue;
      }
      await dispatch(req);
    }
    pumping = false;
  }

  rl.on("line", (line) => {
    queue.push(line);
    void pump();
  });
}

main();
