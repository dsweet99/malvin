import assert from "node:assert/strict";
import { describe, it } from "node:test";
import { modelSelectionFromRaw } from "./model_selection.js";
import { formatModelListLine, formatModelParams } from "./models.js";
describe("modelSelectionFromRaw", () => {
    it("parses bare ids", () => {
        assert.deepEqual(modelSelectionFromRaw("auto"), { id: "auto" });
        assert.deepEqual(modelSelectionFromRaw("  "), { id: "auto" });
    });
    it("parses bracket params", () => {
        assert.deepEqual(modelSelectionFromRaw("claude-opus-5[effort=high,fast=true]"), {
            id: "claude-opus-5",
            params: [
                { id: "effort", value: "high" },
                { id: "fast", value: "true" },
            ],
        });
    });
    it("rejects malformed brackets", () => {
        assert.throws(() => modelSelectionFromRaw("opus[effort=high"), /must end with/);
        assert.throws(() => modelSelectionFromRaw("[effort=high]"), /base id/);
        assert.throws(() => modelSelectionFromRaw("opus[effort]"), /key=value/);
    });
});
describe("formatModelParams", () => {
    it("formats parameter catalogs", () => {
        assert.equal(formatModelParams(undefined), "");
        assert.equal(formatModelParams([
            {
                id: "effort",
                values: [{ value: "low" }, { value: "high" }],
            },
            {
                id: "fast",
                values: [{ value: "false" }, { value: "true" }],
            },
        ]), "effort=low|high fast=false|true");
    });
    it("formats list lines", () => {
        const bare = { id: "auto", displayName: "Auto" };
        assert.equal(formatModelListLine(bare), "cursor:auto");
        const withParams = {
            id: "composer-2.5",
            displayName: "Composer 2.5",
            parameters: [
                {
                    id: "fast",
                    values: [{ value: "false" }, { value: "true" }],
                },
            ],
        };
        assert.equal(formatModelListLine(withParams), "cursor:composer-2.5\tfast=false|true");
    });
});
