import assert from "node:assert/strict";
import { describe, it } from "node:test";
import {
  eventsAfterStreamFailure,
  exitCodeForSignal,
  isInterruptOp,
  isStaleAuthText,
  progressHeartbeatDue,
  runAcceptsProgressHeartbeat,
} from "./bridge_policy.js";
import { canonicalRunDoneStatus } from "./protocol.js";

describe("stale auth misclassification", () => {
  it("detects forum Authentication / idle-token forms", () => {
    assert.equal(isStaleAuthText("AuthenticationError", "x"), true);
    assert.equal(isStaleAuthText("", "Authentication"), true);
    assert.equal(isStaleAuthText("", "ERROR_NOT_LOGGED_IN"), true);
    assert.equal(isStaleAuthText("", "[unauthenticated] Error"), true);
    assert.equal(
      isStaleAuthText(
        "",
        "If you are logged in, try logging out and back in.",
      ),
      true,
    );
    assert.equal(isStaleAuthText("", "request timed out"), false);
  });
});

describe("canonical run_done status", () => {
  it("collapses aliases to finished, error, or cancelled", () => {
    assert.equal(canonicalRunDoneStatus("completed"), "finished");
    assert.equal(canonicalRunDoneStatus("finished"), "finished");
    assert.equal(canonicalRunDoneStatus("failed"), "error");
    assert.equal(canonicalRunDoneStatus("interrupted"), "cancelled");
    assert.equal(canonicalRunDoneStatus("bogus"), "error");
  });
});

describe("bridge stream failure protocol", () => {
  it("emits fatal only — never a trailing run_done", () => {
    const events = eventsAfterStreamFailure(new Error("boom"));
    assert.equal(events.length, 1);
    assert.equal(events[0]?.event, "fatal");
    assert.match(events[0]?.message ?? "", /^stream error: boom$/);
    assert.equal(
      events.some((e) => (e as { event?: string }).event === "run_done"),
      false,
    );
  });
});

describe("bridge interrupt ops", () => {
  it("treats cancel and close as interrupts", () => {
    assert.equal(isInterruptOp("cancel"), true);
    assert.equal(isInterruptOp("close"), true);
    assert.equal(isInterruptOp("send"), false);
    assert.equal(isInterruptOp("create"), false);
    assert.equal(isInterruptOp("resume"), false);
  });
});

describe("progress heartbeat policy", () => {
  it("fires at 15s only while a run is open", () => {
    assert.equal(progressHeartbeatDue(true, false, 1000, 15_999), false);
    assert.equal(progressHeartbeatDue(true, false, 1000, 16_000), true);
    assert.equal(progressHeartbeatDue(false, false, 1000, 20_000), false);
    assert.equal(progressHeartbeatDue(true, true, 1000, 20_000), false);
  });

  it("treats runPending like an open run for heartbeats", () => {
    assert.equal(
      runAcceptsProgressHeartbeat(false, true, false, 1000, 16_000),
      true,
    );
    assert.equal(
      runAcceptsProgressHeartbeat(false, false, false, 1000, 16_000),
      false,
    );
  });
});

describe("parent death watch", () => {
  it("exports installParentDeathWatch and arms a timer", async () => {
    const { installParentDeathWatch } = await import("./parent_death.js");
    let exited: number | null = null;
    const timer = installParentDeathWatch((code) => {
      exited = code;
    }, 10_000);
    assert.equal(typeof timer.refresh, "function");
    clearInterval(timer);
    assert.equal(exited, null);
  });
});
