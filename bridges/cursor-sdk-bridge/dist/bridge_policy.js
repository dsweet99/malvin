import { AuthenticationError, CursorAgentError } from "@cursor/sdk";
export const PROGRESS_HEARTBEAT_MS = 15_000;
export function progressHeartbeatDue(runOpen, closing, lastForwardedAt, now) {
    return runOpen && !closing && now - lastForwardedAt >= PROGRESS_HEARTBEAT_MS;
}
export function runAcceptsProgressHeartbeat(currentRun, runPending, closing, lastForwardedAt, now) {
    return progressHeartbeatDue(currentRun || runPending, closing, lastForwardedAt, now);
}
function errorMessage(err) {
    if (err instanceof Error)
        return err.message;
    return String(err);
}
function isRetryable(err) {
    return err instanceof CursorAgentError ? Boolean(err.isRetryable) : false;
}
export function isStaleAuthMisclassification(err) {
    if (err instanceof AuthenticationError)
        return true;
    const name = err instanceof Error ? err.name : "";
    return isStaleAuthText(name, errorMessage(err));
}
export function isStaleAuthText(name, message) {
    const n = name.toLowerCase();
    const m = message.toLowerCase();
    if (n.includes("authentication"))
        return true;
    if (m === "authentication" || m.startsWith("authentication"))
        return true;
    if (m.includes("authenticationerror"))
        return true;
    if (m.includes("error_not_logged_in"))
        return true;
    if (m.includes("[unauthenticated]") || m.includes("unauthenticated")) {
        return true;
    }
    if (m.includes("logged in") && m.includes("logging out"))
        return true;
    return false;
}
export function eventsAfterStreamFailure(streamErr) {
    return [
        {
            event: "fatal",
            message: `stream error: ${errorMessage(streamErr)}`,
            retryable: isRetryable(streamErr),
        },
    ];
}
export function isInterruptOp(op) {
    return op === "cancel" || op === "close";
}
export function exitCodeForSignal(signal) {
    if (signal === "SIGINT")
        return 130;
    if (signal === "SIGTERM")
        return 143;
    return 1;
}
