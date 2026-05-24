# malvin init

Prepare a workspace (repository) for malvin: install default config files, hook pre-commit, run `kiss init`, optionally create an initial git commit, then run a short agent summary of what was installed.

## Intention

Turn an empty or existing directory into a malvin-aware repo with language-appropriate hooks, kiss configuration, and git hygiene—without requiring the user to hand-copy templates from the malvin source tree.

## Usage

```text
malvin init [OPTIONS] <LANGUAGES>...
```

## Arguments

### `<LANGUAGES>...` (required, one or more)

Languages to enable in generated `.pre-commit-config.yaml`:

- `python` — adds ruff hook
- `rust` — adds clippy hook

At least one language is required. Duplicates are ignored. Unknown values error.

`kiss` and an untracked-files check hook are always included regardless of language list.

## Options

### `--force`

Overwrite files installed from malvin’s bundled `default_repo/` templates and refresh `admin/check_untracked.sh`. Without `--force`, existing files are left unchanged.

### `--path <PATH>`

Target directory. Default: current working directory. Created if missing.

### Global options

`--model`, `--no-force`, `--no-tee`, `--no-markdown`, `--verbose`, `--no-color` — see `malvin.md`. Init uses the agent only for the summary phase; `--no-markdown` affects that session’s stdout styling.

Init-specific: `--force` on the subcommand is separate from global `--no-force` (agent `--force`).

## What init does (non-agent steps)

1. Write templates: `.gitignore`, `.kissignore`, `.pre-commit-config.yaml`, `admin/check_untracked.sh` (respecting `--force`).
2. Run `pre-commit install` in the target directory.
3. Require `kiss` on PATH and run `kiss init`.
4. Install git LFS if available.
5. If the repo has no commits yet: `git add .`, initial commit as malvin (with `--no-verify` to avoid bootstrap cycle). Does not rename the current branch.
6. Ensure `.malvin/checks`, `.malvin/advice.md`, and `.malvin/logs/` exist (checks seeded with language-appropriate defaults; nextest used when `cargo nextest --version` succeeds).

## Prompt workflow

Init runs **one** coder session after filesystem bootstrap.

| Step | Prompt role (effect) | Log |
|------|----------------------|-----|
| 1 | **Identity + context header** — Establishes malvin persona, run paths, and workspace context for a coding-style session. | Combined with step 2 |
| 2 | **Summary** — Agent explains what was installed and how to use the repo with malvin; user-facing wrap-up. | `summary.log` (label `summary`) |

Session dotfiles (`.kissconfig`, `.kissignore`, `.malvin/checks`) are backed up before the agent runs and restored afterward so init does not permanently leave agent mutations in those files.

## Requirements

- `pre-commit` on PATH (`pip install pre-commit`)
- `kiss` on PATH
- `git` for optional initial commit
- Cursor agent CLI for the summary phase

## Does not require

kiss pre-check at CLI entry (unlike `code` / `tidy`); `kiss init` is run as part of init itself.

## Example

```text
malvin init python rust --path ~/myproject
```
