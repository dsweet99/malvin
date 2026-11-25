# malvin

malvin is a non-interactive research and coding agent. You give it a request,
and it works in the current directory until the request is done. Every run
writes logs under `~/.malvinconf/logs`.

## Install

malvin needs Rust 1.95 or newer.

```bash
cargo install malvin
```

## Set up a backend

malvin runs models through four backends. A model id starts with the
backend's prefix, for example `cursor:auto` or `claude:sonnet`. Set up at least
one backend. The default model is `cursor:auto`.

| Backend | Install                                     | Credentials                                |
|---------|---------------------------------------------|--------------------------------------------|
| cursor  | Node.js ≥ 22.13 with npm                    | `CURSOR_API_KEY`                           |
| claude  | Node.js ≥ 22.13 with npm                    | `ANTHROPIC_API_KEY`, or run `claude login` |
| codex   | the `codex` CLI                             | `codex login`, or `OPENAI_API_KEY`         |
| pi      | Node.js ≥ 22.19 with npm, plus the Pi agent | the key for your Pi provider               |

**cursor.** The first time a `cursor:` model runs, malvin installs the Cursor
SDK under `~/.malvinconf/sdk-bridges/`. You do not need the `cursor-agent` CLI.

**claude.** The first time a `claude:` model runs, malvin installs the Claude
Agent SDK under `~/.malvinconf/sdk-bridges/`. Instead of `ANTHROPIC_API_KEY`,
you may set `CLAUDE_CODE_OAUTH_TOKEN`, `ANTHROPIC_OAUTH_TOKEN`, or
`ANTHROPIC_AUTH_TOKEN`.

**codex.** malvin does not bundle Codex. Install the `codex` CLI so that it is
on your `PATH` (or set `MALVIN_CODEX` to its path), then run `codex login`.

**pi.** malvin does not install Pi. Install it once:

```bash
npm install --prefix ~/.malvinconf/sdk-bridges @earendil-works/pi-coding-agent
```

Pi model ids name a provider and a model, as in `pi:openrouter/<model>`. Set the
provider's key (here, `OPENROUTER_API_KEY`) or log in with Pi itself.

To check your setup, list the models malvin can use:

```bash
malvin admin models
```

## Use

```bash
malvin "Where (on Earth) am I, precisely? Be creative."
```

Choose a model with `--model`, for example `--model=claude:sonnet`.

## Configure an alias

An alias is a short name for a model id. Add aliases to
`~/.malvinconf/config.toml`:

```toml
[aliases.models]
opus = "claude:opus[effort=medium]"

[agent]
model = ["opus"]
```

Now `malvin --model=opus "..."` uses that model, and so does a plain `malvin`
run, because `[agent].model` lists it first.

## More

- `malvin --help` lists the options.
- `malvin --doc` is the full reference, including local models and running on
  [Modal](https://modal.com) with `--remote=modal:sandbox`.
