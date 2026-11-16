import "./parent_death.js";
import { query } from "@anthropic-ai/claude-agent-sdk";
async function* idle() {
    await new Promise(() => { });
}
const session = query({ prompt: idle(), options: { cwd: process.cwd() } });
try {
    const models = await session.supportedModels();
    process.stdout.write(`${JSON.stringify(models)}\n`);
    session.close();
    process.exit(0);
}
catch (error) {
    process.stderr.write(`${error instanceof Error ? error.message : String(error)}\n`);
    session.close();
    process.exit(1);
}
