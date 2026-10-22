# malvin


## Installation

```bash
cargo install malvin
```

The build needs neither Node nor network access beyond crates.io.

`cursor:` models (including the default, `cursor:auto`) need
[Node.js](https://nodejs.org/) ≥ 22.13 with `npm` at run time. The first time a
`cursor:` model runs, malvin installs the Cursor SDK (`@cursor/sdk`) under
`~/.malvinconf/sdk-bridges/` with `npm ci`.

`claude:` models need Node.js ≥ 22.13 with npm. The first time a `claude:` model
runs, malvin installs the Claude Agent SDK (`@anthropic-ai/claude-agent-sdk`)
under `~/.malvinconf/sdk-bridges/`. Set `ANTHROPIC_API_KEY`,
`ANTHROPIC_OAUTH_TOKEN`, `ANTHROPIC_AUTH_TOKEN`, or `CLAUDE_CODE_OAUTH_TOKEN`
before running one.

`pi:` models also need Node (≥ 22.19 for Pi 1.x), plus the npm Pi agent
(`npm install @earendil-works/pi-coding-agent` in `~/.malvinconf/sdk-bridges/`, or
set `MALVIN_PI`). `codex:` models do not need Node.

To run on [Modal](https://modal.com) instead of locally, add `--remote=modal:sandbox` (for example `malvin --remote=modal:sandbox "Make the tests pass"`), or `--remote=modal:sandbox[gpu=...,ncpu=...,mem=...,timeout=...]` to choose a GPU, CPU count, memory, and time limit (defaults: no GPU, 1 CPU, 8 GiB, 30 minutes); `malvin admin remotes` lists the suboptions and their values. It needs Node ≥ 22.13 and Modal credentials (`modal setup`, or `MODAL_TOKEN_ID` and `MODAL_TOKEN_SECRET`). See "Running on Modal" in `malvin --doc`.

## Usage

```text
malvin [OPTION]... [REQUEST]
   or: malvin [OPTION]... <COMMAND>
```

Most of the time, just ask for what you want:
```bash
malvin "What time is it?"
```

By default malvin prints a full agent stream. For the final answer only:
```bash
malvin -q "Where (geographically) am I?"
```

By default, malvin runs an investigation. The larger or more complex the task, the more helpful this is. You can ask difficult questions or request complex changes somewhat tersely:
```bash
malvin "Speed up my_function.py by at least 3x."
```
and expect good results.

You can also pass a request file instead of a string:
```bash
malvin code_review.md
```
That works well in CI or cron. For quieter stdout on the default router, use `-q` (DM bodies only). For no process stdout at all, redirect:
```bash
malvin overnight_logs_alerter.md >/dev/null
```
For example, `overnight_logs_alerter.md` might tell malvin to scan prod logs and report oddities via Slack. Malvin *always* writes run logs under `~/.malvinconf/logs` (useful for process improvement and as later context).

Flag reference: `malvin --help`. Behavioral contracts: `malvin --doc` and `malvin <COMMAND> --doc`.

## Notes

`malvin` allows all tool calls by default.

