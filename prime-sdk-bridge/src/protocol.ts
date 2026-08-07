/** Shared emit / parse for the Prime JSONL bridge. */

export type BridgeRequest =
  | { op: "create"; cwd: string; model: string; apiKey?: string; noForcePolicy?: string }
  | { op: "send"; prompt: string }
  | { op: "cancel" }
  | { op: "close" };

export function emit(obj: Record<string, unknown>): void {
  process.stdout.write(`${JSON.stringify(obj)}\n`);
}

export function parseRequest(line: string): BridgeRequest {
  const obj = JSON.parse(line) as BridgeRequest;
  if (!obj || typeof obj !== "object" || !("op" in obj)) {
    throw new Error("bridge request missing op");
  }
  return obj;
}

export function exitCodeForSignal(signal: string): number {
  if (signal === "SIGINT") return 130;
  if (signal === "SIGTERM") return 143;
  return 1;
}
