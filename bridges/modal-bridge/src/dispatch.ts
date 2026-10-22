import {
  type Session,
  createSandbox,
  download,
  ensureImage,
  exec,
  listTagged,
  terminate,
  terminateIds,
  upload,
} from "./ops.js";
import { type Emit, type Json, type Request, errorText } from "./protocol.js";

export async function dispatch(s: Session, req: Request, emit: Emit): Promise<Json> {
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

export async function handle(s: Session, req: Request, write: (v: Json) => void): Promise<void> {
  const emit: Emit = (event, data) => write({ id: req.id, event, data });
  try {
    const result = await dispatch(s, req, emit);
    write({ id: req.id, ok: true, ...result });
  } catch (e) {
    write({ id: req.id, ok: false, error: errorText(e) });
  }
}
