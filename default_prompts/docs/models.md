# malvin models

List model ids for malvin runs. No malvin prompts and no `./.malvin/logs` run directory.

## Summary

| | |
|---|---|
| Agent session | None |
| kiss / `.malvin/` | Not required |
| Output | Model list + default model footer (see below) |

## Intention

Discover valid `--model` values for other malvin commands. Default mode uses the Cursor agent CLI; `--mini` lists OpenRouter chat models for the in-process mini backend.

## Usage

```text
malvin models [OPTIONS]
```

### Subcommand flags

| Flag | Default | Purpose |
|------|---------|---------|
| `--mini` | off | Fetch live OpenRouter models instead of running `agent models` |

## Global options

See `malvin --doc`. Only `--no-color` materially affects output formatting. Global `--model` is parsed but **not used** by this subcommand. Global `--mini` does **not** trigger OpenRouter listing; only `malvin models --mini` does.

## Behavior (default — Cursor agent CLI)

1. Resolve `agent` or `cursor-agent` on `PATH`.
2. Run `<binary> models`.
3. Strip ANSI escapes and trailing “Tip:” banner lines.
4. Parse bullet-list model names when possible; otherwise print cleaned stdout verbatim.
5. Print blank line and: `Default model: auto`.

## Behavior (`--mini` — OpenRouter)

1. `GET {OPENROUTER_BASE_URL}/models?output_modalities=text&sort=most-popular` (default base URL: `https://openrouter.ai/api/v1`).
2. `OPENROUTER_API_KEY` is optional for this public catalog; when set, the same Bearer / referer headers as mini completions are sent.
3. Print one tab-separated row per model: `slug\tname`.
4. Print blank line and: `Default mini model: anthropic/claude-sonnet-4`.
5. Network or API failures exit non-zero with an error message (no static fallback list).

Environment variables (mini listing only):

| Variable | Required | Purpose |
|----------|----------|---------|
| `OPENROUTER_API_KEY` | no | Optional Bearer token |
| `OPENROUTER_HTTP_REFERER` | no | OpenRouter attribution header |
| `OPENROUTER_BASE_URL` | no | Override API base (testing) |
| `OPENROUTER_REQUEST_TIMEOUT` | no | HTTP timeout in seconds (default 120) |

## Examples

```text
malvin models
malvin models --mini
malvin --no-color models
malvin --model sonnet-4 code plan.md    # --model applies to code, not models
```
