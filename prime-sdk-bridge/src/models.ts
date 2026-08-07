/** Full Prime model listing for `malvin models`. */

import { AuthStorage, ModelRegistry } from "prime-agent";
import { emit, exitCodeForSignal } from "./protocol.js";

function installQuietSignalHandlers(): void {
  for (const signal of ["SIGINT", "SIGTERM"] as const) {
    process.on(signal, () => {
      process.exit(exitCodeForSignal(signal));
    });
  }
}

async function main(): Promise<void> {
  installQuietSignalHandlers();
  try {
    const authStorage = AuthStorage.create();
    const registry = ModelRegistry.create(authStorage);
    const available = await registry.getAvailable();
    const ids: string[] = [];
    for (const m of available) {
      ids.push(`${m.provider}/${m.id}`);
    }
    ids.sort();
    for (const id of ids) {
      process.stdout.write(`prime:${id}\n`);
    }
  } catch (err) {
    const msg = err instanceof Error ? err.message : String(err);
    emit({ event: "fatal", message: msg, retryable: false });
    process.exit(1);
  }
}

main();
