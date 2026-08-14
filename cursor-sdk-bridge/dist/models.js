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
/** Compact `id=v1|v2` summaries for catalog params (thinking / effort / fast / …). */
export function formatModelParams(parameters) {
    if (!parameters || parameters.length === 0) {
        return "";
    }
    return parameters
        .map((p) => {
        const values = p.values.map((v) => v.value).join("|");
        return `${p.id}=${values}`;
    })
        .join(" ");
}
export function formatModelListLine(m) {
    const params = formatModelParams(m.parameters);
    return params ? `cursor:${m.id}\t${params}` : `cursor:${m.id}`;
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
            process.stdout.write(`${formatModelListLine(m)}\n`);
        }
    }
    catch (err) {
        const msg = err instanceof Error ? err.message : String(err);
        const retryable = err instanceof CursorAgentError ? Boolean(err.isRetryable) : false;
        emit({ event: "fatal", message: msg, retryable });
        process.exit(1);
    }
}
const isMain = typeof process.argv[1] === "string" &&
    (process.argv[1].endsWith("/models.js") ||
        process.argv[1].endsWith("/models.ts") ||
        process.argv[1].endsWith("models.js"));
if (isMain) {
    void main();
}
