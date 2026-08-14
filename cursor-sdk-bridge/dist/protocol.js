
export function emit(ev) {
    process.stdout.write(`${JSON.stringify(ev)}\n`);
}
export function parseRequest(line) {
    const raw = JSON.parse(line);
    if (!raw || typeof raw !== "object" || !("op" in raw)) {
        throw new Error("bridge request missing op");
    }
    return raw;
}
