import assert from "node:assert/strict";
import { describe, it } from "node:test";
import {
  enrichCompleteSummary,
  flatten,
  shortenMiddle,
  toolPhaseFromEnd,
  toolSummaryFromArgs,
} from "./tool_summary.js";

describe("toolSummaryFromArgs", () => {
  it("maps ipython %%bash cells to Run <cmd>", () => {
    assert.equal(
      toolSummaryFromArgs("ipython", { code: "%%bash\nls -la\n" }),
      "Run ls -la",
    );
  });

  it("maps ipython bang shell to Run <cmd>", () => {
    assert.equal(toolSummaryFromArgs("ipython", { code: "!ls" }), "Run ls");
  });

  it("maps plain ipython code", () => {
    assert.equal(
      toolSummaryFromArgs("ipython", { code: "print(1)" }),
      "ipython: print(1)",
    );
  });

  it("falls back to tool name without code", () => {
    assert.equal(toolSummaryFromArgs("ipython", {}), "ipython");
  });

  it("maps shell command", () => {
    assert.equal(
      toolSummaryFromArgs("shell", { command: "echo hi" }),
      "Run echo hi",
    );
  });
});

describe("toolPhaseFromEnd", () => {
  it("maps success to complete", () => {
    assert.equal(toolPhaseFromEnd(false), "complete");
  });
  it("maps failure to error", () => {
    assert.equal(toolPhaseFromEnd(true), "error");
  });
});

describe("enrichCompleteSummary", () => {
  it("keeps start summary on success without exit", () => {
    assert.equal(
      enrichCompleteSummary("Run ls -la", "ipython", { status: "ok" }, false),
      "Run ls -la",
    );
  });

  it("appends exit code on Run errors (cursor parity)", () => {
    assert.equal(
      enrichCompleteSummary("Run false", "ipython", { exitCode: 1 }, true),
      "Run false · exit 1",
    );
  });

  it("does not duplicate exit segment", () => {
    assert.equal(
      enrichCompleteSummary("Run false · exit 1", "shell", { exitCode: 1 }, true),
      "Run false · exit 1",
    );
  });
});

describe("shortenMiddle / flatten", () => {
  it("flattens whitespace", () => {
    assert.equal(flatten("a\n  b"), "a b");
  });
  it("shortens long strings", () => {
    const s = "x".repeat(80);
    const out = shortenMiddle(s, 20);
    assert.ok(out.includes("..."));
    assert.equal(out.length, 20);
  });
});
