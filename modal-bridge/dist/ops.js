import { num, optStr, str, strList, strMap } from "./protocol.js";
import { TOOLCHAIN_PROBE, toolchainProblem } from "./toolchain.js";
export const APP_NAME = "malvin";
const BUILD_TIMEOUT_MS = 30 * 60 * 1000;
const SNAPSHOT_TIMEOUT_MS = 5 * 60 * 1000;
const REMOTE_BINARY = "/usr/local/bin/malvin";
export class Session {
    modal;
    sandbox = null;
    #app = null;
    constructor(modal) {
        this.modal = modal;
    }
    async app() {
        this.#app ??= await this.modal.apps.fromName(APP_NAME, { createIfMissing: true });
        return this.#app;
    }
    active() {
        if (!this.sandbox) {
            throw new Error("no Sandbox: call create_sandbox first");
        }
        return this.sandbox;
    }
}
async function runText(sb, argv) {
    const proc = await sb.exec(argv);
    const [out, err] = await Promise.all([proc.stdout.readText(), proc.stderr.readText()]);
    const code = await proc.wait();
    return { code, out: out + err };
}
async function mustRun(sb, argv) {
    const { code, out } = await runText(sb, argv);
    if (code !== 0) {
        throw new Error(`\`${argv.join(" ")}\` exited ${code} in the build Sandbox: ${out.trim()}`);
    }
    return out;
}
function layeredImage(modal, req) {
    let image = modal.images.fromRegistry(str(req, "base"));
    const layers = Array.isArray(req.layers) ? req.layers : [];
    for (const layer of layers) {
        const commands = strList({ layer }, "layer");
        if (commands.length > 0) {
            image = image.dockerfileCommands(commands);
        }
    }
    return image;
}
async function populateBuildSandbox(sb, req, emit) {
    const probe = await mustRun(sb, ["sh", "-c", TOOLCHAIN_PROBE]);
    const problem = toolchainProblem(probe, optStr(req, "min_glibc"), req.need_node === true);
    if (problem) {
        throw new Error(problem);
    }
    const binary = optStr(req, "binary");
    if (binary) {
        emit("log", `uploading ${binary} to the Modal image (once per malvin build)…`);
        await sb.filesystem.copyFromLocal(binary, REMOTE_BINARY);
        await mustRun(sb, ["chmod", "+x", REMOTE_BINARY]);
    }
    await mustRun(sb, [REMOTE_BINARY, "--version"]);
}
export async function ensureImage(s, req, emit) {
    const name = str(req, "name");
    if (req.force !== true) {
        try {
            await s.modal.images.fromName(name);
            return { image: name, built: false };
        }
        catch {
            // A missing or expired image is rebuilt below.
        }
    }
    emit("log", `building Modal image ${name}…`);
    const sb = await s.modal.sandboxes.create(await s.app(), layeredImage(s.modal, req), {
        timeoutMs: BUILD_TIMEOUT_MS,
    });
    try {
        await populateBuildSandbox(sb, req, emit);
        const snapshot = await sb.snapshotFilesystem({ timeoutMs: SNAPSHOT_TIMEOUT_MS });
        await snapshot.publish(name);
    }
    finally {
        await sb.terminate();
    }
    return { image: name, built: true };
}
export async function createSandbox(s, req) {
    const image = await s.modal.images.fromName(str(req, "image"));
    s.sandbox = await s.modal.sandboxes.create(await s.app(), image, {
        gpu: optStr(req, "gpu"),
        cpu: num(req, "cpu", 1),
        memoryMiB: num(req, "memory_mib", 4096),
        timeoutMs: num(req, "timeout_ms", 30 * 60 * 1000),
        tags: strMap(req, "tags"),
    });
    return { sandbox_id: s.sandbox.sandboxId };
}
async function pump(stream, kind, emit) {
    for await (const chunk of stream) {
        if (chunk.length > 0) {
            emit(kind, chunk);
        }
    }
}
export async function exec(s, req, emit) {
    const env = strMap(req, "env");
    const proc = await s.active().exec(strList(req, "argv"), {
        workdir: optStr(req, "workdir"),
        env: Object.keys(env).length > 0 ? env : undefined,
    });
    if (req.stream === true) {
        await Promise.all([pump(proc.stdout, "stdout", emit), pump(proc.stderr, "stderr", emit)]);
        return { exit_code: await proc.wait() };
    }
    const [stdout, stderr] = await Promise.all([proc.stdout.readText(), proc.stderr.readText()]);
    return { exit_code: await proc.wait(), stdout, stderr };
}
export async function upload(s, req) {
    await s.active().filesystem.copyFromLocal(str(req, "local"), str(req, "remote"));
    return {};
}
export async function download(s, req) {
    await s.active().filesystem.copyToLocal(str(req, "remote"), str(req, "local"));
    return {};
}
export async function terminate(s) {
    const sb = s.sandbox;
    s.sandbox = null;
    if (sb) {
        await sb.terminate();
    }
    return {};
}
export async function listTagged(s, req) {
    const app = await s.app();
    const found = [];
    for await (const sb of s.modal.sandboxes.list({ appId: app.appId, tags: strMap(req, "tags") })) {
        found.push({ sandbox_id: sb.sandboxId, tags: await sb.getTags() });
    }
    return { sandboxes: found };
}
export async function terminateIds(s, req) {
    const ids = strList(req, "ids");
    for (const id of ids) {
        await (await s.modal.sandboxes.fromId(id)).terminate();
    }
    return { terminated: ids.length };
}
