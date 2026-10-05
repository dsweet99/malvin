import assert from "node:assert/strict";
import { test } from "node:test";
import { handle } from "./dispatch.js";
import { Session } from "./ops.js";
import { parseRequest } from "./protocol.js";
import { toolchainProblem } from "./toolchain.js";
function textStream(text) {
    return {
        async *[Symbol.asyncIterator]() {
            if (text)
                yield text;
        },
        readText: async () => text,
    };
}
function mockModal(log, opts = {}) {
    const image = (id) => ({
        imageId: id,
        dockerfileCommands: (c) => (log.push(`layer:${c.join("|")}`), image(`${id}+`)),
        publish: async (name) => void log.push(`publish:${name}`),
    });
    const sandbox = (id) => ({
        sandboxId: id,
        exec: async (argv, params) => {
            log.push(`exec:${argv.join(" ")}:${params?.workdir ?? ""}:${JSON.stringify(params?.env ?? {})}`);
            const out = argv[0] === "sh" ? (opts.probe ?? "ldd (Debian GLIBC 2.41-12) 2.41\nv22.20.0\n") : "out";
            return { stdout: textStream(out), stderr: textStream(""), wait: async () => 0 };
        },
        filesystem: {
            copyFromLocal: async (l, r) => void log.push(`up:${l}->${r}`),
            copyToLocal: async (r, l) => void log.push(`down:${r}->${l}`),
        },
        snapshotFilesystem: async () => (log.push("snapshot"), image("snap")),
        getTags: async () => ({ "malvin-pid": "1" }),
        terminate: async () => void log.push(`terminate:${id}`),
    });
    return {
        apps: { fromName: async () => ({ appId: "ap-1" }) },
        images: {
            fromName: async (name) => {
                if (!opts.published)
                    throw new Error(`NotFound ${name}`);
                return image("im-pub");
            },
            fromRegistry: (tag) => (log.push(`registry:${tag}`), image("im-base")),
        },
        sandboxes: {
            create: async (_app, img, params) => (log.push(`create:${img.imageId}:${params.timeoutMs}:${params.cpu}:${params.gpu ?? "none"}`), sandbox("sb-1")),
            list: async function* () {
                yield sandbox("sb-old");
            },
            fromId: async (id) => sandbox(id),
        },
    };
}
async function call(s, line) {
    const out = [];
    await handle(s, parseRequest(line), (v) => out.push(v));
    return out;
}
test("ensure_image builds, uploads the binary, and publishes when missing", async () => {
    const log = [];
    const s = new Session(mockModal(log));
    const req = { id: 1, op: "ensure_image", name: "malvin-bin:t", base: "node:22", layers: [["RUN a"]], binary: "/b/malvin", min_glibc: "2.31", need_node: true };
    const out = await call(s, JSON.stringify(req));
    assert.deepEqual(out.at(-1), { id: 1, ok: true, image: "malvin-bin:t", built: true });
    assert.ok(log.includes("layer:RUN a"));
    assert.ok(log.includes("up:/b/malvin->/usr/local/bin/malvin"));
    assert.ok(log.indexOf("snapshot") < log.indexOf("publish:malvin-bin:t"));
    assert.ok(log.includes("terminate:sb-1"));
});
test("ensure_image reuses a published image", async () => {
    const log = [];
    const out = await call(new Session(mockModal(log, { published: true })), '{"id":2,"op":"ensure_image","name":"n","base":"b"}');
    assert.deepEqual(out.at(-1), { id: 2, ok: true, image: "n", built: false });
    assert.equal(log.length, 0);
});
test("ensure_image refuses to publish an image with old glibc", async () => {
    const log = [];
    const s = new Session(mockModal(log, { probe: "ldd (GNU libc) 2.28\nv22.20.0\n" }));
    const out = await call(s, '{"id":3,"op":"ensure_image","name":"n","base":"b","min_glibc":"2.31"}');
    assert.equal(out.at(-1)?.ok, false);
    assert.match(String(out.at(-1)?.error), /glibc 2.28/);
    assert.ok(!log.some((l) => l.startsWith("publish")));
    assert.ok(log.includes("terminate:sb-1"));
});
test("exec streams output events, forwards env, and returns the exit code", async () => {
    const log = [];
    const s = new Session(mockModal(log, { published: true }));
    await call(s, '{"id":4,"op":"create_sandbox","image":"n","timeout_ms":5,"cpu":3,"gpu":"T4:2","tags":{"malvin-run":"r"}}');
    assert.ok(log.includes("create:im-pub:5:3:T4:2"));
    const out = await call(s, '{"id":5,"op":"exec","argv":["malvin","x"],"workdir":"/w","env":{"K":"v"},"stream":true}');
    assert.deepEqual(out, [
        { id: 5, event: "stdout", data: "out" },
        { id: 5, ok: true, exit_code: 0 },
    ]);
    assert.ok(log.includes('exec:malvin x:/w:{"K":"v"}'));
});
test("file ops, listing, and terminate", async () => {
    const log = [];
    const s = new Session(mockModal(log, { published: true }));
    assert.equal((await call(s, '{"id":6,"op":"upload","local":"a","remote":"/b"}'))[0].ok, false);
    await call(s, '{"id":7,"op":"create_sandbox","image":"n","gpu":null}');
    assert.ok(log.includes("create:im-pub:1800000:1:none"));
    await call(s, '{"id":8,"op":"upload","local":"a","remote":"/b"}');
    await call(s, '{"id":9,"op":"download","remote":"/c","local":"d"}');
    const listed = await call(s, '{"id":10,"op":"list_tagged","tags":{"malvin-host":"h"}}');
    assert.deepEqual(listed[0].sandboxes, [{ sandbox_id: "sb-old", tags: { "malvin-pid": "1" } }]);
    await call(s, '{"id":11,"op":"terminate_ids","ids":["sb-old"]}');
    await call(s, '{"id":12,"op":"terminate"}');
    assert.deepEqual(log.filter((l) => !l.startsWith("create")), ["up:a->/b", "down:/c->d", "terminate:sb-old", "terminate:sb-1"]);
    assert.equal((await call(s, '{"id":13,"op":"bogus"}'))[0].ok, false);
});
test("toolchainProblem names the missing piece", () => {
    assert.equal(toolchainProblem("ldd (Debian GLIBC 2.41-12) 2.41\nv22.20.0", "2.31", true), null);
    assert.match(String(toolchainProblem("ldd 2.41\nnode-missing", "2.31", true)), /Node >= 22.13/);
    assert.match(String(toolchainProblem("ldd 2.41\nv20.19.2", undefined, true)), /v20.19.2/);
    assert.equal(toolchainProblem("ldd 2.41\nnode-missing", "2.31", false), null);
});
