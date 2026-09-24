# malvin --do

One **single-turn** agent session: no gate loop, no experiment log, no review fan-out.

## Summary

| | |
|---|---|
| Input | `<REQUEST>` text or existing `.md` path |
| Output | Default: plain stdout with only text between `__MALVIN_DM_START__` / `__MALVIN_DM_END__`. With `--verbose`: same agent log classes as the default workflow (thought tokens, narrative tee, full outgoing prompts). |
| Log | `do.log` under `~/.malvin_home/logs/<hash>/<run>/` |
| Requires | No `.malvin/gates` at startup |

## Intention

Answer a question, perform a one-off task, or continue informal work without a gate-loop pipeline. Suitable for terminals and pipes.

## Usage

```text
malvin --do [OPTION]... REQUEST
```

Each `--do` applies only to the single `REQUEST` that follows it (options may appear between `--do` and that request). Additional `REQUEST` args without their own `--do` use the default router. If `--do` is present with no following request (and `--doc` is not set), malvin prints short usage on stdout and exits 0.

## Arguments

### `REQUEST`

Required to run a do-mode turn. One shell argument (quote any argument that contains spaces, e.g. `malvin --do "fix the typo"`). Literal text, or an existing `.md` file path (same rules as bare `malvin REQUEST`). Repeat `--do` before each request that should be one-shot:

```text
malvin --do "Hello" "Research the topic"
malvin "router task" --do "one-shot" --do "another one-shot"
```

In the first example, only `Hello` is do-mode; `Research the topic` uses the router.

| Form | Work directory | Stored as |
|------|----------------|-----------|
| Literal | `.` (cwd) | `plan_<random>.md` in run dir |
| `path/to/file.md` | Parent of file | `plan_<random>.md` |

## Global options

See `malvin --doc`. Notable for `--do`:

| Flag | Effect on `--do` |
|------|----------------|
| `--quiet` / `-q` | Not needed without `--verbose`: `--do` is already DM-body-only |
| `--verbose` / `-v` | Same stdout log classes as the default workflow (thoughts, narrative tee, full prompt bodies); also full bodies in `prompts.log` |
| `--iml` | the Infinite Meta-Loop: after all REQUEST args finish once, repeat the full sequence forever |

## Prompt workflow

`start_coder_session` sends **one** host prompt: `header.md` plus `do_header.md` plus the user request (labeled `do_header.md`). There is no follow-up work turn and no separate agent reply to the header alone.

| Piece | Role |
|-------|------|
| `header.md` + `do_header.md` | Standard Malvin context and do-mode persona / DM rules |
| User request | The operator request, co-sent in the same host prompt |

No implement, review, concerns, learn, or summary phases.

## Session behavior

- Ensures `~/.malvin_home/config.toml` exists with defaults (same as `tidy`).
- Backs up `.gitignore`, `.malvin/gates`, and `.malvin/config.toml`; restores after the session.
- Checks `result.md` for `ABORT:` after the session.

## Related commands

| Command | When |
|---------|------|
| `malvin --do Hello` | One-turn agent connectivity smoke check |

## Examples

```text
malvin --do Hello
malvin --do "List failing tests and suggest fixes"
malvin --do notes/task.md
malvin --verbose --do "Show the full agent stream"
malvin --do "Hello" "Research the topic"
```
