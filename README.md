# malvin


## Installation

Requires [Node.js](https://nodejs.org/) ≥ 22.13 (with `npm` on `PATH`). During
`cargo install` / `cargo build`, malvin's build script installs the Cursor SDK
(`@cursor/sdk`) under `~/.malvin_home/sdk-bridges/`
(skipped when the repo already has a built in-tree bridge).

```bash
cargo install malvin
```

Without Node/npm the Rust build fails unless you set `MALVIN_SKIP_SDK_BRIDGES=1`
(agent backends will not work).

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
and expect good results. For a simple one-shot turn:
```bash
malvin --do "Hello"
```

You can also pass a request file instead of a string:
```bash
malvin code_review.md
```
That works well in CI or cron. For no stdout at all, use `-b`:
```bash
malvin -b overnight_logs_alerter.md
```
For example, `overnight_logs_alerter.md` might tell malvin to scan prod logs and report oddities via Slack. Malvin *always* writes run logs under `~/.malvin_home/logs` (useful for process improvement and as later context).

Flag reference: `malvin --help`. Behavioral contracts: `malvin --doc` and `malvin <COMMAND> --doc`.

## Notes

`malvin` allows all tool calls by default.

## Speed

`malvin` likes to run linters and unit tests. It does its best to only run what's necessary, but these tools can help speed things up:

- [Python] [pytest-testmon](https://www.testmon.org) Runs only unit tests affected by code changes
- [Rust] [cargo-nextest](https://nexte.st) Faster than `cargo test`
- [Rust] [cargo-difftests](https://github.com/dnbln/cargo-difftests) Re-runs only tests whose executed code changed (LLVM coverage indexes)


# EXPERIMENTAL - USE AT YOUR OWN RISK

- pi: models (requires an externally installed `pi` binary; see `design.md`)
- Codex: models (requires an externally installed `codex` binary; local stdio app-server)
