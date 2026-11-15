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
malvin admin facts add FIELD=VALUE...
malvin admin facts remove FACT_ID
```

`malvin admin` with no subcommand prints a short catalog of the admin subcommands with their descriptions and exits 0, like bare `malvin`.

## Subcommands

### `models`

List `cursor:`, `pi:`, and `codex:` model ids. See `malvin admin models --doc` for the full contract.

### `remotes`

List the remotes that `--remote` accepts (only `modal:sandbox` today) and each remote's suboptions: their values, defaults, and meaning, plus the GPU types Modal offers. The first line is a one-line summary in the style of `malvin admin models` (`modal:sandbox<TAB>gpu=none|TYPE[:COUNT] ncpu=N mem=N[G|GB|GiB] timeout=N[s|m|h]`); a table and an example follow. See **Running on Modal** in `malvin --doc`.

malvin does not keep its own list of GPU types. It reads them from the "Specifying GPU type" section of Modal's GPU guide (<https://modal.com/docs/guide/gpu.md>) and caches the list in `~/.malvinconf/modal_gpu_types.json`. It fetches again when the cache is at least 24 hours old, or every time with `--refresh`. If a fetch fails, malvin shows the cached list with a note giving its age; with no cache, it shows only the note. malvin accepts any `gpu=` value of the form `TYPE[:COUNT]`, where `TYPE` uses letters, digits, `-`, `!`, or `+`; it does not check `TYPE` against the list and passes it to Modal unchanged.

Options:

- `--refresh`: fetch the GPU types now, even if the cache is less than 24 hours old.

### `reset-herdr`

Alias: `rh`.

Release the current herdr pane's malvin agent so the agents-list entry goes away, and clear display metadata.

Requires a herdr-hosted environment: `HERDR_ENV=1`, `HERDR_SOCKET_PATH`, and `HERDR_PANE_ID`. Useful when a prior malvin process exited without tearing down and the pane still shows `working` or a leftover idle circle.

### `facts add`

`malvin admin facts add field_name=value [field_name=value ...]` appends one fact to `~/.malvinconf/facts.jsonl` and exits (no agent, no run logs). It is meant for LLM agents: fields may come in any order, and bad input is rejected with an error and exit status 1.

- Fields: `fact_id` (required; lowercase letters, digits, `-`, `_`; starts with a letter; at most 64 characters; must not already exist in the file), `fact` (required; at most 4000 characters), and `symptoms` (optional; repeat the field once per symptom; at most 20, each at most 500 characters).
- Values are trimmed, must be non-empty, and must be a single line without control or invisible formatting characters. Unknown or repeated `fact_id`/`fact` fields are errors. Quote values that contain spaces.
- The file and its directory are created if missing. A lock file `facts.jsonl.lock` guards the uniqueness check; remove it if a crashed run left it behind.
- Example: `malvin admin facts add fact_id=tectonic-path fact="Only Tectonic is installed." symptoms="pdflatex not found"`

### `facts remove`

`malvin admin facts remove FACT_ID` deletes the fact with that `fact_id` from `~/.malvinconf/facts.jsonl` and exits. An unknown id, an invalid id, or a missing file is an error with exit status 1. Other lines are left untouched. The same `facts.jsonl.lock` lock guards the rewrite.
