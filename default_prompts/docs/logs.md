# malvin logs

Inspect and manually trigger run-log garbage collection for a workspace bucket under `~/.malvin_home/logs/<hash>/`.

## Summary

| | |
|---|---|
| Agent session | None |
| kiss / `.malvin/` | Not required |
| Run directory | Not created |
| Config | Ensures `~/.malvin_home/config.toml` exists (merges missing keys) |

## Intention

Report retention state and prune old run directories without starting an agent session. Use `--dry-run` to preview deletions.

## Usage

```text
malvin logs <SUBCOMMAND> [OPTIONS]
```

### Subcommands

| Subcommand | Purpose |
|------------|---------|
| `status` | Print bucket path, run count, total bytes, oldest/newest run, effective `[logs]` config, and per-policy prune triggers |
| `gc` | Run the same prune logic as opportunistic GC on run creation |

### `logs status` flags

| Flag | Default | Purpose |
|------|---------|---------|
| `--work-dir PATH` | current directory | Workspace whose log bucket to inspect |

### `logs gc` flags

| Flag | Default | Purpose |
|------|---------|---------|
| `--work-dir PATH` | current directory | Workspace whose log bucket to prune |
| `--dry-run` | off | Report what would be deleted without removing directories |
| `--all-buckets` | off | Also remove empty hash buckets under `~/.malvin_home/logs/` |

## Retention policies

Pruning triggers when **any** of these apply (delete oldest runs first until all satisfied):

1. Total bytes exceed `max_bytes` (empty string disables byte cap)
2. Run count exceeds `max_count` (`0` = unlimited)
3. Oldest run is older than `max_age_days` (`0` = disabled)

Defaults from `~/.malvin_home/config.toml`:

```toml
[logs]
max_count = 1000
max_age_days = 90
max_bytes = "2GiB"
```

## Examples

```text
malvin logs status
malvin logs status --work-dir /path/to/repo
malvin logs gc --dry-run
malvin logs gc
malvin logs gc --all-buckets --dry-run
```

## Notes

- `malvin init` still skips GC; `malvin do` and other agent commands run GC before creating a new run dir.
- After upgrading to a build with `max_count`, the next GC-enabled command or `malvin logs gc` may delete excess oldest runs once.
