/** Malvin-owned JSONL bridge protocol (Rust ↔ Node). */

export type NoForcePolicy = "sandbox" | "fail_fast" | "auto_review";

export type CreateOp = {
  op: "create";
  cwd: string;
  model: string;
  apiKey?: string;
  settingSources?: string[];
  sandboxEnabled?: boolean;
  autoReview?: boolean;
  noForcePolicy?: NoForcePolicy;
};

/** Re-open a prior agent id after evicting a stale handle (idle auth bug). */
export type ResumeOp = {
  op: "resume";
  agentId: string;
  cwd: string;
  model: string;
  apiKey?: string;
  settingSources?: string[];
  sandboxEnabled?: boolean;
  autoReview?: boolean;
  noForcePolicy?: NoForcePolicy;
};

export type SendOp = {
  op: "send";
  prompt: string;
  runId?: string;
  forceStuck?: boolean;
};

export type CancelOp = { op: "cancel" };
export type CloseOp = { op: "close" };
export type ListModelsOp = { op: "list_models"; apiKey?: string };

export type BridgeRequest =
  | CreateOp
  | ResumeOp
  | SendOp
  | CancelOp
  | CloseOp
  | ListModelsOp;

export type BridgeEvent =
  | { event: "ok"; agentId?: string }
  | { event: "assistant"; text: string }
  | { event: "thinking"; text: string }
  | {
      event: "tool_call";
      phase: "start" | "complete" | "error";
      name?: string;
      summary?: string;
      toolCallId?: string;
    }
  | { event: "step"; kind?: string }
  | { event: "usage"; usage: Record<string, number> }
  | {
      event: "run_done";
      status: string;
      result?: string;
      usage?: Record<string, number>;
      error?: string;
      durationMs?: number;
    }
  | {
      event: "models";
      models: Array<{
        id: string;
        displayName?: string;
        parameters?: Array<{ id: string; values: string[] }>;
      }>;
    }
  | { event: "fatal"; message: string; retryable?: boolean };

export function emit(ev: BridgeEvent): void {
  process.stdout.write(`${JSON.stringify(ev)}\n`);
}

export function parseRequest(line: string): BridgeRequest {
  const raw = JSON.parse(line) as BridgeRequest;
  if (!raw || typeof raw !== "object" || !("op" in raw)) {
    throw new Error("bridge request missing op");
  }
  return raw;
}
