import { createSandbox, download, ensureImage, exec, listTagged, terminate, terminateIds, upload, } from "./ops.js";
import { errorText } from "./protocol.js";
export async function dispatch(s, req, emit) {
    switch (req.op) {
        case "ensure_image":
            return ensureImage(s, req, emit);
        case "create_sandbox":
            return createSandbox(s, req);
        case "upload":
            return upload(s, req);
        case "exec":
            return exec(s, req, emit);
        case "download":
            return download(s, req);
        case "terminate":
            return terminate(s);
        case "list_tagged":
            return listTagged(s, req);
        case "terminate_ids":
            return terminateIds(s, req);
        default:
            throw new Error(`unknown op \`${req.op}\``);
    }
}
export async function handle(s, req, write) {
    const emit = (event, data) => write({ id: req.id, event, data });
    try {
        const result = await dispatch(s, req, emit);
        write({ id: req.id, ok: true, ...result });
    }
    catch (e) {
        write({ id: req.id, ok: false, error: errorText(e) });
    }
}
