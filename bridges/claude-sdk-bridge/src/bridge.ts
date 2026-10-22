import "./parent_death.js";
import * as readline from "node:readline";
import { query, type Options, type SDKUserMessage } from "@anthropic-ai/claude-agent-sdk";
import { emit, parseRequest, type BridgeRequest, type CreateOp, type ResumeOp } from "./protocol.js";
import { mapMessage } from "./message_map.js";

class PromptQueue implements AsyncIterable<SDKUserMessage> {
  #entries: string[] = [];
  #waiter: (() => void) | undefined;
  #closed = false;

  push(prompt: string): void {
    if (this.#closed) throw new Error("bridge is closed");
    this.#entries.push(prompt);
    this.#waiter?.();
    this.#waiter = undefined;
  }

  close(): void {
    this.#closed = true;
    this.#waiter?.();
    this.#waiter = undefined;
  }

  async *[Symbol.asyncIterator](): AsyncIterator<SDKUserMessage> {
    for (;;) {
      const prompt = this.#entries.shift();
      if (prompt) {
        yield { type: "user", message: { role: "user", content: prompt }, parent_tool_use_id: null };
        continue;
      }
      if (this.#closed) return;
      await new Promise<void>((resolve) => { this.#waiter = resolve; });
    }
  }
}

let queue: PromptQueue | undefined;
let activeQuery: ReturnType<typeof query> | undefined;
let closing = false;

function optionsFromBoot(request: CreateOp | ResumeOp): Options {
  const options: Options = {
    cwd: request.cwd,
    model: request.model,
    permissionMode: "bypassPermissions",
    allowDangerouslySkipPermissions: true,
  };
  if (request.op === "resume") options.resume = request.agentId;
  return options;
}

function start(request: CreateOp | ResumeOp): void {
  if (activeQuery) {
    emit({ event: "fatal", message: "agent already created", retryable: false });
    return;
  }
  queue = new PromptQueue();
  activeQuery = query({ prompt: queue, options: optionsFromBoot(request) });
  void consume(activeQuery);
  emit({ event: "ok" });
}

async function consume(session: ReturnType<typeof query>): Promise<void> {
  try {
    for await (const message of session) {
      for (const event of mapMessage(message)) emit(event);
    }
  } catch (error) {
    if (!closing) emit({ event: "fatal", message: errorMessage(error), retryable: false });
  }
}

function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}

function dispatch(request: BridgeRequest): void {
  switch (request.op) {
    case "create":
    case "resume":
      start(request);
      return;
    case "send":
      if (!queue) emit({ event: "fatal", message: "create before send", retryable: false });
      else queue.push(request.prompt);
      return;
    case "cancel":
      void activeQuery?.interrupt();
      return;
    case "close":
      closing = true;
      queue?.close();
      activeQuery?.close();
      return;
  }
}

const lines = readline.createInterface({ input: process.stdin, crlfDelay: Infinity });
lines.on("line", (line) => {
  try { dispatch(parseRequest(line)); }
  catch (error) { emit({ event: "fatal", message: errorMessage(error), retryable: false }); }
});
lines.on("close", () => dispatch({ op: "close" }));
