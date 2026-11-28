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

`reset-herdr` cannot help when the pane holds a saved session from another agent, such as an interactive Codex session that has since exited. Herdr keeps that session and ignores malvin's status reports in that pane, and its API offers malvin no way to clear it. While malvin runs, Herdr also cannot see the saved session's process exit, so malvin cannot clear it either. When malvin detects this at startup, it prints a warning once and writes it to `herdr.log` in the run directory. To free the pane, start that agent (for example `codex` or `claude`) from the pane's top-level shell and quit it. Herdr drops the saved session only when it sees the agent exit back to that shell. Do this outside `pixi shell` or any other nested shell. `pixi shell` runs its shell on a pseudo-terminal of its own, so Herdr never sees an agent start or exit there. After an exit into a plain nested shell, Herdr keeps the session until that nested shell exits too. So type `exit` until you are back at the pane's original shell, start the agent, quit it, and then re-enter `pixi shell` if you need it. Alternatively, run malvin in another tab. Tabs in the same workspace can each host a malvin.

### `facts add`

`malvin admin facts add field_name=value [field_name=value ...]` appends one fact to `~/.malvinconf/facts.jsonl` and exits (no agent, no run logs). It is meant for LLM agents: fields may come in any order, and bad input is rejected with an error and exit status 1.

- Fields: `fact_id` (required; lowercase letters, digits, `-`, `_`; starts with a letter; at most 64 characters; must not already exist in the file), `description` (optional; at most 7 words; defaults to the first 7 words of `fact`), `fact` (required; at most 4000 characters), and `symptoms` (optional; repeat the field once per symptom; at most 20, each at most 500 characters).
- Values are trimmed, must be non-empty, and must be a single line without control or invisible formatting characters. Unknown or repeated `fact_id`/`fact` fields are errors. Quote values that contain spaces.
- The file and its directory are created if missing. A lock file `facts.jsonl.lock` guards the uniqueness check; remove it if a crashed run left it behind.
- Example: `malvin admin facts add fact_id=tectonic-path fact="Only Tectonic is installed." symptoms="pdflatex not found"`

### `facts remove`

`malvin admin facts remove FACT_ID` deletes the fact with that `fact_id` from `~/.malvinconf/facts.jsonl` and exits. An unknown id, an invalid id, or a missing file is an error with exit status 1. Other lines are left untouched. The same `facts.jsonl.lock` lock guards the rewrite.
