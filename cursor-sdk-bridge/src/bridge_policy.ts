/** Pure helpers for bridge send/cancel policy (unit-tested). */

import { AuthenticationError, CursorAgentError } from "@cursor/sdk";

function errorMessage(err: unknown): string {
  if (err instanceof Error) return err.message;
  return String(err);
}

function isRetryable(err: unknown): boolean {
  return err instanceof CursorAgentError ? Boolean(err.isRetryable) : false;
}

/**
 * Idle local SDK connections can surface as AuthenticationError /
 * ERROR_NOT_LOGGED_IN even when the API key is still valid (stale gRPC /
 * short-lived access token). Consumers should evict the agent handle,
 * Agent.resume(agentId), and retry send — see Cursor forum / err.md.
 */
export function isStaleAuthMisclassification(err: unknown): boolean {
  if (err instanceof AuthenticationError) return true;
  const name = err instanceof Error ? err.name : "";
  return isStaleAuthText(name, errorMessage(err));
}

/** String-level detector for fatal/run_done payloads (no Error instance). */
export function isStaleAuthText(name: string, message: string): boolean {
  const n = name.toLowerCase();
  const m = message.toLowerCase();
  if (n.includes("authentication")) return true;
  if (m === "authentication" || m.startsWith("authentication")) return true;
  if (m.includes("authenticationerror")) return true;
  if (m.includes("error_not_logged_in")) return true;
  if (m.includes("[unauthenticated]") || m.includes("unauthenticated")) {
    return true;
  }
  if (m.includes("logged in") && m.includes("logging out")) return true;
  return false;
}

/**
 * After a stream failure, emit `fatal` only — never a trailing `run_done`.
 * Rust `drain_until_run_done` returns on `fatal`; a leftover `run_done` would
 * be consumed by the next prompt as a stale success.
 */
export function eventsAfterStreamFailure(
  streamErr: unknown,
): Array<{ event: "fatal"; message: string; retryable: boolean }> {
  return [
    {
      event: "fatal",
      message: `stream error: ${errorMessage(streamErr)}`,
      retryable: isRetryable(streamErr),
    },
  ];
}

/** True when this op must interrupt an in-flight send without waiting for it. */
export function isInterruptOp(op: string | undefined): boolean {
  return op === "cancel" || op === "close";
}

/** Exit status for a quiet process signal (no Node stack dump). */
export function exitCodeForSignal(signal: NodeJS.Signals): number {
  if (signal === "SIGINT") return 130;
  if (signal === "SIGTERM") return 143;
  return 1;
}
