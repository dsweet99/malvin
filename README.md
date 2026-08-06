# malvin


## Installation

```bash
cargo install malvin
```

## Usage

Most of the time, just ask for what you want like this:
```bash
malvin "What time is it?"
```

malvin will show lots of logs by default. If you just want to see malvin's answer at the end use
```bash
malvin -q "Where (geographically) am I?"
```

Be default, malvin does an "investigation". The larger or more complex the task is, the more helpful this approach is. It permits you to ask difficult questions or ask for complex changes somewhat tersely:
```bash
malvin "Speed up my_function.py by at least 3x."
```
and expect good results. However, if you want to do something very simple and want a quick response, use
```bash
malvin --do "Hello"
```

You can also provide malvin with a request file instead of a string:
```bash
malvin code_review.md
```
This can be great for use in CI or cron. In fact, if you want malvin to work totally in the background -- with no stdout -- you can use `-b`:
```bash
malvin -b overnight_logs_alerter.md
```
Maybe `overnight_logs_alerter.md` tells malvin to scan your prod logs and report weirdness to you via Slack. Note that malvin *always* logs to `~/.malvin_home/logs`. Those logs can be used by you for process improvement. They are always used by malvin as context.

## Notes

`malvin` allows all tool calls by default.

## Speed

`malvin` likes to run linters and unit tests. It does its best to only run what's necessary, but these tools can help speed things up:

- [Python] [pytest-testmon](https://www.testmon.org) Runs only unit tests affected by code changes
- [Rust] [cargo-nextest](https://nexte.st) Faster than `cargo test`
- [Rust] [cargo-difftests](https://github.com/dnbln/cargo-difftests) Re-runs only tests whose executed code changed (LLVM coverage indexes)
- [Rust] [sccache](https://github.com/mozilla/sccache) Speeds up builds by caching build artifacts


# EXPERIMENTAL - USE AT YOUR OWN RISK

- prime: models
- mini:openrouter/… models
- mini:local/… models
