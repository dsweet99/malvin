export const TOOLCHAIN_PROBE = "ldd --version 2>&1 | head -n 1; node --version 2>/dev/null || echo node-missing";

const MIN_NODE: [number, number] = [22, 13];

function parseVersion(text: string): [number, number] | null {
  const m = /(\d+)\.(\d+)/.exec(text);
  return m ? [Number(m[1]), Number(m[2])] : null;
}

function atLeast(have: [number, number], want: [number, number]): boolean {
  return have[0] > want[0] || (have[0] === want[0] && have[1] >= want[1]);
}

/** Returns a sentence naming the missing piece, or null when the image is usable. */
export function toolchainProblem(probeOutput: string, minGlibc: string | undefined, needNode: boolean): string | null {
  const [lddLine = "", nodeLine = ""] = probeOutput.trim().split("\n");
  if (minGlibc) {
    const want = parseVersion(minGlibc);
    const have = parseVersion(lddLine.replace(/^.*\)\s*/, ""));
    if (want && (!have || !atLeast(have, want))) {
      return `The Modal image has glibc ${have ? have.join(".") : "(unknown)"}, but the uploaded malvin binary needs glibc ${minGlibc} or newer.`;
    }
  }
  if (needNode) {
    const have = parseVersion(nodeLine);
    if (!nodeLine.startsWith("v") || !have || !atLeast(have, MIN_NODE)) {
      return `The Modal image needs Node >= ${MIN_NODE.join(".")} for cursor: models, but it has ${nodeLine.trim() || "no node"}.`;
    }
  }
  return null;
}
