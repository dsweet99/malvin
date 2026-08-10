/** Shared emit / parse for the Prime JSONL bridge. */
export function emit(obj) {
    process.stdout.write(`${JSON.stringify(obj)}\n`);
}
export function parseRequest(line) {
    const obj = JSON.parse(line);
    if (!obj || typeof obj !== "object" || !("op" in obj)) {
        throw new Error("bridge request missing op");
    }
    return obj;
}
export function exitCodeForSignal(signal) {
    if (signal === "SIGINT")
        return 130;
    if (signal === "SIGTERM")
        return 143;
    return 1;
}
