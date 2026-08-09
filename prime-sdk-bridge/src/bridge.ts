/**
 * Long-lived malvin ↔ prime-agent JSONL bridge.
 * One createAgentSession per process; many session.prompt calls.
 * No resume in v1 — if the process dies, Rust creates a fresh session.
 */

// Arm parent-death watch before heavy imports (see parent_death.ts).
import "./parent_death.js";
import * as readline from "node:readline";
import { getModel } from "@earendil-works/pi-ai";
import {
  AuthStorage,
  createAgentSession,
  ModelRegistry,
  SessionManager,
  type AgentSession,
} from "prime-agent";
import { emit, exitCodeForSignal, parseRequest, type BridgeRequest } from "./protocol.js";
import {
  enrichCompleteSummary,
  toolPhaseFromEnd,
  toolSummaryFromArgs,
} from "./tool_summary.js";

export { installParentDeathWatch } from "./parent_death.js";

let session: AgentSession | null = null;
let unsubscribe: (() => void) | null = null;
let promptInFlight: Promise<void> | null = null;
let closing = false;
let lastUsage: Record<string, unknown> | null = null;
let lastAssistant = "";
/** Start-phase subject by toolCallId — complete/error must keep enriched Run/… lines. */
const toolStartSummaries = new Map<string, string>();

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
}

function errorMessage(err: unknown): string {
  if (err instanceof Error) return err.message;
  return String(err);
}

function emitFatal(err: unknown): void {
  emit({
    event: "fatal",
    message: errorMessage(err),
    retryable: false,
  });
}

function splitModel(slug: string): { provider: string; modelId: string } {
  const idx = slug.indexOf("/");
  if (idx <= 0 || idx === slug.length - 1) {
    throw new Error(`prime model must be provider/model (got ${slug})`);
  }
  return { provider: slug.slice(0, idx), modelId: slug.slice(idx + 1) };
}

function resolveModel(slug: string, registry: ModelRegistry) {
  const { provider, modelId } = splitModel(slug);
  const fromRegistry = registry.find(provider, modelId);
  if (fromRegistry) return fromRegistry;
  const builtIn = getModel(provider as never, modelId as never);
  if (builtIn) return builtIn;
  throw new Error(`unknown prime model ${provider}/${modelId}`);
}

function attachSubscriber(s: AgentSession): void {
  unsubscribe?.();
  lastUsage = null;
  lastAssistant = "";
  unsubscribe = s.subscribe((event) => {
    switch (event.type) {
      case "turn_start":
        // Cursor bridge emits `step` via onStep; turn_start is the closest Prime analog.
        emit({ event: "step", kind: "turn_start" });
        break;
      case "message_update": {
        const ame = event.assistantMessageEvent;
        if (ame.type === "text_delta") {
          lastAssistant += ame.delta;
          emit({ event: "assistant", text: ame.delta });
        } else if (ame.type === "thinking_delta") {
          emit({ event: "thinking", text: ame.delta });
        }
        break;
      }
      case "tool_execution_start": {
        const toolCallId = String(event.toolCallId ?? event.toolName);
        // Enriched subject so Rust log adapter can tee VISION `t|` lines (cursor parity).
        const summary = toolSummaryFromArgs(event.toolName, event.args);
        toolStartSummaries.set(toolCallId, summary);
        emit({
          event: "tool_call",
          phase: "start",
          name: event.toolName,
          summary,
          toolCallId,
        });
        break;
      }
      case "tool_execution_end": {
        const toolCallId = String(event.toolCallId ?? event.toolName);
        const isError = Boolean(event.isError);
        const base =
          toolStartSummaries.get(toolCallId) ??
          toolSummaryFromArgs(event.toolName, undefined);
        toolStartSummaries.delete(toolCallId);
        emit({
          event: "tool_call",
          // Cursor protocol uses complete/error — not "end" (Rust only tees those phases).
          phase: toolPhaseFromEnd(isError),
          name: event.toolName,
          summary: enrichCompleteSummary(base, event.toolName, event.result, isError),
          toolCallId,
        });
        break;
      }
      case "agent_end": {
        // Best-effort usage from last assistant message when present.
        try {
          const usage = (event as { messages?: Array<{ usage?: Record<string, unknown> }> })
            .messages?.slice(-1)[0]?.usage;
          if (usage) {
            lastUsage = usage;
            emit({ event: "usage", usage });
          }
        } catch {
          // ignore
        }
        break;
      }
      default:
        break;
    }
  });
}

async function handleCreate(req: Extract<BridgeRequest, { op: "create" }>): Promise<void> {
  if (session) {
    emit({ event: "fatal", message: "agent already created", retryable: false });
    return;
  }
  if (req.noForcePolicy === "fail_fast") {
    emit({
      event: "fatal",
      message:
        "--no-force is not supported with the Prime SDK backend (no interactive tool approval). Omit --no-force.",
      retryable: false,
    });
    return;
  }
  const authStorage = AuthStorage.create();
  const modelRegistry = req.modelsJsonPath
    ? ModelRegistry.create(authStorage, req.modelsJsonPath)
    : ModelRegistry.create(authStorage);
  const model = resolveModel(req.model || "openai/gpt-4o", modelRegistry);
  const cwd = req.cwd || process.cwd();
  const created = await createAgentSession({
    model,
    cwd,
    tools: ["ipython"],
    sessionManager: SessionManager.inMemory(cwd),
    authStorage,
    modelRegistry,
  });
  session = created.session;
  toolStartSummaries.clear();
  attachSubscriber(session);
  emit({ event: "ok", agentId: session.sessionId });
}

async function handleSend(req: Extract<BridgeRequest, { op: "send" }>): Promise<void> {
  if (!session) {
    emit({ event: "fatal", message: "create before send", retryable: false });
    return;
  }
  lastAssistant = "";
  lastUsage = null;
  const started = Date.now();
  try {
    promptInFlight = session.prompt(req.prompt);
    await promptInFlight;
    promptInFlight = null;
    emit({
      event: "run_done",
      status: "ok",
      result: lastAssistant || undefined,
      usage: lastUsage || undefined,
      durationMs: Date.now() - started,
    });
  } catch (err) {
    promptInFlight = null;
    emit({
      event: "run_done",
      status: "error",
      error: errorMessage(err),
      result: lastAssistant || undefined,
      usage: lastUsage || undefined,
      durationMs: Date.now() - started,
    });
  }
}

async function handleCancel(): Promise<void> {
  if (session) {
    try {
      await session.abort();
    } catch {
      // ignore
    }
  }
}

async function handleClose(): Promise<void> {
  closing = true;
  unsubscribe?.();
  unsubscribe = null;
  if (session) {
    try {
      session.dispose();
    } catch {
      // ignore
    }
    session = null;
  }
  quietExit(0);
}

async function dispatch(req: BridgeRequest): Promise<void> {
  switch (req.op) {
    case "create":
      await handleCreate(req);
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
    default:
      emit({ event: "fatal", message: `unknown op`, retryable: false });
  }
}

async function main(): Promise<void> {
  installQuietSignalHandlers();
  const rl = readline.createInterface({ input: process.stdin, crlfDelay: Infinity });
  const queue: string[] = [];
  let pumping = false;

  async function pump(): Promise<void> {
    if (pumping) return;
    pumping = true;
    while (queue.length > 0 && !closing) {
      const line = queue.shift()!;
      try {
        const req = parseRequest(line);
        await dispatch(req);
      } catch (err) {
        emitFatal(err);
      }
    }
    pumping = false;
  }

  rl.on("line", (line) => {
    queue.push(line);
    void pump();
  });
}

main();
