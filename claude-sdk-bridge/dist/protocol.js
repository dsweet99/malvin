export function emit(event) {
    process.stdout.write(`${JSON.stringify(event)}\n`);
}
export function parseRequest(line) {
    const request = JSON.parse(line);
    if (!request || typeof request !== "object" || !("op" in request)) {
        throw new Error("bridge request missing op");
    }
    return request;
}
