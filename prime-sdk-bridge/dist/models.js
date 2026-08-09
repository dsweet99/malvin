/** Full Prime model listing for `malvin models`. */
import { AuthStorage, ModelRegistry } from "prime-agent";
import { emit, exitCodeForSignal } from "./protocol.js";
function installQuietSignalHandlers() {
    for (const signal of ["SIGINT", "SIGTERM"]) {
        process.on(signal, () => {
            process.exit(exitCodeForSignal(signal));
        });
    }
}
async function main() {
    installQuietSignalHandlers();
    try {
        const authStorage = AuthStorage.create();
        const registry = ModelRegistry.create(authStorage);
        const available = await registry.getAvailable();
        const ids = [];
        for (const m of available) {
            ids.push(`${m.provider}/${m.id}`);
        }
        ids.sort();
        for (const id of ids) {
            process.stdout.write(`prime:${id}\n`);
        }
    }
    catch (err) {
        const msg = err instanceof Error ? err.message : String(err);
        emit({ event: "fatal", message: msg, retryable: false });
        process.exit(1);
    }
}
main();
