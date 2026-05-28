# malvin init

Bootstrap a workspace for malvin: templates, pre-commit, `kiss init`, optional initial git commit, then a short agent summary.

## Summary

| | |
|---|---|
| Input | One or more languages: `python`, `rust` |
| Agent | One summary session after filesystem setup |
| kiss check at CLI | Not required (unlike `code` / `tidy`) |

## Intention

Turn an empty or existing directory into a malvin-aware repo without hand-copying templates from the malvin source tree.

## Usage

```text
malvin init [OPTIONS] <LANGUAGES>...
```

## Arguments

### `<LANGUAGES>...` (required, one or more)

| Language | Effect on `.pre-commit-config.yaml` |
|----------|-----------------------------------|
| `python` | Adds ruff hook |
| `rust` | Adds clippy hook |

At least one language is required. Duplicates are ignored. Unknown values error.

`kiss` and an untracked-files check hook are always included.

## Options

### `--force`

Overwrite files from malvin’s bundled `default_repo/` templates and refresh `admin/check_untracked.sh`. Without `--force`, existing files are left unchanged.

### `--path <PATH>`

Target directory (default: cwd). Created if missing.

### Global options

See `malvin --doc`. Init uses the agent only for the summary phase; `--no-markdown` affects that session. Subcommand `--force` is unrelated to global `--no-force` (agent tool approval).

## What init does (before the agent)

1. Write templates: `.gitignore`, `.kissignore`, `.pre-commit-config.yaml`, `admin/check_untracked.sh` (respecting `--force`).
2. Run `pre-commit install`.
3. Require `kiss` on PATH; run `kiss init`.
4. Install git LFS if available.
5. If the repo has no commits: `git add .`, initial commit as malvin (`--no-verify`). Does not rename the current branch.
6. Ensure `.malvin/checks`, `{{ advice_path }}`, `.malvin/config.toml`, and `.malvin/logs/` exist (checks seeded with language-appropriate defaults; nextest when `cargo nextest --version` succeeds).

## Prompt workflow

One coder session after filesystem bootstrap:

| Step | Role | Log |
|------|------|-----|
| Identity + context header | Malvin persona and workspace paths | Combined with summary |
| Summary | Agent explains what was installed | `summary.log` |

Session dotfiles (`.kissconfig`, `.kissignore`, `.malvin/checks`) are backed up before the agent and restored afterward.

## Requirements

- `pre-commit` on PATH
- `kiss` on PATH
- `git` for optional initial commit
- Cursor agent CLI for the summary phase

## Examples

```text
malvin init python rust --path ~/myproject
malvin init rust
```
