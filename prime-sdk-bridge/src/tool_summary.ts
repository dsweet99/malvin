/** ACP-parity tool subject lines for Prime bridge events (VISION `t|` parity). */

const SUMMARY_MAX = 60;

export function flatten(s: string): string {
  return s.replace(/\s+/g, " ").trim();
}

/** Middle-ellipsis shorten — mirrors Rust `tool_summary::shorten_middle`. */
export function shortenMiddle(s: string, max = SUMMARY_MAX): string {
  if (s.length <= max) return s;
  const keep = max - 3;
  const left = Math.ceil(keep / 2);
  const right = Math.floor(keep / 2);
  return `${s.slice(0, left)}...${s.slice(s.length - right)}`;
}

function asRecord(v: unknown): Record<string, unknown> | null {
  if (v && typeof v === "object" && !Array.isArray(v)) {
    return v as Record<string, unknown>;
  }
  return null;
}

/**
 * Subject line from tool name + args (no duration).
 * IPython `%%bash` cells map to `Run <cmd>` so Rust tees the shell-style ✓ line.
 */
export function toolSummaryFromArgs(name: string | undefined, args: unknown): string {
  const label = (name ?? "tool").trim() || "tool";
  const a = asRecord(args) ?? {};
  const n = label.toLowerCase();

  if (n === "ipython") {
    const code = typeof a.code === "string" ? a.code : "";
    if (!code.trim()) return label;
    const flat = flatten(code);
    if (/^%%bash\b/i.test(flat)) {
      const cmd = flatten(flat.replace(/^%%bash\b/i, ""));
      return cmd ? `Run ${shortenMiddle(cmd)}` : "Run bash";
    }
    // IPython shell bang: `!ls -la`
    if (flat.startsWith("!")) {
      const cmd = flatten(flat.slice(1));
      return cmd ? `Run ${shortenMiddle(cmd)}` : "Run";
    }
    return shortenMiddle(`ipython: ${flat}`);
  }

  if (n === "shell" || n === "bash") {
    const cmd = typeof a.command === "string" ? a.command : "";
    return cmd ? `Run ${shortenMiddle(flatten(cmd))}` : "Run";
  }

  return label;
}

/** Map prime-agent tool_execution_end → Cursor-style phase. */
export function toolPhaseFromEnd(isError: boolean): "complete" | "error" {
  return isError ? "error" : "complete";
}

function asResultRecord(result: unknown): Record<string, unknown> | null {
  const r = asRecord(result);
  if (!r) return null;
  return asRecord(r.value) ?? r;
}

function extractExitCode(result: unknown): number | undefined {
  const v = asResultRecord(result);
  if (!v) return undefined;
  for (const key of ["exitCode", "exit_code", "code"]) {
    const n = v[key];
    if (typeof n === "number" && Number.isFinite(n)) return n;
  }
  const details = asRecord(v.details);
  if (details) {
    for (const key of ["exitCode", "exit_code", "code"]) {
      const n = details[key];
      if (typeof n === "number" && Number.isFinite(n)) return n;
    }
  }
  return undefined;
}

/**
 * Enrich complete/error tool summaries like cursor-sdk-bridge `enrichCompleteSummary`.
 * Keeps the start-phase subject and appends exit codes when present.
 */
export function enrichCompleteSummary(
  base: string,
  name: string | undefined,
  result: unknown,
  isError: boolean,
): string {
  const n = (name ?? "").toLowerCase();
  const exit = extractExitCode(result);
  if (
    (n === "ipython" || n === "shell" || n === "bash" || base.startsWith("Run ")) &&
    exit !== undefined &&
    (isError || exit !== 0)
  ) {
    if (base.includes(" · exit ")) return base;
    return `${base} · exit ${exit}`;
  }
  return base;
}
