# malvin init

Bootstrap a workspace for malvin: templates, pre-commit, `kiss init`, optional initial git commit, checks discovery on existing repos, then a short agent summary.

## Summary

| | |
|---|---|
| Input | One or more languages: `python`, `rust` |
| Agent | KPop checks discovery on existing repos (when applicable), then one summary session |
| kiss check at CLI | Not required (unlike `code` / `tidy`) |

## Intention

Turn an empty or existing directory into a malvin-aware repo without hand-copying templates from the malvin source tree. On **existing** repos, an agent discovers `.malvin/checks` from pre-commit, Makefile, CI, and related signals. **Empty** repos (no commits and no meaningful source/tooling artifacts) keep language-based defaults with no discovery agent and no summary.

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

Overwrite files from malvin’s bundled `default_repo/` templates and refresh `admin/check_untracked.sh`. When `.malvin/checks` already exists, re-seed provisional builtins and re-run checks discovery. Without `--force`, existing files are left unchanged and discovery is skipped if checks are already present.

### `--path <PATH>`

Target directory (default: cwd). Created if missing.

### Global options

See `malvin --doc`. Init uses the agent for discovery (when it runs) and the summary phase; `--no-markdown` affects those sessions. Subcommand `--force` is unrelated to global `--no-force` (agent tool approval).

## What init does (before the agent)

1. Write templates: `.gitignore`, `.kissignore`, `.pre-commit-config.yaml`, `admin/check_untracked.sh` (respecting `--force`).
2. Run `git init` when the target is not already inside a git work tree (default branch `main`).
3. Run `pre-commit install`.
4. Require `kiss` on PATH; run `kiss init`.
5. Install git LFS if available.
6. If the repo has no commits: `git add .`, initial commit as malvin (`--no-verify`). Does not rename the current branch.
7. Ensure `.malvin/checks`, `{{ advice_path }}`, `.malvin/config.toml`, and `.malvin/logs/` exist (checks seeded with language-appropriate defaults when missing).

## Checks discovery (existing repos)

When the repo is not empty for discovery and `.malvin/checks` was missing (or `--force` with existing checks), malvin runs a KPop loop with `init_constraints.md`:

| Step | Role | Log |
|------|------|-----|
| KPop | Agent inspects tooling and writes `.malvin/checks` | `_kpop/exp_log_*_gN.md` |
| Summary | Agent explains what was installed and which checks were chosen | `summary.log` |

Discovery uses **discovery semantics**: a single `## KPOP_SOLVED` and a structurally valid checks file are enough; gates need not pass on first run. Session restore during discovery skips `.malvin/checks` so agent edits persist; kiss config, kissignore, and malvin config still restore per turn.

Skip discovery when:

- Empty repo (no commits and no `.py`/`.rs`/tests/Cargo.toml/pyproject/pre-commit signals), or
- `.malvin/checks` already exists and `--force` is not set.

## Prompt workflow (summary)

One coder session after bootstrap (skipped on the empty-repo fast path):

| Step | Role | Log |
|------|------|-----|
| Identity + context header | Malvin persona and workspace paths | Combined with summary |
| Summary | Agent explains what was installed | `summary.log` |

Session dotfiles (`.kissconfig`, `.kissignore`, `.malvin/config.toml`, and `.malvin/checks` except during discovery) are backed up before the summary agent and restored afterward.

## Requirements

- `pre-commit` on PATH
- `kiss` on PATH
- `git` for optional initial commit
- Cursor agent CLI for discovery and summary when those phases run

## Examples

```text
malvin init python rust --path ~/myproject
malvin init rust
```
