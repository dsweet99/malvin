# malvin hello

One **single-turn** Cursor ACP connectivity probe: sends the fixed prompt `Hello` and streams the agent reply to stdout.

## Summary

| | |
|---|---|
| Input | Fixed prompt `Hello` (no `<REQUEST>` argument) |
| Output | Plain stdout (same tee rules as `malvin do`) |
| Log | `do.log` under `~/.malvin_home/logs/<hash>/<run>/` |

## Intention

Verify that `cursor-agent` / `agent acp` is installed, authenticated, and reachable from this environment. Used by DeepSWE `ops/deepswe_run.py hello TASK` (Modal sandbox: same auth injection and CIDR allowlist path as `solve`, without task plan or Harbor grading) and local smoke checks (`--host` or `malvin hello`).

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
| `python ops/deepswe_run.py hello TASK` | Full Modal sandbox smoke test (auth + CIDR allowlist); no solving or grading |

## Examples

```text
malvin hello
malvin hello --thoughts
```
