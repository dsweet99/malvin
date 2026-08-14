/**
 * Long-lived malvin ↔ @cursor/sdk JSONL bridge.
 * One Agent.create per process; many agent.send calls.
 *
 * Cancel/close are handled concurrently with an in-flight send: stdin is read
 * into a queue so cancel is not blocked behind `await handleSend`.
 */
// Arm parent-death watch before heavy imports (see parent_death.ts).
import "./parent_death.js";
import * as readline from "node:readline";
import { Agent, Cursor, CursorAgentError, configureCursorSdk } from "@cursor/sdk";
import { modelSelectionFromRaw } from "./model_selection.js";
import { emit, parseRequest, } from "./protocol.js";
import { eventsAfterStreamFailure, exitCodeForSignal, isInterruptOp, isStaleAuthMisclassification, isStaleAuthText, } from "./bridge_policy.js";
import { forwardSdkMessage, usageRecord } from "./sdk_map.js";
export { eventsAfterStreamFailure, exitCodeForSignal, isInterruptOp, isStaleAuthMisclassification, isStaleAuthText, } from "./bridge_policy.js";
export { installParentDeathWatch } from "./parent_death.js";
let agent = null;
/** Last successful create/resume agent id (for stale-auth resume). */
let agentId = null;
/** Options used for the live agent (needed by Agent.resume). */
let agentOptions = null;
let currentRun = null;
let closing = false;
function quietExit(code) {
    closing = true;
    // Avoid waiting on dispose/cancel — those can dump stacks while dying.
    process.exit(code);
}
function installQuietSignalHandlers() {
    for (const signal of ["SIGINT", "SIGTERM"]) {
        process.on(signal, () => {
            quietExit(exitCodeForSignal(signal));
        });
    }
    process.on("uncaughtException", (err) => {
        if (closing)
            return;
        emitFatal(err);
        quietExit(1);
    });
    process.on("unhandledRejection", (reason) => {
        if (closing)
            return;
        emitFatal(reason);
        // Do not process.exit here during normal ops — emit fatal and let Rust decide.
        // During signal shutdown, `closing` suppresses dumps.
    });
}
function apiKeyFrom(req) {
    const k = (req.apiKey ?? process.env.CURSOR_API_KEY ?? "").trim();
    return k.length > 0 ? k : undefined;
}
function errorMessage(err) {
    if (err instanceof Error)
        return err.message;
    return String(err);
}
function isRetryable(err) {
    return err instanceof CursorAgentError ? Boolean(err.isRetryable) : false;
}
function emitFatal(err) {
    emit({
        event: "fatal",
        message: errorMessage(err),
        retryable: isRetryable(err),
    });
}
function preferHttp1ForAgent() {
    // HTTP/2 agent streams often fail through authenticated CONNECT proxies
    // (pier egress): SDK retries end as "Connection failed repeatedly", while
    // cursor-agent CLI and HTTP/1 succeed on the same network.
    const flag = (process.env.MALVIN_CURSOR_USE_HTTP1 || "").trim().toLowerCase();
    if (flag === "1" || flag === "true" || flag === "yes")
        return true;
    if (flag === "0" || flag === "false" || flag === "no")
        return false;
    return Boolean((process.env.HTTPS_PROXY || "").trim() ||
        (process.env.HTTP_PROXY || "").trim());
}
/** Apply once per bridge process before Agent.create/resume. */
function ensureHttp1AgentTransport() {
    if (!preferHttp1ForAgent())
        return;
    configureCursorSdk({ local: { useHttp1ForAgent: true } });
}
function agentOptionsFromBoot(req, apiKey) {
    const local = {
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
function rejectNoForce(req) {
    if (req.noForcePolicy !== "fail_fast")
        return false;
    emit({
        event: "fatal",
        message: "--no-force is not supported with the Cursor SDK backend (no interactive tool approval). Omit --no-force.",
        retryable: false,
    });
    return true;
}
function requireApiKey(req) {
    const apiKey = apiKeyFrom(req);
    if (apiKey)
        return apiKey;
    emit({
        event: "fatal",
        message: "CURSOR_API_KEY is required for the Cursor SDK backend (also accepts CURSOR_AGENT_API_KEY / AGENT_API_KEY via malvin).",
        retryable: false,
    });
    return undefined;
}
async function handleCreate(req) {
    if (agent) {
        emit({ event: "fatal", message: "agent already created", retryable: false });
        return;
    }
    if (rejectNoForce(req))
        return;
    const apiKey = requireApiKey(req);
    if (!apiKey)
        return;
    try {
        ensureHttp1AgentTransport();
        const opts = agentOptionsFromBoot(req, apiKey);
        agent = await Agent.create(opts);
        agentOptions = opts;
        agentId = agent.agentId;
        emit({ event: "ok", agentId });
    }
    catch (err) {
        emitFatal(err);
    }
}
async function handleResume(req) {
    if (agent) {
        emit({ event: "fatal", message: "agent already created", retryable: false });
        return;
    }
    if (rejectNoForce(req))
        return;
    const apiKey = requireApiKey(req);
    if (!apiKey)
        return;
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
    }
    catch (err) {
        emitFatal(err);
    }
}
/** Evict cached handle and Agent.resume — forum workaround for idle auth. */
async function recoverStaleAuth() {
    if (!agentId || !agentOptions)
        return false;
    const id = agentId;
    const opts = agentOptions;
    agent = null;
    try {
        agent = await Agent.resume(id, opts);
        agentId = agent.agentId || id;
        return true;
    }
    catch {
        agent = null;
        return false;
    }
}
async function handleSend(req, alreadyRecovered = false) {
    if (!agent) {
        emit({ event: "fatal", message: "create before send", retryable: false });
        return;
    }
    const started = Date.now();
    try {
        currentRun = await agent.send(req.prompt, {
            onStep: () => {
                emit({ event: "step", kind: "onStep" });
            },
            local: req.forceStuck ? { force: true } : undefined,
        });
        try {
            for await (const msg of currentRun.stream()) {
                forwardSdkMessage(msg);
            }
        }
        catch (streamErr) {
            if (!alreadyRecovered &&
                isStaleAuthMisclassification(streamErr) &&
                (await recoverStaleAuth())) {
                currentRun = null;
                return handleSend(req, true);
            }
            for (const ev of eventsAfterStreamFailure(streamErr)) {
                emit(ev);
            }
            try {
                await currentRun.wait();
            }
            catch {
                // best-effort drain of the failed run
            }
            currentRun = null;
            return;
        }
        const result = await currentRun.wait();
        currentRun = null;
        const errObj = result.error;
        const errMsg = errObj?.message ?? "";
        if ((result.status === "error" || result.status === "cancelled") &&
            errMsg &&
            !alreadyRecovered &&
            isStaleAuthText("", errMsg) &&
            (await recoverStaleAuth())) {
            return handleSend(req, true);
        }
        emit({
            event: "run_done",
            status: result.status,
            result: result.result,
            usage: usageRecord(result.usage),
            error: errObj?.message,
            durationMs: result.durationMs ?? Date.now() - started,
        });
    }
    catch (err) {
        currentRun = null;
        if (!alreadyRecovered &&
            isStaleAuthMisclassification(err) &&
            (await recoverStaleAuth())) {
            return handleSend(req, true);
        }
        emitFatal(err);
    }
}
async function handleCancel() {
    const run = currentRun;
    if (!run)
        return;
    try {
        if (run.supports("cancel")) {
            await run.cancel();
        }
    }
    catch {
        // best-effort
    }
}
async function handleClose() {
    closing = true;
    await handleCancel();
    agent = null;
    agentId = null;
    agentOptions = null;
    // Skip asyncDispose: malvin tears down the process group immediately after
    // close; dispose can add hundreds of ms (ideas.md #10–11).
    quietExit(0);
}
async function handleListModels(apiKey) {
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
    }
    catch (err) {
        emitFatal(err);
    }
}
async function dispatch(req) {
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
                message: `unknown op: ${req.op}`,
                retryable: false,
            });
    }
}
async function main() {
    installQuietSignalHandlers();
    const rl = readline.createInterface({
        input: process.stdin,
        crlfDelay: Infinity,
    });
    const queue = [];
    let wake = null;
    let stdinClosed = false;
    rl.on("line", (line) => {
        queue.push(line);
        wake?.();
    });
    rl.on("close", () => {
        // Parent (malvin) died or closed the pipe — exit quietly; do not leave
        // in-flight SDK work free to dump unhandledRejection stacks on stderr.
        stdinClosed = true;
        wake?.();
        if (!closing) {
            closing = true;
            void handleCancel().finally(() => quietExit(0));
        }
    });
    async function nextLine() {
        for (;;) {
            if (queue.length > 0) {
                return queue.shift() ?? null;
            }
            if (stdinClosed) {
                return null;
            }
            await new Promise((resolve) => {
                wake = resolve;
            });
            wake = null;
        }
    }
    let serial = Promise.resolve();
    for (;;) {
        if (closing)
            break;
        const line = await nextLine();
        if (line === null)
            break;
        const trimmed = line.trim();
        if (!trimmed)
            continue;
        let req;
        try {
            req = parseRequest(trimmed);
        }
        catch (err) {
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
            }
            catch (err) {
                emit({
                    event: "fatal",
                    message: `bad request: ${errorMessage(err)}`,
                    retryable: false,
                });
            }
            if (req.op === "close")
                break;
            continue;
        }
        // Serialize create/send/list_models, but do not await here so cancel can interleave.
        serial = serial
            .then(() => dispatch(req))
            .catch((err) => {
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
