export function parseRequest(line) {
    const value = JSON.parse(line);
    if (typeof value !== "object" || value === null) {
        throw new Error("request must be a JSON object");
    }
    const req = value;
    if (typeof req.id !== "number" || typeof req.op !== "string") {
        throw new Error("request needs numeric `id` and string `op`");
    }
    return req;
}
export function encodeLine(value) {
    return `${JSON.stringify(value)}\n`;
}
export function str(req, key) {
    const v = req[key];
    if (typeof v !== "string" || v === "") {
        throw new Error(`missing string field \`${key}\``);
    }
    return v;
}
export function optStr(req, key) {
    const v = req[key];
    return typeof v === "string" && v !== "" ? v : undefined;
}
export function num(req, key, fallback) {
    const v = req[key];
    return typeof v === "number" && Number.isFinite(v) ? v : fallback;
}
export function strMap(req, key) {
    const v = req[key];
    const out = {};
    if (typeof v !== "object" || v === null) {
        return out;
    }
    for (const [k, val] of Object.entries(v)) {
        if (typeof val === "string") {
            out[k] = val;
        }
    }
    return out;
}
export function strList(req, key) {
    const v = req[key];
    return Array.isArray(v) ? v.filter((x) => typeof x === "string") : [];
}
export function errorText(e) {
    if (e instanceof Error) {
        return `${e.name}: ${e.message}`;
    }
    return String(e);
}
