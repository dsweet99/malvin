export function modelSelectionFromRaw(raw) {
    const trimmed = raw.trim();
    const open = trimmed.indexOf("[");
    if (open < 0) {
        return { id: trimmed || "auto" };
    }
    if (!trimmed.endsWith("]")) {
        throw new Error(`cursor model bracket overrides must end with ']' (got '${raw}')`);
    }
    const id = trimmed.slice(0, open).trim();
    if (!id) {
        throw new Error(`cursor model bracket overrides require a base id before '[' (got '${raw}')`);
    }
    const inner = trimmed.slice(open + 1, -1).trim();
    if (!inner) {
        return { id };
    }
    if (inner.includes("[") || inner.includes("]")) {
        throw new Error(`cursor model bracket overrides must not nest brackets (got '${raw}')`);
    }
    const params = inner.split(",").map((part) => {
        const p = part.trim();
        const eq = p.indexOf("=");
        if (eq <= 0 || eq === p.length - 1) {
            throw new Error(`cursor model bracket overrides must be key=value (got '${p}')`);
        }
        return {
            id: p.slice(0, eq).trim(),
            value: p.slice(eq + 1).trim(),
        };
    });
    for (const p of params) {
        if (!p.id || !p.value) {
            throw new Error(`cursor model bracket overrides must be key=value (got empty key or value)`);
        }
    }
    return { id, params };
}
