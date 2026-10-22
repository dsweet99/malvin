export type Json = Record<string, unknown>;

export interface Request extends Json {
  id: number;
  op: string;
}

export type Emit = (event: "stdout" | "stderr" | "log", data: string) => void;

export function parseRequest(line: string): Request {
  const value = JSON.parse(line) as unknown;
  if (typeof value !== "object" || value === null) {
    throw new Error("request must be a JSON object");
  }
  const req = value as Json;
  if (typeof req.id !== "number" || typeof req.op !== "string") {
    throw new Error("request needs numeric `id` and string `op`");
  }
  return req as Request;
}

export function encodeLine(value: Json): string {
  return `${JSON.stringify(value)}\n`;
}

export function str(req: Json, key: string): string {
  const v = req[key];
  if (typeof v !== "string" || v === "") {
    throw new Error(`missing string field \`${key}\``);
  }
  return v;
}

export function optStr(req: Json, key: string): string | undefined {
  const v = req[key];
  return typeof v === "string" && v !== "" ? v : undefined;
}

export function num(req: Json, key: string, fallback: number): number {
  const v = req[key];
  return typeof v === "number" && Number.isFinite(v) ? v : fallback;
}

export function strMap(req: Json, key: string): Record<string, string> {
  const v = req[key];
  const out: Record<string, string> = {};
  if (typeof v !== "object" || v === null) {
    return out;
  }
  for (const [k, val] of Object.entries(v as Json)) {
    if (typeof val === "string") {
      out[k] = val;
    }
  }
  return out;
}

export function strList(req: Json, key: string): string[] {
  const v = req[key];
  return Array.isArray(v) ? v.filter((x): x is string => typeof x === "string") : [];
}

export function errorText(e: unknown): string {
  if (e instanceof Error) {
    return `${e.name}: ${e.message}`;
  }
  return String(e);
}
