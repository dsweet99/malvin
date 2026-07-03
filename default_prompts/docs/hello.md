# malvin hello

One **single-turn** Cursor ACP connectivity probe: sends the fixed prompt `Hello` and streams the agent reply to stdout.

## Summary

| | |
|---|---|
| Input | Fixed prompt `Hello` (no `<REQUEST>` argument) |
| Output | Plain stdout (same tee rules as `malvin do`) |
| Log | `do.log` under `~/.malvin_home/logs/<hash>/<run>/` |

## Intention

Verify that `cursor-agent` / `agent acp` is installed, authenticated, and reachable from this environment. Use it for local smoke checks before running longer agent workflows.

## Usage

```text
malvin hello [OPTIONS]
```

## Options

### `--thoughts`

Stream agent thought tokens to stdout in addition to normal output.

## Global options

See `malvin --doc`. Same tee and auth behavior as `malvin do`.

## Related commands

| Command | When |
|---------|------|
| `malvin do` | General one-shot agent turn with a custom request |

## Examples

```text
malvin hello
malvin hello --thoughts
```
