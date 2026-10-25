const SUMMARY_MAX = 60;

function flatten(s: string): string {
  return s.replace(/\s+/g, " ").trim();
}

export function shortenMiddle(s: string, max = SUMMARY_MAX): string {
  if (s.length <= max) return s;
  const keep = max - 3;
  const left = Math.ceil(keep / 2);
  const right = Math.floor(keep / 2);
  return `${s.slice(0, left)}...${s.slice(s.length - right)}`;
}

function strField(obj: Record<string, unknown>, ...keys: string[]): string | undefined {
  for (const k of keys) {
    const v = obj[k];
    if (typeof v === "string" && v.trim()) return v;
  }
  return undefined;
}

function labeled(label: string, value: string | undefined): string {
  return value ? `${label} ${shortenMiddle(flatten(value))}` : label;
}

/** Human-readable one-line summary of a Claude tool_use block (name + input). */
export function toolSummary(name: string | undefined, input: unknown): string {
  const label = (name ?? "tool").trim() || "tool";
  const a =
    input && typeof input === "object" && !Array.isArray(input)
      ? (input as Record<string, unknown>)
      : {};
  switch (label.toLowerCase()) {
    case "bash":
      return labeled("Run", strField(a, "command"));
    case "read":
      return labeled("Read", strField(a, "file_path", "path"));
    case "edit":
    case "multiedit":
      return labeled("Edit", strField(a, "file_path", "path"));
    case "write":
      return labeled("Write", strField(a, "file_path", "path"));
    case "notebookedit":
      return labeled("Edit", strField(a, "notebook_path"));
    case "grep": {
      const pattern = strField(a, "pattern");
      const path = strField(a, "path");
      return labeled("Search", pattern && path ? `${pattern} in ${path}` : pattern);
    }
    case "glob":
      return labeled("Glob", strField(a, "pattern"));
    case "websearch":
      return labeled("WebSearch", strField(a, "query"));
    case "webfetch":
      return labeled("WebFetch", strField(a, "url"));
    case "task":
    case "agent":
      return labeled("Task", strField(a, "description", "prompt"));
    default:
      return labeled(
        label,
        strField(a, "command", "file_path", "path", "pattern", "query", "url", "description"),
      );
  }
}
