export type CreateOp = { op: "create"; cwd: string; model: string };
export type ResumeOp = { op: "resume"; agentId: string; cwd: string; model: string };
export type SendOp = { op: "send"; prompt: string };
export type BridgeRequest = CreateOp | ResumeOp | SendOp | { op: "cancel" } | { op: "close" };

export type BridgeEvent =
  | { event: "ok"; agentId?: string }
  | { event: "assistant"; text: string }
  | { event: "thinking"; text: string }
  | { event: "tool_call"; phase: "start" | "complete" | "error"; name?: string; summary?: string; toolCallId?: string }
  | { event: "step"; kind?: string }
  | { event: "usage"; usage: Record<string, number> }
  | { event: "run_done"; status: "finished" | "error" | "cancelled"; result?: string; usage?: Record<string, number>; error?: string; durationMs?: number }
  | { event: "progress"; kind?: string }
  | { event: "fatal"; message: string; retryable?: boolean };

export function emit(event: BridgeEvent): void {
  process.stdout.write(`${JSON.stringify(event)}\n`);
}

export function parseRequest(line: string): BridgeRequest {
  const request: unknown = JSON.parse(line);
  if (!request || typeof request !== "object" || !("op" in request)) {
    throw new Error("bridge request missing op");
  }
  return request as BridgeRequest;
}
