
import { emit } from "./protocol.js";
const SUMMARY_MAX = 60;
function textFromBlocks(msg) {
    const blocks = msg.message?.content;
    if (!Array.isArray(blocks))
        return "";
    return blocks
        .filter((b) => b && (b.type === "text" || typeof b.text === "string"))
        .map((b) => b.text ?? "")
        .join("");
}
function flatten(s) {
    return s.replace(/\s+/g, " ").trim();
}

export function shortenMiddle(s, max = SUMMARY_MAX) {
    if (s.length <= max)
        return s;
    const keep = max - 3;
    const left = Math.ceil(keep / 2);
    const right = Math.floor(keep / 2);
    return `${s.slice(0, left)}...${s.slice(s.length - right)}`;
}
function asRecord(v) {
    if (v && typeof v === "object" && !Array.isArray(v)) {
        return v;
    }
    return null;
}
function strField(obj, ...keys) {
    for (const k of keys) {
        const v = obj[k];
        if (typeof v === "string" && v.trim())
            return v;
    }
    return undefined;
}
function resultValue(result) {
    const r = asRecord(result);
    if (!r)
        return null;
    return asRecord(r.value) ?? r;
}
function humanizeBytes(n) {
    if (n < 1024)
        return `${n} B`;
    if (n < 1024 * 1024) {
        const kb = n / 1024;
        return kb < 10 ? `${kb.toFixed(1)} KB` : `${Math.round(kb)} KB`;
    }
    const mb = n / (1024 * 1024);
    return mb < 10 ? `${mb.toFixed(1)} MB` : `${Math.round(mb)} MB`;
}
function extractExitCode(result) {
    const v = resultValue(result);
    if (!v)
        return undefined;
    const code = v.exitCode;
    return typeof code === "number" ? code : undefined;
}
function extractFileSize(result) {
    const v = resultValue(result);
    if (!v)
        return undefined;
    if (typeof v.fileSize === "number")
        return v.fileSize;
    if (typeof v.content === "string")
        return v.content.length;
    return undefined;
}
function editCounts(result) {
    const v = resultValue(result);
    if (!v)
        return "";
    const added = typeof v.linesAdded === "number" ? v.linesAdded : undefined;
    const removed = typeof v.linesRemoved === "number" ? v.linesRemoved : undefined;
    if (added !== undefined && removed !== undefined)
        return `+${added}/−${removed}`;
    if (added !== undefined)
        return `+${added}`;
    if (removed !== undefined)
        return `−${removed}`;
    return "";
}

export function toolSummaryFromArgs(name, args) {
    const label = (name ?? "tool").trim() || "tool";
    const a = asRecord(args) ?? {};
    const n = label.toLowerCase();
    if (n === "shell" || n === "bash") {
        const cmd = strField(a, "command");
        return cmd ? `Run ${shortenMiddle(flatten(cmd))}` : "Run";
    }
    if (n === "read") {
        const path = strField(a, "path");
        return path ? `Read ${shortenMiddle(path)}` : "Read";
    }
    if (n === "edit" || n === "strreplace") {
        const path = strField(a, "path");
        return path ? `Edit ${shortenMiddle(path)}` : "Edit";
    }
    if (n === "write") {
        const path = strField(a, "path");
        return path ? `Write ${shortenMiddle(path)}` : "Write";
    }
    if (n === "delete") {
        const path = strField(a, "path");
        return path ? `Delete ${shortenMiddle(path)}` : "Delete";
    }
    if (n === "grep") {
        const pattern = strField(a, "pattern");
        const path = strField(a, "path");
        if (pattern && path) {
            return `Search ${shortenMiddle(flatten(`${pattern} in ${path}`))}`;
        }
        if (pattern)
            return `Search ${shortenMiddle(flatten(pattern))}`;
        return "Search";
    }
    if (n === "glob") {
        const pat = strField(a, "globPattern", "glob_pattern", "pattern");
        return pat ? `Glob ${shortenMiddle(pat)}` : "Glob";
    }
    if (n === "websearch") {
        const q = strField(a, "search_term", "query", "searchTerm");
        return q ? `WebSearch ${shortenMiddle(flatten(q))}` : "WebSearch";
    }
    if (n === "task") {
        const d = strField(a, "description", "prompt");
        return d ? `Task ${shortenMiddle(flatten(d))}` : "Task";
    }
    const hint = strField(a, "command", "path", "pattern", "globPattern", "query", "search_term", "description");
    if (hint)
        return `${label} ${shortenMiddle(flatten(hint))}`;
    return label;
}
function enrichCompleteSummary(base, name, result) {
    const n = (name ?? "").toLowerCase();
    if (n === "read") {
        const size = extractFileSize(result);
        if (size !== undefined)
            return `${base} · ${humanizeBytes(size)}`;
    }
    if (n === "edit" || n === "strreplace") {
        const counts = editCounts(result);
        if (counts)
            return `${base} · ${counts}`;
    }
    if (n === "shell" || n === "bash") {
        const exit = extractExitCode(result);
        if (exit !== undefined && exit !== 0) {
            return `${base} · exit ${exit}`;
        }
    }
    return base;
}
function toolPhase(status, name, result) {
    if (status === "error")
        return "error";
    if (status === "completed") {
        const n = (name ?? "").toLowerCase();
        if (n === "shell" || n === "bash") {
            const exit = extractExitCode(result);
            if (exit !== undefined && exit !== 0)
                return "error";
        }
        return "complete";
    }
    return "start";
}
export function bridgeEventFromSdk(msg) {
    const t = msg.type ?? "";
    if (t === "assistant") {
        const text = textFromBlocks(msg);
        return text ? { event: "assistant", text } : null;
    }
    if (t === "thinking") {
        const text = typeof msg.text === "string" ? msg.text : textFromBlocks(msg);
        return text ? { event: "thinking", text } : null;
    }
    if (t === "tool_call") {
        const phase = toolPhase(msg.status, msg.name, msg.result);
        let summary = toolSummaryFromArgs(msg.name, msg.args);
        if (phase === "complete" || phase === "error") {
            summary = enrichCompleteSummary(summary, msg.name, msg.result);
        }
        return {
            event: "tool_call",
            phase,
            name: msg.name,
            toolCallId: msg.call_id ?? msg.toolCallId,
            summary,
        };
    }
    if (t === "usage" && msg.usage) {
        return { event: "usage", usage: msg.usage };
    }
    return null;
}
export function forwardSdkMessage(msg) {
    const ev = bridgeEventFromSdk(msg);
    if (ev)
        emit(ev);
}
export function usageRecord(usage) {
    return usage;
}
