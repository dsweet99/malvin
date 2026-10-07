# malvin admin

Operator maintenance commands. No agent session and no run directory under `~/.malvinconf/logs/`.

## Summary

| | |
|---|---|
| Agent session | None |
| `.malvin/` | Not required |
| Output | Short status line on success (or model list for `models`, remote list for `remotes`) |

## Intention

Fix local malvin/herdr bookkeeping or list available model ids and remotes, without starting a research or coding turn. Agent-session flags (`--model`, `-g`, …) are not listed on `admin` help; see `malvin --doc`.

## Usage

```text
malvin admin <COMMAND>
malvin admin models [OPTION]... [PREFIX]...
malvin admin remotes
malvin admin reset-herdr
malvin admin rh
```

`malvin admin` with no subcommand prints a short catalog of the admin subcommands with their descriptions and exits 0, like bare `malvin`.

## Subcommands

### `models`

List `cursor:`, `pi:`, and `codex:` model ids. See `malvin admin models --doc` for the full contract.

### `remotes`

List the remotes that `--remote` accepts (only `modal` today) and each remote's suboptions: their values, defaults, and meaning, plus the GPU types Modal offers. The first line is a one-line summary in the style of `malvin admin models` (`modal<TAB>gpu=none|TYPE[:COUNT] ncpu=N mem=N[G|GB|GiB] timeout=N[s|m|h]`); a table and an example follow. See **Running on Modal** in `malvin --doc`.

malvin does not keep its own list of GPU types. It reads them from the "Specifying GPU type" section of Modal's GPU guide (<https://modal.com/docs/guide/gpu.md>) and caches the list in `~/.malvinconf/modal_gpu_types.json`. It fetches again when the cache is at least 24 hours old, or every time with `--refresh`. If a fetch fails, malvin shows the cached list with a note giving its age; with no cache, it shows only the note. malvin accepts any `gpu=` value of the form `TYPE[:COUNT]`, where `TYPE` uses letters, digits, `-`, `!`, or `+`; it does not check `TYPE` against the list and passes it to Modal unchanged.

Options:

- `--refresh`: fetch the GPU types now, even if the cache is less than 24 hours old.

### `reset-herdr`

Alias: `rh`.

Release the current herdr pane's malvin agent so the agents-list entry goes away, and clear display metadata.

Requires a herdr-hosted environment: `HERDR_ENV=1`, `HERDR_SOCKET_PATH`, and `HERDR_PANE_ID`. Useful when a prior malvin process exited without tearing down and the pane still shows `working` or a leftover idle circle.
