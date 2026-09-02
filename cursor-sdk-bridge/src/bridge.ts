

import "./parent_death.js";
import * as readline from "node:readline";
import { Agent, Cursor, CursorAgentError, configureCursorSdk } from "@cursor/sdk";
import type { AgentOptions, Run, SDKAgent } from "@cursor/sdk";
import { modelSelectionFromRaw } from "./model_selection.js";
import {
  canonicalRunDoneStatus,
  emit,
  parseRequest,
  type BridgeRequest,
  type CreateOp,
  type ResumeOp,
  type SendOp,
} from "./protocol.js";
import {
  eventsAfterStreamFailure,
  exitCodeForSignal,
  isInterruptOp,
  runAcceptsProgressHeartbeat,
  isStaleAuthMisclassification,
  isStaleAuthText,
} from "./bridge_policy.js";
import { forwardSdkMessage, usageRecord } from "./sdk_map.js";

export {
  eventsAfterStreamFailure,
  exitCodeForSignal,
  isInterruptOp,
  isStaleAuthMisclassification,
  isStaleAuthText,
} from "./bridge_policy.js";
export { installParentDeathWatch } from "./parent_death.js";

let agent: SDKAgent | null = null;

let agentId: string | null = null;

let agentOptions: AgentOptions | null = null;
let currentRun: Run | null = null;
let closing = false;

function quietExit(code: number): void {
  closing = true;
  
  process.exit(code);
}

function installQuietSignalHandlers(): void {
  for (const signal of ["SIGINT", "SIGTERM"] as const) {
    process.on(signal, () => {
      quietExit(exitCodeForSignal(signal));
    });
  }
  process.on("uncaughtException", (err) => {
    if (closing) return;
    emitFatal(err);
    quietExit(1);
  });
  process.on("unhandledRejection", (reason) => {
    if (closing) return;
    emitFatal(reason);
    
    
  });
}

function apiKeyFrom(req: { apiKey?: string }): string | undefined {
  const k = (req.apiKey ?? process.env.CURSOR_API_KEY ?? "").trim();
  return k.length > 0 ? k : undefined;
}

function errorMessage(err: unknown): string {
  if (err instanceof Error) return err.message;
  return String(err);
}

function isRetryable(err: unknown): boolean {
  return err instanceof CursorAgentError ? Boolean(err.isRetryable) : false;
}

function emitFatal(err: unknown): void {
  emit({
    event: "fatal",
    message: errorMessage(err),
    retryable: isRetryable(err),
  });
}

type AgentBootOp = CreateOp | ResumeOp;

function preferHttp1ForAgent(): boolean {
  
  
  
  const flag = (process.env.MALVIN_CURSOR_USE_HTTP1 || "").trim().toLowerCase();
  if (flag === "1" || flag === "true" || flag === "yes") return true;
  if (flag === "0" || flag === "false" || flag === "no") return false;
  return Boolean(
    (process.env.HTTPS_PROXY || "").trim() ||
      (process.env.HTTP_PROXY || "").trim(),
  );
}

function ensureHttp1AgentTransport(): void {
  if (!preferHttp1ForAgent()) return;
  configureCursorSdk({ local: { useHttp1ForAgent: true } });
}

function agentOptionsFromBoot(req: AgentBootOp, apiKey: string): AgentOptions {
  const local: NonNullable<AgentOptions["local"]> = {
    cwd: req.cwd,
    settingSources: [],
  };
  if (req.sandboxEnabled || req.noForcePolicy === "sandbox") {
    local.sandboxOptions = { enabled: true };
  }
  if (req.autoReview || req.noForcePolicy === "auto_review") {
    local.autoReview = true;
  }
  return {
    apiKey,
    model: modelSelectionFromRaw(req.model || "auto"),
    local,
  };
}

function rejectNoForce(req: AgentBootOp): boolean {
  if (req.noForcePolicy !== "fail_fast") return false;
  emit({
    event: "fatal",
    message:
      "--no-force is not supported with the Cursor SDK backend (no interactive tool approval). Omit --no-force.",
    retryable: false,
  });
  return true;
}

function requireApiKey(req: AgentBootOp): string | undefined {
  const apiKey = apiKeyFrom(req);
  if (apiKey) return apiKey;
  emit({
    event: "fatal",
    message:
      "CURSOR_API_KEY is required for the Cursor SDK backend (also accepts CURSOR_AGENT_API_KEY / AGENT_API_KEY via malvin).",
    retryable: false,
  });
  return undefined;
}

async function handleCreate(req: CreateOp): Promise<void> {
  if (agent) {
    emit({ event: "fatal", message: "agent already created", retryable: false });
    return;
  }
  if (rejectNoForce(req)) return;
  const apiKey = requireApiKey(req);
  if (!apiKey) return;
  try {
    ensureHttp1AgentTransport();
    const opts = agentOptionsFromBoot(req, apiKey);
    agent = await Agent.create(opts);
    agentOptions = opts;
    agentId = agent.agentId;
    emit({ event: "ok", agentId });
  } catch (err) {
    emitFatal(err);
  }
}

async function handleResume(req: ResumeOp): Promise<void> {
  if (agent) {
    emit({ event: "fatal", message: "agent already created", retryable: false });
    return;
  }
  if (rejectNoForce(req)) return;
  const apiKey = requireApiKey(req);
  if (!apiKey) return;
  if (!req.agentId?.trim()) {
    emit({
      event: "fatal",
      message: "resume requires agentId",
      retryable: false,
    });
    return;
  }
  try {
    ensureHttp1AgentTransport();
    const opts = agentOptionsFromBoot(req, apiKey);
    agent = await Agent.resume(req.agentId, opts);
    agentOptions = opts;
    agentId = agent.agentId || req.agentId;
    emit({ event: "ok", agentId });
  } catch (err) {
    emitFatal(err);
  }
}

async function recoverStaleAuth(): Promise<boolean> {
  if (!agentId || !agentOptions) return false;
  const id = agentId;
  const opts = agentOptions;
  agent = null;
  try {
    agent = await Agent.resume(id, opts);
    agentId = agent.agentId || id;
    return true;
  } catch {
    agent = null;
    return false;
  }
}

async function handleSend(req: SendOp, alreadyRecovered = false): Promise<void> {
  if (!agent) {
    emit({ event: "fatal", message: "create before send", retryable: false });
    return;
  }
  let started = Date.now();
  let lastForwardedAt = Date.now();
  let runPending = { value: false };
  let noteForwarded = (): void => {
    lastForwardedAt = Date.now();
  };
  const progressTimer = setInterval(() => {
    if (
      !runAcceptsProgressHeartbeat(
        Boolean(currentRun),
        runPending.value,
        closing,
        lastForwardedAt,
        Date.now(),
      )
    ) {
      return;
    }
    emit({ event: "progress", kind: "heartbeat" });
    lastForwardedAt = Date.now();
  }, 1000);
  try {
    runPending.value = true;
    currentRun = await agent.send(req.prompt, {
      onStep: () => {
        noteForwarded();
        emit({ event: "step", kind: "onStep" });
      },
      local: req.forceStuck ? { force: true } : undefined,
    });
    runPending.value = false;
    try {
      for await (const msg of currentRun.stream()) {
        if (forwardSdkMessage(msg)) noteForwarded();
      }
    } catch (streamErr) {
      if (
        !alreadyRecovered &&
        isStaleAuthMisclassification(streamErr) &&
        (await recoverStaleAuth())
      ) {
        currentRun = null;
        return handleSend(req, true);
      }
      for (const ev of eventsAfterStreamFailure(streamErr)) {
        emit(ev);
      }
      try {
        await currentRun.wait();
      } catch {
        
      }
      currentRun = null;
      return;
    }
    const result = await currentRun.wait();
    currentRun = null;
    const errObj = result.error;
    const errMsg = errObj?.message ?? "";
    if (
      (result.status === "error" || result.status === "cancelled") &&
      errMsg &&
      !alreadyRecovered &&
      isStaleAuthText("", errMsg) &&
      (await recoverStaleAuth())
    ) {
      return handleSend(req, true);
    }
    emit({
      event: "run_done",
      status: canonicalRunDoneStatus(String(result.status ?? "")),
      result: result.result,
      usage: usageRecord(result.usage as Record<string, number> | undefined),
      error: errObj?.message,
      durationMs: result.durationMs ?? Date.now() - started,
    });
  } catch (err) {
    currentRun = null;
    if (
      !alreadyRecovered &&
      isStaleAuthMisclassification(err) &&
      (await recoverStaleAuth())
    ) {
      return handleSend(req, true);
    }
    emitFatal(err);
  } finally {
    runPending.value = false;
    clearInterval(progressTimer);
  }
}

async function handleCancel(): Promise<void> {
  const run = currentRun;
  if (!run) return;
  try {
    if (run.supports("cancel")) {
      await run.cancel();
    }
  } catch {
    
  }
}

async function handleClose(): Promise<void> {
  closing = true;
  await handleCancel();
  agent = null;
  agentId = null;
  agentOptions = null;
  
  
  quietExit(0);
}

async function handleListModels(apiKey?: string): Promise<void> {
  const key = apiKeyFrom({ apiKey });
  if (!key) {
    emit({
      event: "fatal",
      message: "CURSOR_API_KEY required to list models",
      retryable: false,
    });
    return;
  }
  try {
    const models = await Cursor.models.list({ apiKey: key });
    emit({
      event: "models",
      models: models.map((m) => ({
        id: m.id,
        displayName: m.displayName,
        parameters: m.parameters?.map((p) => ({
          id: p.id,
          values: p.values.map((v) => v.value),
        })),
      })),
    });
  } catch (err) {
    emitFatal(err);
  }
}

async function dispatch(req: BridgeRequest): Promise<void> {
  switch (req.op) {
    case "create":
      await handleCreate(req);
      break;
    case "resume":
      await handleResume(req);
      break;
    case "send":
      await handleSend(req);
      break;
    case "cancel":
      await handleCancel();
      break;
    case "close":
      await handleClose();
      break;
    case "list_models":
      await handleListModels(req.apiKey);
      break;
    default:
      emit({
        event: "fatal",
        message: `unknown op: ${(req as { op?: string }).op}`,
        retryable: false,
      });
  }
}

async function main(): Promise<void> {
  installQuietSignalHandlers();
  const rl = readline.createInterface({
    input: process.stdin,
    crlfDelay: Infinity,
  });
  const queue: string[] = [];
  let wake: (() => void) | null = null;
  let stdinClosed = false;

  rl.on("line", (line) => {
    queue.push(line);
    wake?.();
  });
  rl.on("close", () => {
    
    
    stdinClosed = true;
    wake?.();
    if (!closing) {
      closing = true;
      void handleCancel().finally(() => quietExit(0));
    }
  });

  async function nextLine(): Promise<string | null> {
    for (;;) {
      if (queue.length > 0) {
        return queue.shift() ?? null;
      }
      if (stdinClosed) {
        return null;
      }
      await new Promise<void>((resolve) => {
        wake = resolve;
      });
      wake = null;
    }
  }

  let serial: Promise<void> = Promise.resolve();

  for (;;) {
    if (closing) break;
    const line = await nextLine();
    if (line === null) break;
    const trimmed = line.trim();
    if (!trimmed) continue;
    let req: BridgeRequest;
    try {
      req = parseRequest(trimmed);
    } catch (err) {
      emit({
        event: "fatal",
        message: `bad request: ${errorMessage(err)}`,
        retryable: false,
      });
      continue;
    }

    if (isInterruptOp(req.op)) {
      try {
        await dispatch(req);
      } catch (err) {
        emit({
          event: "fatal",
          message: `bad request: ${errorMessage(err)}`,
          retryable: false,
        });
      }
      if (req.op === "close") break;
      continue;
    }

    
    serial = serial
      .then(() => dispatch(req))
      .catch((err: unknown) => {
        emit({
          event: "fatal",
          message: `bad request: ${errorMessage(err)}`,
          retryable: false,
        });
      });
  }

  await serial.catch(() => undefined);
}

main().catch((err) => {
  emitFatal(err);
  process.exit(1);
});
