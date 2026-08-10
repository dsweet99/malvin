/** One-shot Cursor model listing for `malvin models`. */
import { Cursor, CursorAgentError } from "@cursor/sdk";
import { emit } from "./protocol.js";
import { exitCodeForSignal } from "./bridge_policy.js";
function installQuietSignalHandlers() {
    for (const signal of ["SIGINT", "SIGTERM"]) {
        process.on(signal, () => {
            process.exit(exitCodeForSignal(signal));
        });
    }
}
async function main() {
    installQuietSignalHandlers();
    const apiKey = (process.env.CURSOR_API_KEY ??
        process.env.CURSOR_AGENT_API_KEY ??
        process.env.AGENT_API_KEY ??
        "").trim();
    if (!apiKey) {
        emit({
            event: "fatal",
            message: "CURSOR_API_KEY required to list Cursor models",
            retryable: false,
        });
        process.exit(1);
    }
    try {
        const models = await Cursor.models.list({ apiKey });
        for (const m of models) {
            process.stdout.write(`cursor:${m.id}\n`);
        }
    }
    catch (err) {
        const msg = err instanceof Error ? err.message : String(err);
        const retryable = err instanceof CursorAgentError ? Boolean(err.isRetryable) : false;
        emit({ event: "fatal", message: msg, retryable });
        process.exit(1);
    }
}
main();
