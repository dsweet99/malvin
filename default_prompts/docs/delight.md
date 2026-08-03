# malvin delight

Author a **user-delighting feature pitch** by composing a request and running the **default router** workflow (same path as bare `malvin REQUEST`). The composed request embeds any user guidance and the output path.

## Summary

| | |
|---|---|
| Input | Optional guidance text or `.md` path |
| Output | Workspace file at `--out-path` (default: `pitch.md`); path is named in the composed router request |
| Loop | Default router: `header` → `kpop_common` → `router_a` → optional `router_b`; outer `--max-loops` sessions |
| Fast path | **None** — always runs the router |
| Exit policy | Router success (agent fulfills the composed pitch request) |
| Requires | No `.malvin/checks` preflight at CLI entry (document workflow, like `explain`) |

## Intention

Generate a fresh, repo-grounded pitch for a feature or improvement that would delight the user — without overwriting an existing pitch file. Typical pipeline: `malvin delight` → bare `malvin REQUEST` with the pitch path.

## Usage

```text
malvin delight [GUIDANCE] [OPTIONS]
```

### `[GUIDANCE]` (optional)

Literal text or path to an existing `.md` file. When provided, malvin resolves the text and embeds it in the composed router request under user guidance. Omitted guidance leaves the delight intent without that block.

## Options

### `--out-path <PATH>` (default: `pitch.md`)

Workspace path for the generated pitch. With the default `pitch.md`, if that file already exists, malvin allocates the first free sibling (`pitch_1.md`, `pitch_2.md`, …) before composing the router request. For any other `--out-path`, if the path already exists (regular file, empty file, directory, or symlink to an existing target), the command exits immediately with:

```text
malvin delight: `<path>` already exists; refusing to overwrite
```

No run artifacts or agent work starts when a non-default path pre-exists.

### `--max-loops <N>` (default: 3)

Outer router session budget (`effective_max_loops`). `0` is treated as `1`.

### `--tenacious` (default: on)

Sets `--max-acp-retries=9999` and `--max-loops=9999`.

### `--no-tenacious`

Restore normal loop/retry budgets (global flag; see `malvin --doc`).

## Global options

See `malvin --doc`. `--quiet` / `-q` applies because delight invokes the default router (DM-body-only stdout; not the same as `-b`).

## Success criteria

All of the following must hold:

1. Preflight passed (default `pitch.md` may have been auto-allocated to a sibling; non-default paths must not have pre-existed).
2. The default router completed within the `--max-loops` budget.

On success, malvin follows the default router exit reporting.

## Related commands

| Command | When |
|---------|------|
| `malvin inspire` | One-shot MBC2 ideation; no pitch file |
| bare `malvin REQUEST` | Same router engine; delight is a thin request wrapper |

## Examples

```text
malvin delight
malvin delight "Improve error messages for gate failures"
malvin delight guidance.md
malvin delight --out-path plans/feature.md
malvin "Implement the plan in plans/feature.md"
```
