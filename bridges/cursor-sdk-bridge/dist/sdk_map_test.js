import assert from "node:assert/strict";
import { describe, it } from "node:test";
import { bridgeEventFromSdk, shortenMiddle, toolSummaryFromArgs, } from "./sdk_map.js";
import { parseRequest } from "./protocol.js";
describe("sdk_map", () => {
    it("maps assistant text", () => {
        const ev = bridgeEventFromSdk({
            type: "assistant",
            message: { content: [{ type: "text", text: "hi" }] },
        });
        assert.deepEqual(ev, { event: "assistant", text: "hi" });
    });
    it("maps thinking", () => {
        const ev = bridgeEventFromSdk({
            type: "thinking",
            text: "hmm",
        });
        assert.deepEqual(ev, { event: "thinking", text: "hmm" });
    });
    it("maps tool_call start with enriched summary from args", () => {
        const ev = bridgeEventFromSdk({
            type: "tool_call",
            name: "shell",
            call_id: "t1",
            status: "running",
            args: { command: "ls -ltr /tmp | head" },
        });
        assert.equal(ev?.event, "tool_call");
        if (ev?.event === "tool_call") {
            assert.equal(ev.phase, "start");
            assert.equal(ev.name, "shell");
            assert.equal(ev.summary, "Run ls -ltr /tmp | head");
        }
    });
    it("maps tool_call read/edit/grep subjects", () => {
        assert.equal(toolSummaryFromArgs("read", { path: "/home/me/a.rs" }), "Read /home/me/a.rs");
        assert.equal(toolSummaryFromArgs("edit", { path: "src/foo.rs" }), "Edit src/foo.rs");
        assert.equal(toolSummaryFromArgs("grep", { pattern: "tool_call", path: "src" }), "Search tool_call in src");
    });
    it("maps shell complete with exit failure as error phase", () => {
        const ev = bridgeEventFromSdk({
            type: "tool_call",
            name: "shell",
            call_id: "t2",
            status: "completed",
            args: { command: "false" },
            result: { status: "success", value: { exitCode: 1 } },
        });
        assert.equal(ev?.event, "tool_call");
        if (ev?.event === "tool_call") {
            assert.equal(ev.phase, "error");
            assert.match(ev.summary ?? "", /^Run false · exit 1$/);
        }
    });
    it("enriches read complete with byte size", () => {
        const ev = bridgeEventFromSdk({
            type: "tool_call",
            name: "read",
            status: "completed",
            args: { path: "README.md" },
            result: { status: "success", value: { content: "hi", fileSize: 183, totalLines: 1 } },
        });
        assert.equal(ev?.event, "tool_call");
        if (ev?.event === "tool_call") {
            assert.equal(ev.phase, "complete");
            assert.equal(ev.summary, "Read README.md · 183 B");
        }
    });
    it("shortens long subjects in the middle", () => {
        const long = "a".repeat(80);
        const out = shortenMiddle(long, 60);
        assert.equal(out.length, 60);
        assert.ok(out.includes("..."));
    });
    it("flattens multiline shell commands", () => {
        assert.equal(toolSummaryFromArgs("shell", { command: "echo a\necho b" }), "Run echo a echo b");
    });
});
describe("protocol", () => {
    it("parses create", () => {
        const req = parseRequest(JSON.stringify({ op: "create", cwd: "/tmp", model: "auto" }));
        assert.equal(req.op, "create");
    });
});
