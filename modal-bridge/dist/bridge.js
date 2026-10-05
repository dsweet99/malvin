#!/usr/bin/env node
import { createInterface } from "node:readline";
import { ModalClient } from "modal";
import { handle } from "./dispatch.js";
import { Session, terminate } from "./ops.js";
import { encodeLine, errorText, parseRequest } from "./protocol.js";
const session = new Session(new ModalClient());
let shuttingDown = false;
async function shutdown(code) {
    if (!shuttingDown) {
        shuttingDown = true;
        try {
            await terminate(session);
        }
        catch (e) {
            process.stderr.write(`modal-bridge: terminate failed: ${errorText(e)}\n`);
        }
    }
    process.exit(code);
}
for (const signal of ["SIGINT", "SIGTERM", "SIGHUP"]) {
    process.on(signal, () => void shutdown(130));
}
const write = (v) => process.stdout.write(encodeLine(v));
const lines = createInterface({ input: process.stdin, crlfDelay: Infinity });
for await (const line of lines) {
    if (line.trim() === "") {
        continue;
    }
    try {
        await handle(session, parseRequest(line), write);
    }
    catch (e) {
        write({ id: -1, ok: false, error: errorText(e) });
    }
}
await shutdown(0);
