# malvin


## Installation

```bash
cargo install malvin
```

The build needs neither Node nor network access beyond crates.io.

`cursor:` models (including the default, `cursor:auto`) need
[Node.js](https://nodejs.org/) ≥ 22.13 with `npm` at run time. The first time a
`cursor:` model runs, malvin installs the Cursor SDK (`@cursor/sdk`) under
`~/.malvinconf/sdk-bridges/` with `npm ci`. Set `CURSOR_API_KEY`.

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

By default, malvin prints a full agent stream. Use `-q` (or `--quiet`) to print
only direct messages (DMs), plus the closing `TIMING` and `COST` lines. A DM is
text the agent places between `__MALVIN_DM_START__` and `__MALVIN_DM_END__`
fences; the fence markers are omitted from stdout. Ask for a DM in your request:
```bash
malvin -q "Where (geographically) am I? Send the answer in a DM."
```

Use `--no-emit` to disable the configured `[dm].on_emit` command for one
invocation. Direct messages still appear in the normal output and logs.

By default, malvin runs an investigation. The larger or more complex the task, the more helpful this is. You can ask difficult questions or request complex changes somewhat tersely:
```bash
malvin "Speed up my_function.py by at least 3x."
```
and expect good results.

You can also pass a request file instead of a string:
```bash
malvin code_review.md
```
That works well in CI or cron. Use `-qq` (or `--very-quiet`) to suppress all
process stdout, including DMs and the closing `TIMING` and `COST` lines:
```bash
malvin -qq overnight_logs_alerter.md
```
For example, `overnight_logs_alerter.md` might tell malvin to scan prod logs and report oddities via Slack. Malvin *always* writes run logs under `~/.malvinconf/logs` (useful for process improvement and as later context).

Flag reference: `malvin --help`. Behavioral contracts: `malvin --doc` and `malvin <COMMAND> --doc`.

## Notes

`malvin` allows all tool calls by default.


Model selection uses an ordered list in `~/.malvinconf/config.toml`:

```toml
[agent]
model = ["cursor:auto", "claude:sonnet", "codex:gpt-5.6"]
# Optional fast, cheap models for 30-character herdr task and status summaries:
# model_lite = ["cursor:auto"]
```

Each task starts with the first model. If it fails for lack of funds or credits,
including a usage limit or a plan-upgrade error, malvin tries the next model.
Other failures retain their existing error handling. If every model fails for
one of those reasons, the last error is reported.
Existing single-string model settings remain supported.

The optional `agent.model_lite` list accepts the same model IDs, aliases, and quota
fallback order. When it is set, malvin summarizes the user request in the background
into at most 30 characters. Requests with fewer than ten words use their first
30 characters without calling a model, even when `model_lite` is not configured.
A request that already fits is also used as written. Other requests are summarized
by `model_lite`. Model output that exceeds the limit is
shortened at a word boundary when possible. That summary appears after the run ID on `h|`
lines and is reported as the herdr `task` token. Five minutes after the run starts,
and every ten minutes after that, `model_lite` summarizes the recent run output into
at most 30 characters and reports it as the herdr `status` token. Until the task
summary is ready, the heartbeat adds no text. A longer request without configuration or a failed
summary leaves the heartbeat unchanged and leaves the tokens unset. This work never
delays the main run and is cancelled when that run ends.

To display those tokens in Herdr's expanded desktop Agent sidebar, add rows to
Herdr's configuration on the machine running the viewing client:

```toml
[ui.sidebar.agents]
rows = [
    ["state_icon", "agent", "state_text"],
    ["$task"],
    ["$status"],
]
```

Use Herdr's UI **reload config** action after editing the file. With an SSH
connection, the sidebar uses the viewing client's local configuration; changing
the remote server's configuration does not configure that sidebar. Herdr's
collapsed and mobile views keep their compact layouts and do not use these rows.
See [Herdr's configuration guide](https://herdr.dev/docs/configuration/).

If only `malvin` appears, inspect `herdr pane get "$HERDR_PANE_ID"` from the
terminal running malvin. Its `tokens` map shows whether Herdr received `task` and
`status`, independently of the sidebar layout. A missing `status` is expected
during the first five minutes. Summary errors are recorded in the run directory's
`request_summary.log`; delivery errors are recorded in `herdr.log`.

`--model=claude:sonnet` uses only that model. A comma-separated override, such as
`--model=claude:sonnet,cursor:auto`, replaces the entire configured list. Commas
inside bracket parameters remain part of the model, for example
`--model='codex:gpt-5.6[thinking=high,service=priority],cursor:auto'`.
